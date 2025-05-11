// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.28;

import {MerkleProof} from "openzeppelin/utils/cryptography/MerkleProof.sol";
import {Verification} from "./Verification.sol";
import {Initializer} from "./Initializer.sol";
import {AgentExecutor} from "./AgentExecutor.sol";
import {IGatewayBase} from "./interfaces/IGatewayBase.sol";
import {
    OperatingMode,
    ParaID,
    Channel,
    ChannelID,
    MultiAddress,
    InboundMessageV1,
    CommandV1,
    InboundMessageV2,
    CommandV2,
    CommandKind,
    CallsV1,
    HandlersV1,
    CallsV2,
    HandlersV2,
    IGatewayV1,
    IGatewayV2
} from "./Types.sol";
import {Network} from "./v2/Types.sol";
import {Upgrade} from "./Upgrade.sol";
import {IInitializable} from "./interfaces/IInitializable.sol";
import {IUpgradable} from "./interfaces/IUpgradable.sol";
import {ERC1967} from "./utils/ERC1967.sol";
import {Address} from "./utils/Address.sol";
import {SafeNativeTransfer} from "./utils/SafeTransfer.sol";
import {Math} from "./utils/Math.sol";
import {ScaleCodec} from "./utils/ScaleCodec.sol";
import {Functions} from "./Functions.sol";
import {Constants} from "./Constants.sol";

import {CoreStorage} from "./storage/CoreStorage.sol";
import {PricingStorage} from "./storage/PricingStorage.sol";
import {AssetsStorage} from "./storage/AssetsStorage.sol";

import {UD60x18, ud60x18, convert} from "prb/math/src/UD60x18.sol";

contract Gateway is IGatewayBase, IGatewayV1, IGatewayV2, IInitializable, IUpgradable {
    using Address for address;
    using SafeNativeTransfer for address payable;

    // Address of the code to be run within `Agent.sol` using delegatecall
    address public immutable AGENT_EXECUTOR;

    // Consensus client for Polkadot
    address public immutable BEEFY_CLIENT;

    // Message handlers can only be dispatched by the gateway itself
    modifier onlySelf() {
        if (msg.sender != address(this)) {
            revert IGatewayBase.Unauthorized();
        }
        _;
    }

    // Makes functions nonreentrant
    modifier nonreentrant() {
        assembly {
            if tload(0) { revert(0, 0) }

            // Set the flag to mark the function is currently executing.
            tstore(0, 1)
        }
        _;
        // Unlocks the guard, making the pattern composable.
        // After the function exits, it can be called again, even in the same transaction.
        assembly {
            tstore(0, 0)
        }
    }

    constructor(address beefyClient, address agentExecutor) {
        BEEFY_CLIENT = beefyClient;
        AGENT_EXECUTOR = agentExecutor;
    }

    /*
    *     _________
    *     \_   ___ \   ____    _____    _____    ____    ____
    *     /    \  \/  /  _ \  /     \  /     \  /  _ \  /    \
    *     \     \____(  <_> )|  Y Y  \|  Y Y  \(  <_> )|   |  \
    *      \______  / \____/ |__|_|  /|__|_|  / \____/ |___|  /
    *             \/               \/       \/              \/
    */

    // Verify that a message commitment is considered finalized by our BEEFY light client.
    function _verifyCommitment(bytes32 commitment, Verification.Proof calldata proof, bool isV2)
        internal
        view
        virtual
        returns (bool)
    {
        return Verification.verifyCommitment(
            BEEFY_CLIENT,
            ScaleCodec.encodeU32(uint32(ParaID.unwrap(Constants.BRIDGE_HUB_PARA_ID))),
            commitment,
            proof,
            isV2
        );
    }

    /*
    *     _____   __________ .___          ____
    *    /  _  \  \______   \|   | ___  __/_   |
    *   /  /_\  \  |     ___/|   | \  \/ / |   |
    *  /    |    \ |    |    |   |  \   /  |   |
    *  \____|__  / |____|    |___|   \_/   |___|
    *          \/
    */

    /**
     * APIv1 Constants
     */

    // Gas used for:
    // 1. Mapping a command id to an implementation function
    // 2. Calling implementation function
    uint256 constant DISPATCH_OVERHEAD_GAS_V1 = 10_000;

    /**
     * APIv1 External API
     */

    /// @dev Submit a message from Polkadot for verification and dispatch
    /// @param message A message produced by the OutboundQueue pallet on BridgeHub
    /// @param leafProof A message proof used to verify that the message is in the merkle tree
    ///        committed by the OutboundQueue pallet.
    /// @param headerProof A proof that the commitment is included in parachain header that was
    ///        finalized by BEEFY.
    function submitV1(
        InboundMessageV1 calldata message,
        bytes32[] calldata leafProof,
        Verification.Proof calldata headerProof
    ) external nonreentrant {
        uint256 startGas = gasleft();

        Channel storage channel = Functions.ensureChannel(message.channelID);

        // Ensure this message is not being replayed
        if (message.nonce != channel.inboundNonce + 1) {
            revert IGatewayBase.InvalidNonce();
        }

        // Increment nonce for origin.
        // This also prevents the re-entrancy case in which a malicious party tries to re-enter by
        // calling `submitInbound` again with the same (message, leafProof, headerProof) arguments.
        channel.inboundNonce++;

        // Produce the commitment (message root) by applying the leaf proof to the message leaf
        bytes32 leafHash = keccak256(abi.encode(message));
        bytes32 commitment = MerkleProof.processProof(leafProof, leafHash);

        // Verify that the commitment is included in a parachain header finalized by BEEFY.
        if (!_verifyCommitment(commitment, headerProof, false)) {
            revert IGatewayBase.InvalidProof();
        }

        // Make sure relayers provide enough gas so that inner message dispatch
        // does not run out of gas.
        uint256 maxDispatchGas = message.maxDispatchGas;
        if (gasleft() < maxDispatchGas + DISPATCH_OVERHEAD_GAS_V1) {
            revert IGatewayBase.NotEnoughGas();
        }

        bool success = true;

        // Dispatch message to a handler
        if (message.command == CommandV1.AgentExecute) {
            try Gateway(this).v1_handleAgentExecute{gas: maxDispatchGas}(message.params) {}
            catch {
                success = false;
            }
        } else if (message.command == CommandV1.SetOperatingMode) {
            try Gateway(this).v1_handleSetOperatingMode{gas: maxDispatchGas}(message.params) {}
            catch {
                success = false;
            }
        } else if (message.command == CommandV1.Upgrade) {
            try Gateway(this).v1_handleUpgrade{gas: maxDispatchGas}(message.params) {}
            catch {
                success = false;
            }
        } else if (message.command == CommandV1.SetTokenTransferFees) {
            try Gateway(this).v1_handleSetTokenTransferFees{gas: maxDispatchGas}(message.params) {}
            catch {
                success = false;
            }
        } else if (message.command == CommandV1.SetPricingParameters) {
            try Gateway(this).v1_handleSetPricingParameters{gas: maxDispatchGas}(message.params) {}
            catch {
                success = false;
            }
        } else if (message.command == CommandV1.UnlockNativeToken) {
            try Gateway(this).v1_handleUnlockNativeToken{gas: maxDispatchGas}(message.params) {}
            catch {
                success = false;
            }
        } else if (message.command == CommandV1.RegisterForeignToken) {
            try Gateway(this).v1_handleRegisterForeignToken{gas: maxDispatchGas}(message.params) {}
            catch {
                success = false;
            }
        } else if (message.command == CommandV1.MintForeignToken) {
            try Gateway(this).v1_handleMintForeignToken{gas: maxDispatchGas}(
                message.channelID, message.params
            ) {} catch {
                success = false;
            }
        } else {
            success = false;
        }

        // Calculate a gas refund, capped to protect against huge spikes in `tx.gasprice`
        // that could drain funds unnecessarily. During these spikes, relayers should back off.
        uint256 gasUsed = v1_transactionBaseGas() + (startGas - gasleft());
        uint256 refund = gasUsed * Math.min(tx.gasprice, message.maxFeePerGas);

        // Add the reward to the refund amount. If the sum is more than the funds available
        // in the gateway, then reduce the total amount
        uint256 amount = Math.min(refund + message.reward, address(this).balance);

        // Do the payment if there funds available in the gateway
        if (amount > Functions.dustThreshold()) {
            payable(msg.sender).safeNativeTransfer(amount);
        }

        emit IGatewayV1.InboundMessageDispatched(
            message.channelID, message.nonce, message.id, success
        );
    }

    function operatingMode()
        external
        view
        override(IGatewayV1, IGatewayV2)
        returns (OperatingMode)
    {
        return CoreStorage.layout().mode;
    }

    function channelOperatingModeOf(ChannelID channelID) external view returns (OperatingMode) {
        return CallsV1.channelOperatingModeOf(channelID);
    }

    function channelNoncesOf(ChannelID channelID) external view returns (uint64, uint64) {
        return CallsV1.channelNoncesOf(channelID);
    }

    function agentOf(bytes32 agentID)
        external
        view
        override(IGatewayV1, IGatewayV2)
        returns (address)
    {
        return Functions.ensureAgent(agentID);
    }

    function pricingParameters() external view returns (UD60x18, uint128) {
        return CallsV1.pricingParameters();
    }

    function implementation() public view returns (address) {
        return ERC1967.load();
    }

    function isTokenRegistered(address token)
        external
        view
        override(IGatewayV1, IGatewayV2)
        returns (bool)
    {
        return CallsV1.isTokenRegistered(token);
    }

    function depositEther() external payable {
        emit Deposited(msg.sender, msg.value);
    }

    function queryForeignTokenID(address token) external view returns (bytes32) {
        return AssetsStorage.layout().tokenRegistry[token].foreignID;
    }

    // Total fee for registering a token
    function quoteRegisterTokenFee() external view returns (uint256) {
        return CallsV1.quoteRegisterTokenFee();
    }

    // Register an Ethereum-native token in the gateway and on AssetHub
    function registerToken(address token) external payable nonreentrant {
        CallsV1.registerToken(token);
    }

    // Total fee for sending a token
    function quoteSendTokenFee(address token, ParaID destinationChain, uint128 destinationFee)
        external
        view
        returns (uint256)
    {
        return CallsV1.quoteSendTokenFee(token, destinationChain, destinationFee);
    }

    // Transfer ERC20 tokens to a Polkadot parachain
    function sendToken(
        address token,
        ParaID destinationChain,
        MultiAddress calldata destinationAddress,
        uint128 destinationFee,
        uint128 amount
    ) external payable nonreentrant {
        CallsV1.sendToken(
            token, msg.sender, destinationChain, destinationAddress, destinationFee, amount
        );
    }

    // @dev Get token address by tokenID
    function tokenAddressOf(bytes32 tokenID) external view returns (address) {
        return CallsV1.tokenAddressOf(tokenID);
    }

    /**
     * APIv1 Inbound Message Handlers
     */

    // Execute code within an agent
    function v1_handleAgentExecute(bytes calldata data) external onlySelf {
        HandlersV1.agentExecute(AGENT_EXECUTOR, data);
    }

    /// @dev Perform an upgrade of the gateway
    function v1_handleUpgrade(bytes calldata data) external onlySelf {
        HandlersV1.upgrade(data);
    }

    // @dev Set the operating mode of the gateway
    function v1_handleSetOperatingMode(bytes calldata data) external onlySelf {
        HandlersV1.setOperatingMode(data);
    }

    // @dev Set token fees of the gateway
    function v1_handleSetTokenTransferFees(bytes calldata data) external onlySelf {
        HandlersV1.setTokenTransferFees(data);
    }

    // @dev Set pricing params of the gateway
    function v1_handleSetPricingParameters(bytes calldata data) external onlySelf {
        HandlersV1.setPricingParameters(data);
    }

    // @dev Transfer Ethereum native token back from polkadot
    function v1_handleUnlockNativeToken(bytes calldata data) external onlySelf {
        HandlersV1.unlockNativeToken(AGENT_EXECUTOR, data);
    }

    // @dev Register a new fungible Polkadot token for an agent
    function v1_handleRegisterForeignToken(bytes calldata data) external onlySelf {
        HandlersV1.registerForeignToken(data);
    }

    // @dev Mint foreign token from polkadot
    function v1_handleMintForeignToken(ChannelID channelID, bytes calldata data)
        external
        onlySelf
    {
        HandlersV1.mintForeignToken(channelID, data);
    }

    /**
     * APIv1 Internal functions
     */

    // Best-effort attempt at estimating the base gas use of `submitInbound` transaction, outside
    // the block of code that is metered.
    // This includes:
    // * Cost paid for every transaction: 21000 gas
    // * Cost of calldata: Zero byte = 4 gas, Non-zero byte = 16 gas
    // * Cost of code inside submitInitial that is not metered: 14_698
    //
    // The major cost of calldata are the merkle proofs, which should dominate anything else
    // (including the message payload) Since the merkle proofs are hashes, they are much more
    // likely to be composed of more non-zero bytes than zero bytes.
    //
    // Reference: Ethereum Yellow Paper
    function v1_transactionBaseGas() internal pure returns (uint256) {
        return 21_000 + 14_698 + (msg.data.length * 16);
    }

    /*
    *     _____   __________ .___         ________
    *    /  _  \  \______   \|   | ___  __\_____  \
    *   /  /_\  \  |     ___/|   | \  \/ / /  ____/§
    *  /    |    \ |    |    |   |  \   / /       \
    *  \____|__  / |____|    |___|   \_/  \_______ \
    *          \/                                 \/
    */

    /// Overhead in selecting the dispatch handler for an arbitrary command
    uint256 internal constant DISPATCH_OVERHEAD_GAS_V2 = 24_000;

    function v2_submit(
        InboundMessageV2 calldata message,
        bytes32[] calldata leafProof,
        Verification.Proof calldata headerProof,
        bytes32 rewardAddress
    ) external nonreentrant {
        CoreStorage.Layout storage $ = CoreStorage.layout();

        if ($.inboundNonce.get(message.nonce)) {
            revert IGatewayBase.InvalidNonce();
        }

        bytes32 leafHash = keccak256(abi.encode(message));

        $.inboundNonce.set(message.nonce);

        // Produce the commitment (message root) by applying the leaf proof to the message leaf
        bytes32 commitment = MerkleProof.processProof(leafProof, leafHash);

        // Verify that the commitment is included in a parachain header finalized by BEEFY.
        if (!_verifyCommitment(commitment, headerProof, true)) {
            revert IGatewayBase.InvalidProof();
        }

        // Dispatch the message payload. The boolean returned indicates whether all commands succeeded.
        bool success = v2_dispatch(message);

        // Emit the event with a success value "true" if all commands successfully executed, otherwise "false"
        // if all or some of the commands failed.
        emit IGatewayV2.InboundMessageDispatched(
            message.nonce, message.topic, success, rewardAddress
        );
    }

    function v2_outboundNonce() external view returns (uint64) {
        return CallsV2.outboundNonce();
    }

    function v2_isDispatched(uint64 nonce) external view returns (bool) {
        return CoreStorage.layout().inboundNonce.get(nonce);
    }

    // See docs for `IGateway.v2_sendMessage`
    function v2_sendMessage(
        bytes calldata xcm,
        bytes[] calldata assets,
        bytes calldata claimer,
        uint128 executionFee,
        uint128 relayerFee
    ) external payable nonreentrant {
        CallsV2.sendMessage(xcm, assets, claimer, executionFee, relayerFee);
    }

    // See docs for `IGateway.v2_registerToken`
    function v2_registerToken(
        address token,
        uint8 network,
        uint128 executionFee,
        uint128 relayerFee
    ) external payable nonreentrant {
        require(network == uint8(Network.Polkadot), IGatewayV2.InvalidNetwork());
        CallsV2.registerToken(token, Network(network), executionFee, relayerFee);
    }

    // See docs for `IGateway.v2_createAgent`
    function v2_createAgent(bytes32 id) external {
        CallsV2.createAgent(id);
    }

    /**
     * APIv2 Message Handlers
     */

    //  Perform an upgrade of the gateway
    function v2_handleUpgrade(bytes calldata data) external onlySelf {
        HandlersV2.upgrade(data);
    }

    // Set the operating mode of the gateway
    function v2_handleSetOperatingMode(bytes calldata data) external onlySelf {
        HandlersV2.setOperatingMode(data);
    }

    // Unlock Native token
    function v2_handleUnlockNativeToken(bytes calldata data) external onlySelf {
        HandlersV2.unlockNativeToken(AGENT_EXECUTOR, data);
    }

    // Mint foreign token from polkadot
    function v2_handleRegisterForeignToken(bytes calldata data) external onlySelf {
        HandlersV2.registerForeignToken(data);
    }

    // Mint foreign token from polkadot
    function v2_handleMintForeignToken(bytes calldata data) external onlySelf {
        HandlersV2.mintForeignToken(data);
    }

    // Call an arbitrary contract function
    function v2_handleCallContract(bytes32 origin, bytes calldata data) external onlySelf {
        HandlersV2.callContract(origin, AGENT_EXECUTOR, data);
    }

    /**
     * APIv2 Internal functions
     */

    // Internal helper to dispatch a single command
    function _dispatchCommand(CommandV2 calldata command, bytes32 origin, uint64 nonce, uint256 index) internal returns (bool) {
        // check that there is enough gas available to forward to the command handler
        if (gasleft() * 63 / 64 < command.gas + DISPATCH_OVERHEAD_GAS_V2) {
            revert IGatewayV2.InsufficientGasLimit();
        }

        if (command.kind == CommandKind.Upgrade) {
            try Gateway(this).v2_handleUpgrade{gas: command.gas}(command.payload) {} catch {
                return false;
            }
        } else if (command.kind == CommandKind.SetOperatingMode) {
            try Gateway(this).v2_handleSetOperatingMode{gas: command.gas}(command.payload) {} catch {
                return false;
            }
        } else if (command.kind == CommandKind.UnlockNativeToken) {
            try Gateway(this).v2_handleUnlockNativeToken{gas: command.gas}(command.payload) {} catch {
                return false;
            }
        } else if (command.kind == CommandKind.RegisterForeignToken) {
            try Gateway(this).v2_handleRegisterForeignToken{gas: command.gas}(command.payload) {} catch {
                return false;
            }
        } else if (command.kind == CommandKind.MintForeignToken) {
            try Gateway(this).v2_handleMintForeignToken{gas: command.gas}(command.payload) {} catch {
                return false;
            }
        } else if (command.kind == CommandKind.CallContract) {
            try Gateway(this).v2_handleCallContract{gas: command.gas}(origin, command.payload) {} catch {
                return false;
            }
        } else {
            // Unknown command
            return false;
        }
        return true;
    }

    // Dispatch all the commands within the batch of commands in the message payload. Each command is processed
    // independently, and failures emits event CommandFailed without stopping execution of subsequent commands.
    function v2_dispatch(InboundMessageV2 calldata message) internal returns (bool) {
        bool allCommandsSucceeded = true;

        for (uint256 i = 0; i < message.commands.length; i++) {
            bool commandSucceeded = _dispatchCommand(message.commands[i], message.origin, message.nonce, i);
            if (!commandSucceeded) {
                emit IGatewayV2.CommandFailed(message.nonce, i);
                allCommandsSucceeded = false;
            }
        }

        return allCommandsSucceeded;
    }

    /**
     * Upgrades
     */

    /// Initialize storage within the `GatewayProxy` contract using this initializer.
    ///
    /// This initializer cannot be called externally via the proxy as the function selector
    /// is overshadowed in the proxy.
    ///
    /// This implementation is only intended to initialize storage for initial deployments
    /// of the `GatewayProxy` contract to transient or long-lived testnets.
    ///
    /// The `GatewayProxy` deployed to Ethereum mainnet already has its storage initialized.
    /// When its logic contract needs to upgraded, a new logic contract should be developed
    /// that inherits from this base `Gateway` contract. Particularly, the `initialize` function
    /// must be overridden to ensure selective initialization of storage fields relevant
    /// to the upgrade.
    ///
    /// ```solidity
    /// contract Gateway202508 is Gateway {
    ///     function initialize(bytes calldata data) external override {
    ///         if (ERC1967.load() == address(0)) {
    ///             revert Unauthorized();
    ///         }
    ///         # Initialization routines here...
    ///     }
    /// }
    /// ```
    ///
    function initialize(bytes calldata data) external virtual {
        Initializer.initialize(data);
    }
}

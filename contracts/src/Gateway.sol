// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.25;

import {MerkleProof} from "openzeppelin/utils/cryptography/MerkleProof.sol";
import {Verification} from "./Verification.sol";
import {Initializer} from "./Initializer.sol";
import {AgentExecutor} from "./AgentExecutor.sol";
import {Agent} from "./Agent.sol";
import {MultiAddress} from "./MultiAddress.sol";
import {IGatewayBase} from "./interfaces/IGatewayBase.sol";
import {
    OperatingMode,
    ParaID,
    TokenInfo,
    Channel,
    ChannelID,
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
import {Upgrade} from "./Upgrade.sol";
import {IInitializable} from "./interfaces/IInitializable.sol";
import {IUpgradable} from "./interfaces/IUpgradable.sol";
import {ERC1967} from "./utils/ERC1967.sol";
import {Address} from "./utils/Address.sol";
import {SafeNativeTransfer} from "./utils/SafeTransfer.sol";
import {Call} from "./utils/Call.sol";
import {Math} from "./utils/Math.sol";
import {ScaleCodec} from "./utils/ScaleCodec.sol";
import {Functions} from "./Functions.sol";
import {Constants} from "./Constants.sol";

import {CoreStorage} from "./storage/CoreStorage.sol";
import {PricingStorage} from "./storage/PricingStorage.sol";
import {AssetsStorage} from "./storage/AssetsStorage.sol";
import {OperatorStorage} from "./storage/OperatorStorage.sol";

import {UD60x18, ud60x18, convert} from "prb/math/src/UD60x18.sol";

contract Gateway is IGatewayBase, IGatewayV1, IGatewayV2, IInitializable, IUpgradable {
    using Address for address;
    using SafeNativeTransfer for address payable;

    address public immutable AGENT_EXECUTOR;

    // Verification state
    address public immutable BEEFY_CLIENT;

    // BridgeHub
    ParaID internal immutable BRIDGE_HUB_PARA_ID;
    bytes4 internal immutable BRIDGE_HUB_PARA_ID_ENCODED;
    bytes32 internal immutable BRIDGE_HUB_AGENT_ID;

    // Message handlers can only be dispatched by the gateway itself
    modifier onlySelf() {
        if (msg.sender != address(this)) {
            revert IGatewayBase.Unauthorized();
        }
        _;
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
    uint256 DISPATCH_OVERHEAD_GAS_V1 = 10_000;

    /**
     * APIv1 External API
     */

    /// @dev Submit a message from Polkadot for verification and dispatch
    /// @param message A message produced by the OutboundQueue pallet on BridgeHub
    /// @param leafProof A message proof used to verify that the message is in the merkle tree committed by the OutboundQueue pallet
    /// @param headerProof A proof that the commitment is included in parachain header that was finalized by BEEFY.
    function submitV1(
        InboundMessageV1 calldata message,
        bytes32[] calldata leafProof,
        Verification.Proof calldata headerProof
    ) external {
        uint256 startGas = gasleft();

        Channel storage channel = Functions.ensureChannel(message.channelID);

        // Ensure this message is not being replayed
        if (message.nonce != channel.inboundNonce + 1) {
            revert IGatewayBase.InvalidNonce();
        }

        // Increment nonce for origin.
        // This also prevents the re-entrancy case in which a malicious party tries to re-enter by calling `submitInbound`
        // again with the same (message, leafProof, headerProof) arguments.
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
            try Gateway(this).v1_handleAgentExecute{gas: maxDispatchGas}(message.params)
            {} catch {
                success = false;
            }
        } else if (message.command == CommandV1.CreateAgent) {
            try Gateway(this).v1_handleCreateAgent{gas: maxDispatchGas}(message.params) {}
            catch {
                success = false;
            }
        } else if (message.command == CommandV1.CreateChannel) {
            success = false;
        } else if (message.command == CommandV1.UpdateChannel) {
            success = false;
        } else if (message.command == CommandV1.SetOperatingMode) {
            try Gateway(this).v1_handleSetOperatingMode{gas: maxDispatchGas}(
                message.params
            ) {} catch {
                success = false;
            }
        } else if (message.command == CommandV1.TransferNativeFromAgent) {
            try Gateway(this).v1_handleTransferNativeFromAgent{gas: maxDispatchGas}(
                message.params
            ) {} catch {
                success = false;
            }
        } else if (message.command == CommandV1.Upgrade) {
            try Gateway(this).v1_handleUpgrade{gas: maxDispatchGas}(message.params) {}
            catch {
                success = false;
            }
        } else if (message.command == CommandV1.SetTokenTransferFees) {
            try Gateway(this).v1_handleSetTokenTransferFees{gas: maxDispatchGas}(
                message.params
            ) {} catch {
                success = false;
            }
        } else if (message.command == CommandV1.SetPricingParameters) {
            try Gateway(this).v1_handleSetPricingParameters{gas: maxDispatchGas}(
                message.params
            ) {} catch {
                success = false;
            }
        } else if (message.command == CommandV1.UnlockNativeToken) {
            try Gateway(this).v1_handleUnlockNativeToken{gas: maxDispatchGas}(
                message.params
            ) {} catch {
                success = false;
            }
        } else if (message.command == CommandV1.RegisterForeignToken) {
            try Gateway(this).v1_handleRegisterForeignToken{gas: maxDispatchGas}(
                message.params
            ) {} catch {
                success = false;
            }
        } else if (message.command == CommandV1.MintForeignToken) {
            try Gateway(this).v1_handleMintForeignToken{gas: maxDispatchGas}(
                message.params
            ) {} catch {
                success = false;
            }
        }

        // Calculate a gas refund, capped to protect against huge spikes in `tx.gasprice`
        // that could drain funds unnecessarily. During these spikes, relayers should back off.
        uint256 gasUsed = v1_transactionBaseGas() + (startGas - gasleft());
        uint256 refund = gasUsed * Math.min(tx.gasprice, message.maxFeePerGas);

        // Add the reward to the refund amount. If the sum is more than the funds available
        // in the channel agent, then reduce the total amount
        uint256 amount =
            Math.min(refund + message.reward, address(channel.agent).balance);

        // Do the payment if there funds available in the agent
        if (amount > v1_dustThreshold()) {
            Functions.withdrawEther(
                AGENT_EXECUTOR, channel.agent, payable(msg.sender), amount
            );
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

    function channelOperatingModeOf(ChannelID channelID)
        external
        view
        returns (OperatingMode)
    {
        return CallsV1.channelOperatingModeOf(channelID);
    }

    function channelNoncesOf(ChannelID channelID)
        external
        view
        returns (uint64, uint64)
    {
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

    // Total fee for registering a token
    function quoteRegisterTokenFee() external view returns (uint256) {
        return CallsV1.quoteRegisterTokenFee();
    }

    // Register an Ethereum-native token in the gateway and on AssetHub
    function registerToken(address token) external payable {
        CallsV1.registerToken(token);
    }

    // Total fee for sending a token
    function quoteSendTokenFee(
        address token,
        ParaID destinationChain,
        uint128 destinationFee
    ) external view returns (uint256) {
        return CallsV1.quoteSendTokenFee(token, destinationChain, destinationFee);
    }

    // Transfer ERC20 tokens to a Polkadot parachain
    function sendToken(
        address token,
        ParaID destinationChain,
        MultiAddress calldata destinationAddress,
        uint128 destinationFee,
        uint128 amount
    ) external payable {
        CallsV1.sendToken(
            token,
            msg.sender,
            destinationChain,
            destinationAddress,
            destinationFee,
            amount
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

    /// @dev Create an agent for a consensus system on Polkadot
    function v1_handleCreateAgent(bytes calldata data) external onlySelf {
        HandlersV1.createAgent(data);
    }

    /// @dev Perform an upgrade of the gateway
    function v1_handleUpgrade(bytes calldata data) external onlySelf {
        HandlersV1.upgrade(data);
    }

    // @dev Set the operating mode of the gateway
    function v1_handleSetOperatingMode(bytes calldata data) external onlySelf {
        HandlersV1.setOperatingMode(data);
    }

    // @dev Transfer funds from an agent to a recipient account
    function v1_handleTransferNativeFromAgent(bytes calldata data) external onlySelf {
        HandlersV1.transferNativeFromAgent(AGENT_EXECUTOR, data);
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
    function v1_handleMintForeignToken(bytes calldata data) external onlySelf {
        HandlersV1.mintForeignToken(data);
    }

    /**
     * APIv1 Internal functions
     */

    // Best-effort attempt at estimating the base gas use of `submitInbound` transaction, outside the block of
    // code that is metered.
    // This includes:
    // * Cost paid for every transaction: 21000 gas
    // * Cost of calldata: Zero byte = 4 gas, Non-zero byte = 16 gas
    // * Cost of code inside submitInitial that is not metered: 14_698
    //
    // The major cost of calldata are the merkle proofs, which should dominate anything else (including the message payload)
    // Since the merkle proofs are hashes, they are much more likely to be composed of more non-zero bytes than zero bytes.
    //
    // Reference: Ethereum Yellow Paper
    function v1_transactionBaseGas() internal pure returns (uint256) {
        return 21_000 + 14_698 + (msg.data.length * 16);
    }

    /// @dev Define the dust threshold as the minimum cost to transfer ether between accounts
    function v1_dustThreshold() internal view returns (uint256) {
        return 21_000 * tx.gasprice;
    }

    /*
    *     _____   __________ .___         ________
    *    /  _  \  \______   \|   | ___  __\_____  \
    *   /  /_\  \  |     ___/|   | \  \/ / /  ____/§
    *  /    |    \ |    |    |   |  \   / /       \
    *  \____|__  / |____|    |___|   \_/  \_______ \
    *          \/                                 \/
    */

    uint256 public constant DISPATCH_OVERHEAD_GAS_V2 = 32_000;

    /// @dev Submit a message from Polkadot for verification and dispatch
    /// @param message A message produced by the OutboundQueue pallet on BridgeHub
    /// @param leafProof A message proof used to verify that the message is in the merkle tree committed by the OutboundQueue pallet
    /// @param headerProof A proof that the commitment is included in parachain header that was finalized by BEEFY.
    /// @param rewardAddress Account on BH to credit delivery rewards
    function v2_submit(
        InboundMessageV2 calldata message,
        bytes32[] calldata leafProof,
        Verification.Proof calldata headerProof,
        bytes32 rewardAddress
    ) external {
        CoreStorage.Layout storage $ = CoreStorage.layout();

        bytes32 leafHash = keccak256(abi.encode(message));

        if ($.inboundNonce.get(message.nonce)) {
            revert IGatewayBase.InvalidNonce();
        }

        $.inboundNonce.set(message.nonce);

        // Produce the commitment (message root) by applying the leaf proof to the message leaf
        bytes32 commitment = MerkleProof.processProof(leafProof, leafHash);

        // Verify that the commitment is included in a parachain header finalized by BEEFY.
        if (!_verifyCommitment(commitment, headerProof, true)) {
            revert IGatewayBase.InvalidProof();
        }

        bool success = v2_dispatch(message);

        emit IGatewayV2.InboundMessageDispatched(message.nonce, success, rewardAddress);
    }

    function v2_dispatch(InboundMessageV2 calldata message) internal returns (bool) {
        for (uint256 i = 0; i < message.commands.length; i++) {
            if (gasleft() * 63 / 64 < message.commands[i].gas + DISPATCH_OVERHEAD_GAS_V2)
            {
                assembly {
                    invalid()
                }
            }
            if (message.commands[i].kind == CommandKind.Upgrade) {
                try Gateway(this).v2_handleUpgrade{gas: message.commands[i].gas}(
                    message.commands[i].payload
                ) {} catch {
                    return false;
                }
            } else if (message.commands[i].kind == CommandKind.SetOperatingMode) {
                try Gateway(this).v2_handleSetOperatingMode{gas: message.commands[i].gas}(
                    message.commands[i].payload
                ) {} catch {
                    return false;
                }
            } else if (message.commands[i].kind == CommandKind.UnlockNativeToken) {
                try Gateway(this).v2_handleUnlockNativeToken{
                    gas: message.commands[i].gas
                }(message.commands[i].payload) {} catch {
                    return false;
                }
            } else if (message.commands[i].kind == CommandKind.RegisterForeignToken) {
                try Gateway(this).v2_handleRegisterForeignToken{
                    gas: message.commands[i].gas
                }(message.commands[i].payload) {} catch {
                    return false;
                }
            } else if (message.commands[i].kind == CommandKind.MintForeignToken) {
                try Gateway(this).v2_handleMintForeignToken{gas: message.commands[i].gas}(
                    message.commands[i].payload
                ) {} catch {
                    return false;
                }
            } else if (message.commands[i].kind == CommandKind.CreateAgent) {
                try Gateway(this).v2_handleCreateAgent{gas: message.commands[i].gas}(
                    message.origin
                ) {} catch {
                    return false;
                }
            } else if (message.commands[i].kind == CommandKind.CallContract) {
                try Gateway(this).v2_handleCallContract{gas: message.commands[i].gas}(
                    message.origin, message.commands[i].payload
                ) {} catch {
                    return false;
                }
            }
        }
        return true;
    }

    function v2_isDispatched(uint64 nonce) external view returns (bool) {
        CoreStorage.Layout storage $ = CoreStorage.layout();
        return $.inboundNonce.get(nonce);
    }

    // See docs for `IGateway.sendMessage`
    function v2_sendMessage(
        bytes calldata xcm,
        bytes[] calldata assets,
        bytes calldata claimer
    ) external payable {
        CallsV2.sendMessage(xcm, assets, claimer);
    }

    // See docs for `IGateway.registerToken`
    function v2_registerToken(address token, uint128 xcmFeeAHP) external payable {
        CallsV2.registerToken(token, xcmFeeAHP);
    }

    // See docs for `IGateway.registerTokenOnKusama`
    function v2_registerTokenOnKusama(
        address token,
        uint128 xcmFeeAHP,
        uint128 xcmFeeAHK
    ) external payable {
        CallsV2.registerTokenOnKusama(token, xcmFeeAHP, xcmFeeAHK);
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

    // Create an agent for a Polkadot origin
    function v2_handleCreateAgent(bytes32 origin) external onlySelf {
        HandlersV2.createAgent(origin);
    }

    // Call an arbitrary contract function
    function v2_handleCallContract(bytes32 origin, bytes calldata data)
        external
        onlySelf
    {
        HandlersV2.callContract(origin, AGENT_EXECUTOR, data);
    }

    /**
     * Upgrades
     */

    /// @dev Initialize storage in the gateway
    /// NOTE: This is not externally accessible as this function selector is overshadowed in the proxy
    function initialize(bytes calldata data) external virtual {
        Initializer.initialize(data);
    }
}

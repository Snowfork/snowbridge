// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.25;

import {MerkleProof} from "openzeppelin/utils/cryptography/MerkleProof.sol";
import {Verification} from "./Verification.sol";

import {AssetsV1} from "./AssetsV1.sol";
import {AssetsV2} from "./AssetsV2.sol";

import {AgentExecutor} from "./AgentExecutor.sol";
import {Agent} from "./Agent.sol";
import {
    OperatingMode,
    ParaID,
    TokenInfo,
    MultiAddress,
    Channel,
    ChannelID,
    InboundMessageV1,
    CommandV1,
    AgentExecuteCommandV1,
    TicketV1,
    Costs,
    AgentExecuteParamsV1,
    UpgradeParamsV1,
    CreateAgentParamsV1,
    CreateChannelParamsV1,
    UpdateChannelParamsV1,
    SetOperatingModeParamsV1,
    TransferNativeFromAgentParamsV1,
    SetTokenTransferFeesParamsV1,
    SetPricingParametersParamsV1,
    RegisterForeignTokenParamsV1,
    MintForeignTokenParamsV1,
    TransferNativeTokenParamsV1,
    InboundMessageV2,
    CommandV2,
    CommandKindV2,
    TicketV2,
    UpgradeParamsV2,
    SetOperatingModeParamsV2,
    UnlockNativeTokenParamsV2,
    MintForeignTokenParamsV2,
    CallContractParamsV2
} from "./Types.sol";
import {Upgrade} from "./Upgrade.sol";
import {IGateway} from "./interfaces/IGateway.sol";
import {IInitializable} from "./interfaces/IInitializable.sol";
import {IUpgradable} from "./interfaces/IUpgradable.sol";
import {ERC1967} from "./utils/ERC1967.sol";
import {Address} from "./utils/Address.sol";
import {SafeNativeTransfer} from "./utils/SafeTransfer.sol";
import {Call} from "./utils/Call.sol";
import {Math} from "./utils/Math.sol";
import {ScaleCodec} from "./utils/ScaleCodec.sol";

import {CoreStorage} from "./storage/CoreStorage.sol";
import {PricingStorage} from "./storage/PricingStorage.sol";
import {AssetsStorage} from "./storage/AssetsStorage.sol";
import {OperatorStorage} from "./storage/OperatorStorage.sol";

import {UD60x18, ud60x18, convert} from "prb/math/src/UD60x18.sol";

contract Gateway is IGateway, IInitializable, IUpgradable {
    using Address for address;
    using SafeNativeTransfer for address payable;

    address public immutable AGENT_EXECUTOR;

    // Verification state
    address public immutable BEEFY_CLIENT;

    // BridgeHub
    ParaID internal immutable BRIDGE_HUB_PARA_ID;
    bytes4 internal immutable BRIDGE_HUB_PARA_ID_ENCODED;
    bytes32 internal immutable BRIDGE_HUB_AGENT_ID;

    error InvalidProof();
    error InvalidNonce();
    error NotEnoughGas();
    error FeePaymentToLow();
    error Unauthorized();
    error Disabled();
    error AgentAlreadyCreated();
    error AgentDoesNotExist();
    error ChannelAlreadyCreated();
    error ChannelDoesNotExist();
    error InvalidChannelUpdate();
    error AgentExecutionFailed(bytes returndata);
    error InvalidAgentExecutionPayload();
    error InvalidConstructorParams();
    error AlreadyInitialized();
    error TokenNotRegistered();

    // Message handlers can only be dispatched by the gateway itself
    modifier onlySelf() {
        if (msg.sender != address(this)) {
            revert Unauthorized();
        }
        _;
    }

    // handler functions are privileged from agent only
    modifier onlyAgent(bytes32 agentID) {
        bytes32 _agentID = _ensureAgentAddress(msg.sender);
        if (_agentID != agentID) {
            revert Unauthorized();
        }
        _;
    }

    constructor(
        address beefyClient,
        address agentExecutor,
        ParaID bridgeHubParaID,
        bytes32 bridgeHubAgentID,
        uint8 foreignTokenDecimals,
        uint128 maxDestinationFee
    ) {
        if (bridgeHubParaID == ParaID.wrap(0) || bridgeHubAgentID == 0) {
            revert InvalidConstructorParams();
        }

        BEEFY_CLIENT = beefyClient;
        AGENT_EXECUTOR = agentExecutor;
        BRIDGE_HUB_PARA_ID_ENCODED =
            ScaleCodec.encodeU32(uint32(ParaID.unwrap(bridgeHubParaID)));
        BRIDGE_HUB_PARA_ID = bridgeHubParaID;
        BRIDGE_HUB_AGENT_ID = bridgeHubAgentID;
        FOREIGN_TOKEN_DECIMALS = foreignTokenDecimals;
        MAX_DESTINATION_FEE = maxDestinationFee;
    }

    /*
    *     _________
    *     \_   ___ \   ____    _____    _____    ____    ____
    *     /    \  \/  /  _ \  /     \  /     \  /  _ \  /    \
    *     \     \____(  <_> )|  Y Y  \|  Y Y  \(  <_> )|   |  \
    *      \______  / \____/ |__|_|  /|__|_|  / \____/ |___|  /
    *             \/               \/       \/              \/
    */

    /// @dev Ensure that the specified agentID has a corresponding contract
    function _ensureAgent(bytes32 agentID) internal view returns (address agent) {
        agent = CoreStorage.layout().agents[agentID];
        if (agent == address(0)) {
            revert AgentDoesNotExist();
        }
    }

    /// @dev Ensure that the specified address is an valid agent
    function _ensureAgentAddress(address agent)
        internal
        view
        returns (bytes32 agentID)
    {
        agentID = CoreStorage.layout().agentAddresses[agent];
        if (agentID == bytes32(0)) {
            revert AgentDoesNotExist();
        }
    }

    /// @dev Invoke some code within an agent
    function _invokeOnAgent(address agent, bytes memory data)
        internal
        returns (bytes memory)
    {
        (bool success, bytes memory returndata) =
            (Agent(payable(agent)).invoke(AGENT_EXECUTOR, data));
        return Call.verifyResult(success, returndata);
    }

    // Verify that a message commitment is considered finalized by our BEEFY light client.
    function _verifyCommitment(bytes32 commitment, Verification.Proof calldata proof)
        internal
        view
        virtual
        returns (bool)
    {
        return Verification.verifyCommitment(
            BEEFY_CLIENT, BRIDGE_HUB_PARA_ID_ENCODED, commitment, proof
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

    // ChannelIDs
    ChannelID internal constant PRIMARY_GOVERNANCE_CHANNEL_ID =
        ChannelID.wrap(bytes32(uint256(1)));
    ChannelID internal constant SECONDARY_GOVERNANCE_CHANNEL_ID =
        ChannelID.wrap(bytes32(uint256(2)));

    // Gas used for:
    // 1. Mapping a command id to an implementation function
    // 2. Calling implementation function
    uint256 DISPATCH_OVERHEAD_GAS_V1 = 10_000;

    // The maximum fee that can be sent to a destination parachain to pay for execution (DOT).
    // Has two functions:
    // * Reduces the ability of users to perform arbitrage using a favourable exchange rate
    // * Prevents users from mistakenly providing too much fees, which would drain AssetHub's
    //   sovereign account here on Ethereum.
    uint128 internal immutable MAX_DESTINATION_FEE;

    uint8 internal immutable FOREIGN_TOKEN_DECIMALS;

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

        Channel storage channel = _ensureChannelV1(message.channelID);

        // Ensure this message is not being replayed
        if (message.nonce != channel.inboundNonce + 1) {
            revert InvalidNonce();
        }

        // Increment nonce for origin.
        // This also prevents the re-entrancy case in which a malicious party tries to re-enter by calling `submitInbound`
        // again with the same (message, leafProof, headerProof) arguments.
        channel.inboundNonce++;

        // Produce the commitment (message root) by applying the leaf proof to the message leaf
        bytes32 leafHash = keccak256(abi.encode(message));
        bytes32 commitment = MerkleProof.processProof(leafProof, leafHash);

        // Verify that the commitment is included in a parachain header finalized by BEEFY.
        if (!_verifyCommitment(commitment, headerProof)) {
            revert InvalidProof();
        }

        // Make sure relayers provide enough gas so that inner message dispatch
        // does not run out of gas.
        uint256 maxDispatchGas = message.maxDispatchGas;
        if (gasleft() < maxDispatchGas + DISPATCH_OVERHEAD_GAS_V1) {
            revert NotEnoughGas();
        }

        bool success = true;

        // Dispatch message to a handler
        if (message.command == CommandV1.AgentExecute) {
            try Gateway(this).v1_handlerAgentExecute{gas: maxDispatchGas}(message.params)
            {} catch {
                success = false;
            }
        } else if (message.command == CommandV1.CreateAgent) {
            try Gateway(this).v1_handlerCreateAgent{gas: maxDispatchGas}(message.params)
            {} catch {
                success = false;
            }
        } else if (message.command == CommandV1.CreateChannel) {
            success = false;
        } else if (message.command == CommandV1.UpdateChannel) {
            success = false;
        } else if (message.command == CommandV1.SetOperatingMode) {
            try Gateway(this).v1_handlerSetOperatingMode{gas: maxDispatchGas}(
                message.params
            ) {} catch {
                success = false;
            }
        } else if (message.command == CommandV1.TransferNativeFromAgent) {
            try Gateway(this).v1_handlerTransferNativeFromAgent{gas: maxDispatchGas}(
                message.params
            ) {} catch {
                success = false;
            }
        } else if (message.command == CommandV1.Upgrade) {
            try Gateway(this).v1_handlerUpgrade{gas: maxDispatchGas}(message.params) {}
            catch {
                success = false;
            }
        } else if (message.command == CommandV1.SetTokenTransferFees) {
            try Gateway(this).v1_handlerSetTokenTransferFees{gas: maxDispatchGas}(
                message.params
            ) {} catch {
                success = false;
            }
        } else if (message.command == CommandV1.SetPricingParameters) {
            try Gateway(this).v1_handlerSetPricingParameters{gas: maxDispatchGas}(
                message.params
            ) {} catch {
                success = false;
            }
        } else if (message.command == CommandV1.TransferNativeToken) {
            try Gateway(this).v1_handlerTransferNativeToken{gas: maxDispatchGas}(
                message.params
            ) {} catch {
                success = false;
            }
        } else if (message.command == CommandV1.RegisterForeignToken) {
            try Gateway(this).v1_handlerRegisterForeignToken{gas: maxDispatchGas}(
                message.params
            ) {} catch {
                success = false;
            }
        } else if (message.command == CommandV1.MintForeignToken) {
            try Gateway(this).v1_handlerMintForeignToken{gas: maxDispatchGas}(
                message.params
            ) {} catch {
                success = false;
            }
        }

        // Calculate a gas refund, capped to protect against huge spikes in `tx.gasprice`
        // that could drain funds unnecessarily. During these spikes, relayers should back off.
        uint256 gasUsed = _transactionBaseGasV1() + (startGas - gasleft());
        uint256 refund = gasUsed * Math.min(tx.gasprice, message.maxFeePerGas);

        // Add the reward to the refund amount. If the sum is more than the funds available
        // in the channel agent, then reduce the total amount
        uint256 amount =
            Math.min(refund + message.reward, address(channel.agent).balance);

        // Do the payment if there funds available in the agent
        if (amount > _dustThresholdV1()) {
            _transferNativeFromAgentV1(channel.agent, payable(msg.sender), amount);
        }

        emit IGateway.InboundMessageDispatched(
            message.channelID, message.nonce, message.id, success
        );
    }

    function operatingMode() external view returns (OperatingMode) {
        return CoreStorage.layout().mode;
    }

    function channelOperatingModeOf(ChannelID channelID)
        external
        view
        returns (OperatingMode)
    {
        Channel storage ch = _ensureChannelV1(channelID);
        return ch.mode;
    }

    function channelNoncesOf(ChannelID channelID)
        external
        view
        returns (uint64, uint64)
    {
        Channel storage ch = _ensureChannelV1(channelID);
        return (ch.inboundNonce, ch.outboundNonce);
    }

    function agentOf(bytes32 agentID) external view returns (address) {
        return _ensureAgent(agentID);
    }

    function pricingParameters() external view returns (UD60x18, uint128) {
        PricingStorage.Layout storage pricing = PricingStorage.layout();
        return (pricing.exchangeRate, pricing.deliveryCost);
    }

    function implementation() public view returns (address) {
        return ERC1967.load();
    }

    function isTokenRegistered(address token) external view returns (bool) {
        return AssetsV1.isTokenRegistered(token);
    }

    // Total fee for registering a token
    function quoteRegisterTokenFee() external view returns (uint256) {
        return _calculateFeeV1(AssetsV1.registerTokenCosts());
    }

    // Register an Ethereum-native token in the gateway and on AssetHub
    function registerToken(address token) external payable {
        _submitOutboundV1(AssetsV1.registerToken(token));
    }

    // Total fee for sending a token
    function quoteSendTokenFee(
        address token,
        ParaID destinationChain,
        uint128 destinationFee
    ) external view returns (uint256) {
        return _calculateFeeV1(
            AssetsV1.sendTokenCosts(
                token, destinationChain, destinationFee, MAX_DESTINATION_FEE
            )
        );
    }

    // Transfer ERC20 tokens to a Polkadot parachain
    function sendToken(
        address token,
        ParaID destinationChain,
        MultiAddress calldata destinationAddress,
        uint128 destinationFee,
        uint128 amount
    ) external payable {
        TicketV1 memory ticket = AssetsV1.sendToken(
            token,
            msg.sender,
            destinationChain,
            destinationAddress,
            destinationFee,
            MAX_DESTINATION_FEE,
            amount
        );

        _submitOutboundV1(ticket);
    }

    // @dev Get token address by tokenID
    function tokenAddressOf(bytes32 tokenID) external view returns (address) {
        return AssetsV1.tokenAddressOf(tokenID);
    }

    /**
     * APIv1 Inbound Message Handlers
     */

    // Execute code within an agent
    function v1_handlerAgentExecute(bytes calldata data) external onlySelf {
        AgentExecuteParamsV1 memory params = abi.decode(data, (AgentExecuteParamsV1));

        address agent = _ensureAgent(params.agentID);

        if (params.payload.length == 0) {
            revert InvalidAgentExecutionPayload();
        }

        (AgentExecuteCommandV1 command, bytes memory commandParams) =
            abi.decode(params.payload, (AgentExecuteCommandV1, bytes));
        if (command == AgentExecuteCommandV1.TransferToken) {
            (address token, address recipient, uint128 amount) =
                abi.decode(commandParams, (address, address, uint128));
            AssetsV1.transferNativeToken(AGENT_EXECUTOR, agent, token, recipient, amount);
        }
    }

    /// @dev Create an agent for a consensus system on Polkadot
    function v1_handlerCreateAgent(bytes calldata data) external onlySelf {
        CoreStorage.Layout storage $ = CoreStorage.layout();

        CreateAgentParamsV1 memory params = abi.decode(data, (CreateAgentParamsV1));

        // Ensure we don't overwrite an existing agent
        if (address($.agents[params.agentID]) != address(0)) {
            revert AgentAlreadyCreated();
        }

        address payable agent = payable(new Agent(params.agentID));
        $.agents[params.agentID] = agent;

        emit AgentCreated(params.agentID, agent);
    }

    /// @dev Perform an upgrade of the gateway
    function v1_handlerUpgrade(bytes calldata data) external onlySelf {
        UpgradeParamsV1 memory params = abi.decode(data, (UpgradeParamsV1));
        Upgrade.upgrade(params.impl, params.implCodeHash, params.initParams);
    }

    // @dev Set the operating mode of the gateway
    function v1_handlerSetOperatingMode(bytes calldata data) external onlySelf {
        CoreStorage.Layout storage $ = CoreStorage.layout();
        SetOperatingModeParamsV1 memory params =
            abi.decode(data, (SetOperatingModeParamsV1));
        $.mode = params.mode;
        emit OperatingModeChanged(params.mode);
    }

    // @dev Transfer funds from an agent to a recipient account
    function v1_handlerTransferNativeFromAgent(bytes calldata data) external onlySelf {
        TransferNativeFromAgentParamsV1 memory params =
            abi.decode(data, (TransferNativeFromAgentParamsV1));

        address agent = _ensureAgent(params.agentID);

        _transferNativeFromAgentV1(agent, payable(params.recipient), params.amount);
        emit AgentFundsWithdrawn(params.agentID, params.recipient, params.amount);
    }

    // @dev Set token fees of the gateway
    function v1_handlerSetTokenTransferFees(bytes calldata data) external onlySelf {
        AssetsStorage.Layout storage $ = AssetsStorage.layout();
        SetTokenTransferFeesParamsV1 memory params =
            abi.decode(data, (SetTokenTransferFeesParamsV1));
        $.assetHubCreateAssetFee = params.assetHubCreateAssetFee;
        $.assetHubReserveTransferFee = params.assetHubReserveTransferFee;
        $.registerTokenFee = params.registerTokenFee;
        emit TokenTransferFeesChanged();
    }

    // @dev Set pricing params of the gateway
    function v1_handlerSetPricingParameters(bytes calldata data) external onlySelf {
        PricingStorage.Layout storage pricing = PricingStorage.layout();
        SetPricingParametersParamsV1 memory params =
            abi.decode(data, (SetPricingParametersParamsV1));
        pricing.exchangeRate = params.exchangeRate;
        pricing.deliveryCost = params.deliveryCost;
        pricing.multiplier = params.multiplier;
        emit PricingParametersChanged();
    }

    // @dev Register a new fungible Polkadot token for an agent
    function v1_handlerRegisterForeignToken(bytes calldata data) external onlySelf {
        RegisterForeignTokenParamsV1 memory params =
            abi.decode(data, (RegisterForeignTokenParamsV1));
        AssetsV1.registerForeignToken(
            params.foreignTokenID, params.name, params.symbol, params.decimals
        );
    }

    // @dev Mint foreign token from polkadot
    function v1_handlerMintForeignToken(bytes calldata data) external onlySelf {
        MintForeignTokenParamsV1 memory params =
            abi.decode(data, (MintForeignTokenParamsV1));
        AssetsV1.mintForeignToken(params.foreignTokenID, params.recipient, params.amount);
    }

    // @dev Transfer Ethereum native token back from polkadot
    function v1_handlerTransferNativeToken(bytes calldata data) external onlySelf {
        TransferNativeTokenParamsV1 memory params =
            abi.decode(data, (TransferNativeTokenParamsV1));
        address agent = _ensureAgent(params.agentID);
        AssetsV1.transferNativeToken(
            AGENT_EXECUTOR, agent, params.token, params.recipient, params.amount
        );
    }

    /**
     * APIv1 External API
     */

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
    function _transactionBaseGasV1() internal pure returns (uint256) {
        return 21_000 + 14_698 + (msg.data.length * 16);
    }

    // Convert foreign currency to native currency (ROC/KSM/DOT -> ETH)
    function _convertToNativeV1(UD60x18 exchangeRate, UD60x18 multiplier, UD60x18 amount)
        internal
        view
        returns (uint256)
    {
        UD60x18 ethDecimals = convert(1e18);
        UD60x18 foreignDecimals =
            convert(10).pow(convert(uint256(FOREIGN_TOKEN_DECIMALS)));
        UD60x18 nativeAmount = multiplier.mul(amount).mul(exchangeRate).div(
            foreignDecimals
        ).mul(ethDecimals);
        return convert(nativeAmount);
    }

    // Calculate the fee for accepting an outbound message
    function _calculateFeeV1(Costs memory costs) internal view returns (uint256) {
        PricingStorage.Layout storage pricing = PricingStorage.layout();
        UD60x18 amount = convert(pricing.deliveryCost + costs.foreign);
        return costs.native
            + _convertToNativeV1(pricing.exchangeRate, pricing.multiplier, amount);
    }

    // Submit an outbound message to Polkadot, after taking fees
    function _submitOutboundV1(TicketV1 memory ticket) internal {
        ChannelID channelID = ticket.dest.into();
        Channel storage channel = _ensureChannelV1(channelID);

        // Ensure outbound messaging is allowed
        _ensureOutboundMessagingEnabledV1(channel);

        // Destination fee always in DOT
        uint256 fee = _calculateFeeV1(ticket.costs);

        // Ensure the user has enough funds for this message to be accepted
        if (msg.value < fee) {
            revert FeePaymentToLow();
        }

        channel.outboundNonce = channel.outboundNonce + 1;

        // Deposit total fee into agent's contract
        payable(channel.agent).safeNativeTransfer(fee);

        // Reimburse excess fee payment
        if (msg.value > fee) {
            payable(msg.sender).safeNativeTransfer(msg.value - fee);
        }

        // Generate a unique ID for this message
        bytes32 messageID = keccak256(abi.encodePacked(channelID, channel.outboundNonce));

        emit IGateway.OutboundMessageAccepted(
            channelID, channel.outboundNonce, messageID, ticket.payload
        );
    }

    /// @dev Outbound message can be disabled globally or on a per-channel basis.
    function _ensureOutboundMessagingEnabledV1(Channel storage ch) internal view {
        CoreStorage.Layout storage $ = CoreStorage.layout();
        if ($.mode != OperatingMode.Normal || ch.mode != OperatingMode.Normal) {
            revert Disabled();
        }
    }

    /// @dev Ensure that the specified parachain has a channel allocated
    function _ensureChannelV1(ChannelID channelID)
        internal
        view
        returns (Channel storage ch)
    {
        ch = CoreStorage.layout().channels[channelID];
        // A channel always has an agent specified.
        if (ch.agent == address(0)) {
            revert ChannelDoesNotExist();
        }
    }

    /// @dev Transfer ether from an agent
    function _transferNativeFromAgentV1(
        address agent,
        address payable recipient,
        uint256 amount
    ) internal {
        bytes memory call =
            abi.encodeCall(AgentExecutor.transferNative, (recipient, amount));
        _invokeOnAgent(agent, call);
    }

    /// @dev Define the dust threshold as the minimum cost to transfer ether between accounts
    function _dustThresholdV1() internal view returns (uint256) {
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

    bytes32 constant ASSET_HUB_AGENT_ID =
        0x81c5ab2571199e3188135178f3c2c8e2d268be1313d029b30f534fa579b69b79;

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
            revert InvalidNonce();
        }

        $.inboundNonce.set(message.nonce);

        // Produce the commitment (message root) by applying the leaf proof to the message leaf
        bytes32 commitment = MerkleProof.processProof(leafProof, leafHash);

        // Verify that the commitment is included in a parachain header finalized by BEEFY.
        if (!_verifyCommitment(commitment, headerProof)) {
            revert InvalidProof();
        }

        bool success = v2_dispatch(message);

        emit IGateway.InboundMessageDispatched(message.nonce, success, rewardAddress);
    }

    function v2_dispatch(InboundMessageV2 calldata message) internal returns (bool) {
        for (uint256 i = 0; i < message.commands.length; i++) {
            if (gasleft() * 63 / 64 < message.commands[i].gas + DISPATCH_OVERHEAD_GAS_V2)
            {
                assembly {
                    invalid()
                }
            }

            if (message.commands[i].kind == CommandKindV2.Upgrade) {
                UpgradeParamsV2 memory params =
                    abi.decode(message.commands[i].payload, (UpgradeParamsV2));
                try Gateway(this).v2_handleUpgrade{gas: message.commands[i].gas}(
                    params.impl, params.implCodeHash, params.initParams
                ) {} catch {
                    return false;
                }
            } else if (message.commands[i].kind == CommandKindV2.SetOperatingMode) {
                SetOperatingModeParamsV2 memory params =
                    abi.decode(message.commands[i].payload, (SetOperatingModeParamsV2));
                try Gateway(this).v2_handleSetOperatingMode{gas: message.commands[i].gas}(
                    params.mode
                ) {} catch {
                    return false;
                }
            } else if (message.commands[i].kind == CommandKindV2.UnlockNativeToken) {
                UnlockNativeTokenParamsV2 memory params =
                    abi.decode(message.commands[i].payload, (UnlockNativeTokenParamsV2));
                try Gateway(this).v2_handleUnlockNativeToken{
                    gas: message.commands[i].gas
                }(params.token, params.recipient, params.amount) {} catch {
                    return false;
                }
            } else if (message.commands[i].kind == CommandKindV2.MintForeignToken) {
                MintForeignTokenParamsV2 memory params =
                    abi.decode(message.commands[i].payload, (MintForeignTokenParamsV2));
                try Gateway(this).v2_handleMintForeignToken{gas: message.commands[i].gas}(
                    params.foreignTokenID, params.recipient, params.amount
                ) {} catch {
                    return false;
                }
            } else if (message.commands[i].kind == CommandKindV2.CreateAgent) {
                try Gateway(this).v2_handleCreateAgent{gas: message.commands[i].gas}(
                    message.origin
                ) {} catch {
                    return false;
                }
            } else if (message.commands[i].kind == CommandKindV2.CallContract) {
                CallContractParamsV2 memory params =
                    abi.decode(message.commands[i].payload, (CallContractParamsV2));
                try Gateway(this).v2_handleCallContract{gas: message.commands[i].gas}(
                    message.origin, params.target, params.data
                ) {} catch {
                    return false;
                }
            }
        }
        return true;
    }

    /**
     * APIv2 Message Handlers
     */

    //  Perform an upgrade of the gateway
    function v2_handleUpgrade(
        address impl,
        bytes32 implCodeHash,
        bytes memory initParams
    ) external onlySelf {
        Upgrade.upgrade(impl, implCodeHash, initParams);
    }

    // Set the operating mode of the gateway
    function v2_handleSetOperatingMode(OperatingMode mode) external onlySelf {
        CoreStorage.Layout storage $ = CoreStorage.layout();
        $.mode = mode;
        emit OperatingModeChanged(mode);
    }

    // Unlock Native token
    function v2_handleUnlockNativeToken(address token, address recipient, uint128 amount)
        external
        onlySelf
    {
        address agent = _ensureAgent(ASSET_HUB_AGENT_ID);

        AssetsV2.unlockNativeToken(AGENT_EXECUTOR, agent, token, recipient, amount);
    }

    // Mint foreign token from polkadot
    function v2_handleMintForeignToken(
        bytes32 foreignTokenID,
        address recipient,
        uint128 amount
    ) external onlySelf {
        AssetsV2.mintForeignToken(foreignTokenID, recipient, amount);
    }

    function v2_handleCreateAgent(bytes32 origin) external onlySelf {
        CoreStorage.Layout storage core = CoreStorage.layout();
        address agent = CoreStorage.layout().agents[origin];
        if (agent == address(0)) {
            agent = address(new Agent(origin));
            core.agents[origin] = agent;
            core.agentAddresses[agent] = origin;
        }
    }

    function v2_handleCallContract(bytes32 origin, address target, bytes memory data)
        external
        onlySelf
    {
        address agent = _ensureAgent(origin);
        bytes memory call = abi.encodeCall(AgentExecutor.callContract, (target, data));
        _invokeOnAgent(agent, call);
    }

    function sendMessage(bytes calldata xcm, bytes[] calldata assets) external {
        TicketV2 memory ticket = AssetsV2.sendMessage(xcm, assets);
    }

    /**
     * Upgrades
     */

    // Initial configuration for bridge
    struct Config {
        OperatingMode mode;
        /// @dev The fee charged to users for submitting outbound messages (DOT)
        uint128 deliveryCost;
        /// @dev The ETH/DOT exchange rate
        UD60x18 exchangeRate;
        ParaID assetHubParaID;
        bytes32 assetHubAgentID;
        /// @dev The extra fee charged for registering tokens (DOT)
        uint128 assetHubCreateAssetFee;
        /// @dev The extra fee charged for sending tokens (DOT)
        uint128 assetHubReserveTransferFee;
        /// @dev extra fee to discourage spamming
        uint256 registerTokenFee;
        /// @dev Fee multiplier
        UD60x18 multiplier;
        /// @dev Optional rescueOperator
        address rescueOperator;
    }

    /// @dev Initialize storage in the gateway
    /// NOTE: This is not externally accessible as this function selector is overshadowed in the proxy
    function initialize(bytes calldata data) external virtual {
        // Prevent initialization of storage in implementation contract
        if (ERC1967.load() == address(0)) {
            revert Unauthorized();
        }

        CoreStorage.Layout storage core = CoreStorage.layout();

        Config memory config = abi.decode(data, (Config));

        core.mode = config.mode;

        // Initialize agent for BridgeHub
        address bridgeHubAgent = address(new Agent(BRIDGE_HUB_AGENT_ID));
        core.agents[BRIDGE_HUB_AGENT_ID] = bridgeHubAgent;
        core.agentAddresses[bridgeHubAgent] = BRIDGE_HUB_AGENT_ID;

        // Initialize channel for primary governance track
        core.channels[PRIMARY_GOVERNANCE_CHANNEL_ID] = Channel({
            mode: OperatingMode.Normal,
            agent: bridgeHubAgent,
            inboundNonce: 0,
            outboundNonce: 0
        });

        // Initialize channel for secondary governance track
        core.channels[SECONDARY_GOVERNANCE_CHANNEL_ID] = Channel({
            mode: OperatingMode.Normal,
            agent: bridgeHubAgent,
            inboundNonce: 0,
            outboundNonce: 0
        });

        // Initialize agent for for AssetHub
        address assetHubAgent = address(new Agent(config.assetHubAgentID));
        core.agents[config.assetHubAgentID] = assetHubAgent;
        core.agentAddresses[assetHubAgent] = config.assetHubAgentID;

        // Initialize channel for AssetHub
        core.channels[config.assetHubParaID.into()] = Channel({
            mode: OperatingMode.Normal,
            agent: assetHubAgent,
            inboundNonce: 0,
            outboundNonce: 0
        });

        // Initialize pricing storage
        PricingStorage.Layout storage pricing = PricingStorage.layout();
        pricing.exchangeRate = config.exchangeRate;
        pricing.deliveryCost = config.deliveryCost;
        pricing.multiplier = config.multiplier;

        // Initialize assets storage
        AssetsStorage.Layout storage assets = AssetsStorage.layout();

        assets.assetHubParaID = config.assetHubParaID;
        assets.assetHubAgent = assetHubAgent;
        assets.registerTokenFee = config.registerTokenFee;
        assets.assetHubCreateAssetFee = config.assetHubCreateAssetFee;
        assets.assetHubReserveTransferFee = config.assetHubReserveTransferFee;

        // Initialize operator storage
        OperatorStorage.Layout storage operatorStorage = OperatorStorage.layout();
        operatorStorage.operator = config.rescueOperator;
    }
}

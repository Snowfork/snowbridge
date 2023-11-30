// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.22;

import {MerkleProof} from "openzeppelin/utils/cryptography/MerkleProof.sol";
import {Verification} from "./Verification.sol";

import {Assets} from "./Assets.sol";
import {AgentExecutor} from "./AgentExecutor.sol";
import {Agent} from "./Agent.sol";
import {
    Channel,
    ChannelID,
    InboundMessage,
    OperatingMode,
    ParaID,
    Command,
    MultiAddress,
    Ticket,
    Costs
} from "./Types.sol";
import {IGateway} from "./interfaces/IGateway.sol";
import {IInitializable} from "./interfaces/IInitializable.sol";
import {ERC1967} from "./utils/ERC1967.sol";
import {Address} from "./utils/Address.sol";
import {SafeNativeTransfer} from "./utils/SafeTransfer.sol";
import {Call} from "./utils/Call.sol";
import {Math} from "./utils/Math.sol";
import {ScaleCodec} from "./utils/ScaleCodec.sol";

import {CoreStorage} from "./storage/CoreStorage.sol";
import {PricingStorage} from "./storage/PricingStorage.sol";
import {AssetsStorage} from "./storage/AssetsStorage.sol";

import {UD60x18, ud60x18, convert} from "prb/math/src/UD60x18.sol";

contract Gateway is IGateway, IInitializable {
    using Address for address;
    using SafeNativeTransfer for address payable;

    address internal immutable AGENT_EXECUTOR;

    // Verification state
    address internal immutable BEEFY_CLIENT;

    // BridgeHub
    ParaID internal immutable BRIDGE_HUB_PARA_ID;
    bytes4 internal immutable BRIDGE_HUB_PARA_ID_ENCODED;
    bytes32 internal immutable BRIDGE_HUB_AGENT_ID;

    // AssetHub
    ParaID internal immutable ASSET_HUB_PARA_ID;
    bytes32 internal immutable ASSET_HUB_AGENT_ID;

    // ChannelIDs
    ChannelID internal constant PRIMARY_GOVERNANCE_CHANNEL_ID = ChannelID.wrap(bytes32(uint256(1)));
    ChannelID internal constant SECONDARY_GOVERNANCE_CHANNEL_ID = ChannelID.wrap(bytes32(uint256(2)));

    // Fixed amount of gas used outside the block of code
    // that is measured in submitInbound
    uint256 BASE_GAS_USED = 31_000;

    // Gas used for:
    // 1. Mapping a command id to an implementation function
    // 2. Calling implementation function
    uint256 DISPATCH_OVERHEAD_GAS = 10_000;

    uint8 internal immutable FOREIGN_TOKEN_DECIMALS;

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
    error InvalidCodeHash();
    error InvalidConstructorParams();

    // handler functions are privileged
    modifier onlySelf() {
        if (msg.sender != address(this)) {
            revert Unauthorized();
        }
        _;
    }

    constructor(
        address beefyClient,
        address agentExecutor,
        ParaID bridgeHubParaID,
        bytes32 bridgeHubAgentID,
        ParaID assetHubParaID,
        bytes32 assetHubAgentID,
        uint8 foreignTokenDecimals
    ) {
        if (
            bridgeHubParaID == ParaID.wrap(0) || bridgeHubAgentID == 0 || assetHubParaID == ParaID.wrap(0)
                || assetHubAgentID == 0 || bridgeHubParaID == assetHubParaID || bridgeHubAgentID == assetHubAgentID
        ) {
            revert InvalidConstructorParams();
        }

        BEEFY_CLIENT = beefyClient;
        AGENT_EXECUTOR = agentExecutor;
        BRIDGE_HUB_PARA_ID_ENCODED = ScaleCodec.encodeU32(uint32(ParaID.unwrap(bridgeHubParaID)));
        BRIDGE_HUB_PARA_ID = bridgeHubParaID;
        BRIDGE_HUB_AGENT_ID = bridgeHubAgentID;
        ASSET_HUB_PARA_ID = assetHubParaID;
        ASSET_HUB_AGENT_ID = assetHubAgentID;
        FOREIGN_TOKEN_DECIMALS = foreignTokenDecimals;
    }

    /// @dev Submit a message from Polkadot for verification and dispatch
    /// @param message A message produced by the OutboundQueue pallet on BridgeHub
    /// @param leafProof A message proof used to verify that the message is in the merkle tree committed by the OutboundQueue pallet
    /// @param headerProof A proof that the commitment is included in parachain header that was finalized by BEEFY.
    function submitInbound(
        InboundMessage calldata message,
        bytes32[] calldata leafProof,
        Verification.Proof calldata headerProof
    ) external {
        uint256 startGas = gasleft();

        Channel storage channel = _ensureChannel(message.channelID);

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
        if (!verifyCommitment(commitment, headerProof)) {
            revert InvalidProof();
        }

        // Make sure relayers provide enough gas so that message dispatch
        // does not run out of gas.
        uint256 maxDispatchGas = message.maxDispatchGas;
        if (gasleft() < maxDispatchGas + DISPATCH_OVERHEAD_GAS) {
            revert NotEnoughGas();
        }

        bool success = true;

        // Dispatch message to a handler
        if (message.command == Command.AgentExecute) {
            try Gateway(this).agentExecute{gas: maxDispatchGas}(message.params) {}
            catch {
                success = false;
            }
        } else if (message.command == Command.CreateAgent) {
            try Gateway(this).createAgent{gas: maxDispatchGas}(message.params) {}
            catch {
                success = false;
            }
        } else if (message.command == Command.CreateChannel) {
            try Gateway(this).createChannel{gas: maxDispatchGas}(message.params) {}
            catch {
                success = false;
            }
        } else if (message.command == Command.UpdateChannel) {
            try Gateway(this).updateChannel{gas: maxDispatchGas}(message.params) {}
            catch {
                success = false;
            }
        } else if (message.command == Command.SetOperatingMode) {
            try Gateway(this).setOperatingMode{gas: maxDispatchGas}(message.params) {}
            catch {
                success = false;
            }
        } else if (message.command == Command.TransferNativeFromAgent) {
            try Gateway(this).transferNativeFromAgent{gas: maxDispatchGas}(message.params) {}
            catch {
                success = false;
            }
        } else if (message.command == Command.Upgrade) {
            try Gateway(this).upgrade{gas: maxDispatchGas}(message.params) {}
            catch {
                success = false;
            }
        } else if (message.command == Command.SetTokenTransferFees) {
            try Gateway(this).setTokenTransferFees{gas: maxDispatchGas}(message.params) {}
            catch {
                success = false;
            }
        } else if (message.command == Command.SetPricingParameters) {
            try Gateway(this).setPricingParameters{gas: maxDispatchGas}(message.params) {}
            catch {
                success = false;
            }
        }

        // Calculate the actual cost of executing this message
        uint256 gasUsed = startGas - gasleft() + BASE_GAS_USED;
        uint256 calculatedRefund = gasUsed * tx.gasprice;

        // If the actual refund amount is less than the estimated maximum refund, then
        // reduce the amount paid out accordingly
        uint256 amount = message.maxRefund;
        if (message.maxRefund > calculatedRefund) {
            amount = calculatedRefund;
        }

        // Add the reward to the refund amount. If the sum is more than the funds available
        // in the channel agent, then reduce the total amount
        amount = Math.min(amount + message.reward, address(channel.agent).balance);

        // Do the payment if there funds available in the agent
        if (amount > dustThreshold()) {
            _transferNativeFromAgent(channel.agent, payable(msg.sender), amount);
        }

        emit IGateway.InboundMessageDispatched(message.channelID, message.nonce, message.id, success);
    }

    /**
     * Getters
     */

    function operatingMode() external view returns (OperatingMode) {
        return CoreStorage.layout().mode;
    }

    function channelOperatingModeOf(ChannelID channelID) external view returns (OperatingMode) {
        Channel storage ch = _ensureChannel(channelID);
        return ch.mode;
    }

    function channelNoncesOf(ChannelID channelID) external view returns (uint64, uint64) {
        Channel storage ch = _ensureChannel(channelID);
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

    /**
     * Handlers
     */

    struct AgentExecuteParams {
        bytes32 agentID;
        bytes payload;
    }

    // Execute code within an agent
    function agentExecute(bytes calldata data) external onlySelf {
        AgentExecuteParams memory params = abi.decode(data, (AgentExecuteParams));

        address agent = _ensureAgent(params.agentID);

        if (params.payload.length == 0) {
            revert InvalidAgentExecutionPayload();
        }

        bytes memory call = abi.encodeCall(AgentExecutor.execute, params.payload);

        (bool success, bytes memory returndata) = Agent(payable(agent)).invoke(AGENT_EXECUTOR, call);
        if (!success) {
            revert AgentExecutionFailed(returndata);
        }
    }

    struct CreateAgentParams {
        /// @dev The agent ID of the consensus system
        bytes32 agentID;
    }

    /// @dev Create an agent for a consensus system on Polkadot
    function createAgent(bytes calldata data) external onlySelf {
        CoreStorage.Layout storage $ = CoreStorage.layout();

        CreateAgentParams memory params = abi.decode(data, (CreateAgentParams));

        // Ensure we don't overwrite an existing agent
        if (address($.agents[params.agentID]) != address(0)) {
            revert AgentAlreadyCreated();
        }

        address payable agent = payable(new Agent(params.agentID));
        $.agents[params.agentID] = agent;

        emit AgentCreated(params.agentID, agent);
    }

    struct CreateChannelParams {
        /// @dev The channel ID
        ChannelID channelID;
        /// @dev The agent ID
        bytes32 agentID;
        /// @dev Initial operating mode
        OperatingMode mode;
    }

    /// @dev Create a messaging channel for a Polkadot parachain
    function createChannel(bytes calldata data) external onlySelf {
        CoreStorage.Layout storage $ = CoreStorage.layout();

        CreateChannelParams memory params = abi.decode(data, (CreateChannelParams));

        // Ensure that specified agent actually exists
        address agent = _ensureAgent(params.agentID);

        // Ensure channel has not already been created
        Channel storage ch = $.channels[params.channelID];
        if (address(ch.agent) != address(0)) {
            revert ChannelAlreadyCreated();
        }

        ch.mode = params.mode;
        ch.agent = agent;
        ch.inboundNonce = 0;
        ch.outboundNonce = 0;

        emit ChannelCreated(params.channelID);
    }

    struct UpdateChannelParams {
        /// @dev The parachain used to identify the channel to update
        ChannelID channelID;
        /// @dev The new operating mode
        OperatingMode mode;
    }

    /// @dev Update the configuration for a channel
    function updateChannel(bytes calldata data) external onlySelf {
        UpdateChannelParams memory params = abi.decode(data, (UpdateChannelParams));

        Channel storage ch = _ensureChannel(params.channelID);

        // Extra sanity checks when updating the primary governance channel, which should never be halted.
        if (params.channelID == PRIMARY_GOVERNANCE_CHANNEL_ID && (params.mode != OperatingMode.Normal)) {
            revert InvalidChannelUpdate();
        }

        ch.mode = params.mode;
        emit ChannelUpdated(params.channelID);
    }

    struct UpgradeParams {
        /// @dev The address of the implementation contract
        address impl;
        /// @dev the codehash of the new implementation contract.
        /// Used to ensure the implementation isn't updated while
        /// the upgrade is in flight
        bytes32 implCodeHash;
        /// @dev parameters used to upgrade storage of the gateway
        bytes initParams;
    }

    /// @dev Perform an upgrade of the gateway
    function upgrade(bytes calldata data) external onlySelf {
        UpgradeParams memory params = abi.decode(data, (UpgradeParams));

        // Verify that the implementation is actually a contract
        if (!params.impl.isContract()) {
            revert InvalidCodeHash();
        }

        // As a sanity check, ensure that the codehash of implementation contract
        // matches the codehash in the upgrade proposal
        if (params.impl.codehash != params.implCodeHash) {
            revert InvalidCodeHash();
        }

        // Update the proxy with the address of the new implementation
        ERC1967.store(params.impl);

        // Apply the initialization function of the implementation only if params were provided
        if (params.initParams.length > 0) {
            (bool success, bytes memory returndata) =
                params.impl.delegatecall(abi.encodeCall(IInitializable.initialize, params.initParams));
            Call.verifyResult(success, returndata);
        }

        emit Upgraded(params.impl);
    }

    struct SetOperatingModeParams {
        /// @dev The new operating mode
        OperatingMode mode;
    }

    // @dev Set the operating mode of the gateway
    function setOperatingMode(bytes calldata data) external onlySelf {
        CoreStorage.Layout storage $ = CoreStorage.layout();
        SetOperatingModeParams memory params = abi.decode(data, (SetOperatingModeParams));
        $.mode = params.mode;
        emit OperatingModeChanged(params.mode);
    }

    struct TransferNativeFromAgentParams {
        /// @dev The ID of the agent to transfer funds from
        bytes32 agentID;
        /// @dev The recipient of the funds
        address recipient;
        /// @dev The amount to transfer
        uint256 amount;
    }

    // @dev Transfer funds from an agent to a recipient account
    function transferNativeFromAgent(bytes calldata data) external onlySelf {
        TransferNativeFromAgentParams memory params = abi.decode(data, (TransferNativeFromAgentParams));

        address agent = _ensureAgent(params.agentID);

        _transferNativeFromAgent(agent, payable(params.recipient), params.amount);
        emit AgentFundsWithdrawn(params.agentID, params.recipient, params.amount);
    }

    struct SetTokenTransferFeesParams {
        /// @dev The remote fee (DOT) for registering a token on AssetHub
        uint128 assetHubCreateAssetFee;
        /// @dev The remote fee (DOT) for send tokens to AssetHub
        uint128 assetHubReserveTransferFee;
        /// @dev extra fee to register an asset and discourage spamming (Ether)
        uint256 registerTokenFee;
    }

    // @dev Set token fees of the gateway
    function setTokenTransferFees(bytes calldata data) external onlySelf {
        AssetsStorage.Layout storage $ = AssetsStorage.layout();
        SetTokenTransferFeesParams memory params = abi.decode(data, (SetTokenTransferFeesParams));
        $.assetHubCreateAssetFee = params.assetHubCreateAssetFee;
        $.assetHubReserveTransferFee = params.assetHubReserveTransferFee;
        $.registerTokenFee = params.registerTokenFee;
        emit TokenTransferFeesChanged();
    }

    struct SetPricingParametersParams {
        /// @dev The ETH/DOT exchange rate
        UD60x18 exchangeRate;
        /// @dev The cost of delivering messages to BridgeHub in DOT
        uint128 deliveryCost;
    }

    // @dev Set pricing params of the gateway
    function setPricingParameters(bytes calldata data) external onlySelf {
        PricingStorage.Layout storage pricing = PricingStorage.layout();
        SetPricingParametersParams memory params = abi.decode(data, (SetPricingParametersParams));
        pricing.exchangeRate = params.exchangeRate;
        pricing.deliveryCost = params.deliveryCost;
        emit PricingParametersChanged();
    }

    /**
     * Assets
     */

    // Total fee for registering a token
    function quoteRegisterTokenFee() external view returns (uint256) {
        Costs memory costs = Assets.registerTokenCosts();
        return _calculateFee(costs);
    }

    // Register a token on AssetHub
    function registerToken(address token) external payable {
        Ticket memory ticket = Assets.registerToken(token);
        _submitOutbound(ASSET_HUB_PARA_ID, ticket);
    }

    // Total fee for sending a token
    function quoteSendTokenFee(address, ParaID destinationChain, uint128 destinationFee)
        external
        view
        returns (uint256)
    {
        Costs memory costs = Assets.sendTokenCosts(ASSET_HUB_PARA_ID, destinationChain, destinationFee);
        return _calculateFee(costs);
    }

    // Transfer ERC20 tokens to a Polkadot parachain
    function sendToken(
        address token,
        ParaID destinationChain,
        MultiAddress calldata destinationAddress,
        uint128 destinationFee,
        uint128 amount
    ) external payable {
        CoreStorage.Layout storage $ = CoreStorage.layout();
        address assetHubAgent = $.agents[ASSET_HUB_AGENT_ID];

        Ticket memory ticket = Assets.sendToken(
            ASSET_HUB_PARA_ID,
            assetHubAgent,
            token,
            msg.sender,
            destinationChain,
            destinationAddress,
            destinationFee,
            amount
        );

        _submitOutbound(ASSET_HUB_PARA_ID, ticket);
    }

    /* Internal functions */

    // Verify that a message commitment is considered finalized by our BEEFY light client.
    function verifyCommitment(bytes32 commitment, Verification.Proof calldata proof)
        internal
        view
        virtual
        returns (bool)
    {
        return Verification.verifyCommitment(BEEFY_CLIENT, BRIDGE_HUB_PARA_ID_ENCODED, commitment, proof);
    }

    // Convert foreign currency to native currency (ROC/KSM/DOT -> ETH)
    function _convertToNative(UD60x18 exchangeRate, uint256 amount) internal view returns (uint256) {
        UD60x18 amountFP = convert(amount);
        UD60x18 ethDecimals = convert(1e18);
        UD60x18 foreignDecimals = convert(10).pow(convert(uint256(FOREIGN_TOKEN_DECIMALS)));
        UD60x18 nativeAmountFP = amountFP.mul(exchangeRate).div(foreignDecimals).mul(ethDecimals);
        uint256 nativeAmount = convert(nativeAmountFP);
        return nativeAmount;
    }

    // Calculate the fee for accepting an outbound message
    function _calculateFee(Costs memory costs) internal view returns (uint256) {
        PricingStorage.Layout storage pricing = PricingStorage.layout();
        return costs.native + _convertToNative(pricing.exchangeRate, pricing.deliveryCost + costs.foreign);
    }

    // Submit an outbound message to Polkadot
    function _submitOutbound(ParaID dest, Ticket memory ticket) internal {
        ChannelID channelID = dest.into();
        Channel storage channel = _ensureChannel(channelID);

        // Ensure outbound messaging is allowed
        _ensureOutboundMessagingEnabled(channel);

        uint256 fee = _calculateFee(ticket.costs);

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

        emit IGateway.OutboundMessageAccepted(channelID, channel.outboundNonce, messageID, ticket.payload);
    }

    /// @dev Outbound message can be disabled globally or on a per-channel basis.
    function _ensureOutboundMessagingEnabled(Channel storage ch) internal view {
        CoreStorage.Layout storage $ = CoreStorage.layout();
        if ($.mode != OperatingMode.Normal || ch.mode != OperatingMode.Normal) {
            revert Disabled();
        }
    }

    /// @dev Ensure that the specified parachain has a channel allocated
    function _ensureChannel(ChannelID channelID) internal view returns (Channel storage ch) {
        ch = CoreStorage.layout().channels[channelID];
        // A channel always has an agent specified.
        if (ch.agent == address(0)) {
            revert ChannelDoesNotExist();
        }
    }

    /// @dev Ensure that the specified agentID has a corresponding contract
    function _ensureAgent(bytes32 agentID) internal view returns (address agent) {
        agent = CoreStorage.layout().agents[agentID];
        if (agent == address(0)) {
            revert AgentDoesNotExist();
        }
    }

    /// @dev Invoke some code within an agent
    function _invokeOnAgent(address agent, bytes memory data) internal returns (bytes memory) {
        (bool success, bytes memory returndata) = (Agent(payable(agent)).invoke(AGENT_EXECUTOR, data));
        return Call.verifyResult(success, returndata);
    }

    /// @dev Transfer ether from an agent
    function _transferNativeFromAgent(address agent, address payable recipient, uint256 amount) internal {
        bytes memory call = abi.encodeCall(AgentExecutor.transferNative, (recipient, amount));
        _invokeOnAgent(agent, call);
    }

    /// @dev Define the dust threshold as the minimum cost to transfer ether between accounts
    function dustThreshold() internal view returns (uint256) {
        return 21000 * tx.gasprice;
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
        /// @dev The extra fee charged for registering tokens (DOT)
        uint128 assetHubCreateAssetFee;
        /// @dev The extra fee charged for sending tokens (DOT)
        uint128 assetHubReserveTransferFee;
        /// @dev extra fee to discourage spamming
        uint256 registerTokenFee;
    }

    /// @dev Initialize storage in the gateway
    /// NOTE: This is not externally accessible as this function selector is overshadowed in the proxy
    function initialize(bytes calldata data) external {
        // Prevent initialization of storage in implementation contract
        if (ERC1967.load() == address(0)) {
            revert Unauthorized();
        }

        Config memory config = abi.decode(data, (Config));

        CoreStorage.Layout storage core = CoreStorage.layout();
        core.mode = config.mode;

        // Initialize agent for BridgeHub
        address bridgeHubAgent = address(new Agent(BRIDGE_HUB_AGENT_ID));
        core.agents[BRIDGE_HUB_AGENT_ID] = bridgeHubAgent;

        // Initialize channel for primary governance track
        core.channels[PRIMARY_GOVERNANCE_CHANNEL_ID] =
            Channel({mode: OperatingMode.Normal, agent: bridgeHubAgent, inboundNonce: 0, outboundNonce: 0});

        // Initialize channel for secondary governance track
        core.channels[SECONDARY_GOVERNANCE_CHANNEL_ID] =
            Channel({mode: OperatingMode.Normal, agent: bridgeHubAgent, inboundNonce: 0, outboundNonce: 0});

        // Initialize agent for for AssetHub
        address assetHubAgent = address(new Agent(ASSET_HUB_AGENT_ID));
        core.agents[ASSET_HUB_AGENT_ID] = assetHubAgent;

        // Initialize channel for AssetHub
        core.channels[ASSET_HUB_PARA_ID.into()] =
            Channel({mode: OperatingMode.Normal, agent: assetHubAgent, inboundNonce: 0, outboundNonce: 0});

        // Initialize pricing storage
        PricingStorage.Layout storage pricing = PricingStorage.layout();
        pricing.exchangeRate = config.exchangeRate;
        pricing.deliveryCost = config.deliveryCost;

        // Initialize assets storage
        AssetsStorage.Layout storage assets = AssetsStorage.layout();
        assets.registerTokenFee = config.registerTokenFee;
        assets.assetHubCreateAssetFee = config.assetHubCreateAssetFee;
        assets.assetHubReserveTransferFee = config.assetHubReserveTransferFee;
    }
}

// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.25;

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

import {
    UpgradeParams,
    CreateAgentParams,
    AgentExecuteParams,
    CreateChannelParams,
    UpdateChannelParams,
    SetOperatingModeParams,
    TransferNativeFromAgentParams,
    SetTokenTransferFeesParams,
    SetPricingParametersParams
} from "./Params.sol";

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

    // ChannelIDs
    ChannelID internal constant PRIMARY_GOVERNANCE_CHANNEL_ID = ChannelID.wrap(bytes32(uint256(1)));
    ChannelID internal constant SECONDARY_GOVERNANCE_CHANNEL_ID = ChannelID.wrap(bytes32(uint256(2)));

    // Gas used for:
    // 1. Mapping a command id to an implementation function
    // 2. Calling implementation function
    uint256 DISPATCH_OVERHEAD_GAS = 10_000;

    // The maximum fee that can be sent to a destination parachain to pay for execution (DOT).
    // Has two functions:
    // * Reduces the ability of users to perform arbitrage using a favourable exchange rate
    // * Prevents users from mistakenly providing too much fees, which would drain AssetHub's
    //   sovereign account here on Ethereum.
    uint128 internal immutable MAX_DESTINATION_FEE;

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
    error InvalidConstructorParams();

    // Message handlers can only be dispatched by the gateway itself
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
        uint8 foreignTokenDecimals,
        uint128 maxDestinationFee
    ) {
        if (bridgeHubParaID == ParaID.wrap(0) || bridgeHubAgentID == 0) {
            revert InvalidConstructorParams();
        }

        BEEFY_CLIENT = beefyClient;
        AGENT_EXECUTOR = agentExecutor;
        BRIDGE_HUB_PARA_ID_ENCODED = ScaleCodec.encodeU32(uint32(ParaID.unwrap(bridgeHubParaID)));
        BRIDGE_HUB_PARA_ID = bridgeHubParaID;
        BRIDGE_HUB_AGENT_ID = bridgeHubAgentID;
        FOREIGN_TOKEN_DECIMALS = foreignTokenDecimals;
        MAX_DESTINATION_FEE = maxDestinationFee;
    }

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
        if (!_verifyCommitment(commitment, headerProof)) {
            revert InvalidProof();
        }

        bool success = dispatch(message.command, message.params, maxDispatchGas);
        uint256 amount = calculateTotalReward(startGas, message.maxFeePerGas, message.reward);
        if (amount > _dustThreshold()) {
            _transferNativeFromAgent(channel.agent, recipient, amount);
        }

        emit IGateway.InboundMessageDispatched(message.channelID, message.nonce, message.id, success);
    }

    function dispatchV1(CommandV1 command, bytes calldata params, uint256 maxDispatchGas) internal returns (bool) {
        // Make sure relayers provide enough gas so that inner message dispatch
        // does not run out of gas.
        uint256 maxDispatchGas = maxDispatchGas;
        if (gasleft() < maxDispatchGas + DISPATCH_OVERHEAD_GAS) {
            revert NotEnoughGas();
        }

        // Dispatch message to a handler
        if (command == Command.AgentExecute) {
            try Gateway(this).unlockToken{gas: maxDispatchGas}(params, true) {}
            catch {
                return false;
            }
        } else if (command == Command.CreateAgent) {
            try Gateway(this).createAgent{gas: maxDispatchGas}(params) {}
            catch {
                return false;
            }
        } else if (command == Command.SetOperatingMode) {
            try Gateway(this).setOperatingMode{gas: maxDispatchGas}(params) {}
            catch {
                return false;
            }
        } else if (command == Command.Upgrade) {
            try Gateway(this).upgrade{gas: maxDispatchGas}(message.params) {}
            catch {
                return false;
            }
        } else if (command == Command.SetTokenTransferFees) {
            try Gateway(this).setTokenTransferFees{gas: maxDispatchGas}(params) {}
            catch {
                return false;
            }
        } else if (command == Command.SetPricingParameters) {
            try Gateway(this).setPricingParameters{gas: maxDispatchGas}(params) {}
            catch {
                return false;
            }
        } else {
            return false;
        }

        return true;
    }

    /// @dev Submit a message from Polkadot for verification and dispatch
    /// @param message A message produced by the OutboundQueue pallet on BridgeHub
    /// @param leafProof A message proof used to verify that the message is in the merkle tree committed by the OutboundQueue pallet
    /// @param headerProof A proof that the commitment is included in parachain header that was finalized by BEEFY.
    function submitV2(
        InboundMessageV2 calldata message,
        bytes32[] calldata leafProof,
        Verification.Proof calldata headerProof
    ) external {
        uint256 startGas = gasleft();

        CoreStorage.Layout storage $ = CoreStorage.layout();

        bytes32 messageID = keccak256(abi.encode(message));

        if ($.messageHashes[messageID] == true) {
            emit InvalidNonce();
        }

        $.messageHashes[messageID] = true;

        // Produce the commitment (message root) by applying the leaf proof to the message leaf
        bytes32 commitment = MerkleProof.processProof(leafProof, messageID);

        // Verify that the commitment is included in a parachain header finalized by BEEFY.
        if (!_verifyCommitment(commitment, headerProof)) {
            revert InvalidProof();
        }

        bool success = dispatchV2(message.command, maxDispatchGas);
        uint256 amount = calculateTotalReward(startGas, message.maxFeePerGas, message.reward);
        if (amount > _dustThreshold()) {
            payable(msg.sender).safeNativeTransfer(amount);
        }

        emit IGateway.InboundMessageDispatched(messageID, success);
    }

    function dispatchV2(bytes calldata command, uint256 maxDispatchGas) internal returns (bool) {
        // Make sure relayers provide enough gas so that inner message dispatch
        // does not run out of gas.
        if (gasleft() < maxDispatchGas + DISPATCH_OVERHEAD_GAS) {
            revert NotEnoughGas();
        }

        uint8 cmdCode = uint8(command[31]);

        if (cmdCode == CommandV2.Upgrade) {
            try Gateway(this).upgrade{gas: maxDispatchGas}(message.params) {}
            catch {
                return false;
            }
        } else if (cmdCode == CommandV2.SetOperatingMode) {
            try Gateway(this).setOperatingMode{gas: maxDispatchGas}(params) {}
            catch {
                return false;
            }
        } else if (cmdCode == CommandV2.SetPricingParameters) {
            try Gateway(this).setPricingParameters{gas: maxDispatchGas}(params) {}
            catch {
                return false;
            }
        } else if (cmdCode == CommandV2.CreateAgent) {
            try Gateway(this).createAgent{gas: maxDispatchGas}(params) {}
            catch {
                return false;
            }
        } else if (cmdCode == CommandV2.SetTokenTransferFees) {
            try Gateway(this).setTokenTransferFees{gas: maxDispatchGas}(params) {}
            catch {
                return false;
            }
        } else if (cmdCode == CommandV2.UnlockToken) {
            try Gateway(this).unlockToken{gas: maxDispatchGas}(params, false) {}
            catch {
                return false;
            }
        }

        return true;
    }

    function calculateTotalReward(address gasmanager, uint256 startGas, uint256 maxFeePerGas, uint256 reward)
        internal
        returns (uint256)
    {
        // Calculate a gas refund, capped to protect against huge spikes in `tx.gasprice`
        // that could drain funds unnecessarily. During these spikes, relayers should back off.
        uint256 gasUsed = _transactionBaseGas() + (startGas - gasleft());
        uint256 refund = gasUsed * Math.min(tx.gasprice, maxFeePerGas);

        // Add the reward to the refund amount. If the sum is more than the funds available
        // in the gasmanager contract, then reduce the total amount
        return Math.min(refund + reward, address(rewardsAccount).balance);
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

    /// @dev Perform an upgrade of the gateway
    function upgrade(bytes calldata data) external onlySelf {
        UpgradeParams memory params = abi.decode(data, (UpgradeParams));
        Upgrade.upgrade(params.impl, params.implCodeHash, params.initParams);
    }

    // @dev Set the operating mode of the gateway
    function setOperatingMode(bytes calldata data) external onlySelf {
        CoreStorage.Layout storage $ = CoreStorage.layout();
        SetOperatingModeParams memory params = abi.decode(data, (SetOperatingModeParams));
        $.mode = params.mode;
        emit OperatingModeChanged(params.mode);
    }

    function unlockToken(bytes calldata data, bool isLegacyDataFormat) external onlySelf {
        address agent;
        address token;
        address recipient;
        uint128 amount;

        if (isLegacyDataFormat) {
            AgentExecuteParams memory params = abi.decode(data, (AgentExecuteParams));
            (AgentExecuteCommand command, bytes memory innerParams) = abi.decode(data, (AgentExecuteCommand, bytes));
            if (command == AgentExecuteCommand.TransferToken) {
                (token, recipient, amount) = abi.decode(params, (address, address, uint128));
                agent = _ensureAgent(params.agentID);
            } else {
                return;
            }
        } else {
            UnlockTokenParams memory params = abi.decode(data, (UnlockTokenParams));
            agent = _ensureAgent(params.agentID);
            token = params.token;
            recipient = params.recipient;
            amount = params.amount;
        }

        bytes memory call = abi.encodeCall(AgentExecutor.transferToken, (token, recipient, amount));
        _invokeOnAgent(agent, call);
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

    // @dev Set pricing params of the gateway
    function setPricingParameters(bytes calldata data) external onlySelf {
        PricingStorage.Layout storage pricing = PricingStorage.layout();
        SetPricingParametersParams memory params = abi.decode(data, (SetPricingParametersParams));
        pricing.exchangeRate = params.exchangeRate;
        pricing.deliveryCost = params.deliveryCost;
        pricing.multiplier = params.multiplier;
        emit PricingParametersChanged();
    }

    /**
     * Assets
     */
    function isTokenRegistered(address token) external view returns (bool) {
        return Assets.isTokenRegistered(token);
    }

    // Total fee for registering a token
    function quoteRegisterTokenFee() external view returns (uint256) {
        return _calculateFee(Assets.registerTokenCosts());
    }

    // Register an Ethereum-native token in the gateway and on AssetHub
    function registerToken(address token) external payable {
        _submitOutbound(Assets.registerToken(token));
    }

    // Total fee for sending a token
    function quoteSendTokenFee(address token, ParaID destinationChain, uint128 destinationFee)
        external
        view
        returns (uint256)
    {
        return _calculateFee(Assets.sendTokenCosts(token, destinationChain, destinationFee, MAX_DESTINATION_FEE));
    }

    // Transfer ERC20 tokens to a Polkadot parachain
    function sendToken(
        address token,
        ParaID destinationChain,
        MultiAddress calldata destinationAddress,
        uint128 destinationFee,
        uint128 amount
    ) external payable {
        _submitOutbound(
            Assets.sendToken(
                token, msg.sender, destinationChain, destinationAddress, destinationFee, MAX_DESTINATION_FEE, amount
            )
        );
    }

    /**
     * Internal functions
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
    function _transactionBaseGas() internal pure returns (uint256) {
        return 21_000 + 14_698 + (msg.data.length * 16);
    }

    // Verify that a message commitment is considered finalized by our BEEFY light client.
    function _verifyCommitment(bytes32 commitment, Verification.Proof calldata proof)
        internal
        view
        virtual
        returns (bool)
    {
        return Verification.verifyCommitment(BEEFY_CLIENT, BRIDGE_HUB_PARA_ID_ENCODED, commitment, proof);
    }

    // Convert foreign currency to native currency (ROC/KSM/DOT -> ETH)
    function _convertToNative(UD60x18 exchangeRate, UD60x18 multiplier, UD60x18 amount)
        internal
        view
        returns (uint256)
    {
        UD60x18 ethDecimals = convert(1e18);
        UD60x18 foreignDecimals = convert(10).pow(convert(uint256(FOREIGN_TOKEN_DECIMALS)));
        UD60x18 nativeAmount = multiplier.mul(amount).mul(exchangeRate).div(foreignDecimals).mul(ethDecimals);
        return convert(nativeAmount);
    }

    // Calculate the fee for accepting an outbound message
    function _calculateFee(Costs memory costs) internal view returns (uint256) {
        PricingStorage.Layout storage pricing = PricingStorage.layout();
        UD60x18 amount = convert(pricing.deliveryCost + costs.foreign);
        return costs.native + _convertToNative(pricing.exchangeRate, pricing.multiplier, amount);
    }

    // Submit an outbound message to Polkadot, after taking fees
    function _submitOutbound(Ticket memory ticket) internal {
        CoreStorage.Layout storage $ = CoreStorage.layout();

        if ($.mode != OperatingMode.Normal) {
            revert Disabled();
        }

        uint256 fee = _calculateFee(ticket.costs);

        // Ensure the user has enough funds for this message to be accepted
        if (msg.value < fee) {
            revert FeePaymentToLow();
        }

        $.nonces[ticket.destination] = $.nonces[ticket.destination] + 1;

        // Reimburse excess fee payment
        if (msg.value > fee) {
            payable(msg.sender).safeNativeTransfer(msg.value - fee);
        }

        bytes memory message = abi.encodePacked(
            $.nonces[ticket.destination], ticket.sender, ticket.destination, ticket.recipient, ticket.payload
        );
        bytes32 messageHash = keccak256(message);

        emit IGateway.OutboundMessageAccepted(messageHash, message);
    }

    /// @dev Ensure that the specified parachain has a channel allocated [DEPRECATED]
    function _ensureChannel(ChannelID channelID) internal view returns (Channel storage ch) {
        ch = CoreStorage.layout().channels[channelID];
        // A channel always has an agent specified.
        if (ch.agent == address(0)) {
            revert ChannelDoesNotExist();
        }
    }

    /// @dev Ensure that the specified agentID has a corresponding contract [DEPRECATED]
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

    /// @dev Define the dust threshold as the minimum cost to transfer ether between accounts
    function _dustThreshold() internal view returns (uint256) {
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

        // Initialize channel for primary governance track
        core.channels[PRIMARY_GOVERNANCE_CHANNEL_ID] =
            Channel({mode: OperatingMode.Normal, agent: bridgeHubAgent, inboundNonce: 0, outboundNonce: 0});

        // Initialize channel for secondary governance track
        core.channels[SECONDARY_GOVERNANCE_CHANNEL_ID] =
            Channel({mode: OperatingMode.Normal, agent: bridgeHubAgent, inboundNonce: 0, outboundNonce: 0});

        // Initialize agent for for AssetHub
        address assetHubAgent = address(new Agent(config.assetHubAgentID));
        core.agents[config.assetHubAgentID] = assetHubAgent;

        // Initialize channel for AssetHub
        core.channels[config.assetHubParaID.into()] =
            Channel({mode: OperatingMode.Normal, agent: assetHubAgent, inboundNonce: 0, outboundNonce: 0});

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

    /// @dev Temporary rescue ability for the initial bootstrapping phase of the bridge
    function rescue(address impl, bytes32 implCodeHash, bytes calldata initializerParams) external {
        OperatorStorage.Layout storage operatorStorage = OperatorStorage.layout();
        if (msg.sender != operatorStorage.operator) {
            revert Unauthorized();
        }
        Upgrade.upgrade(impl, implCodeHash, initializerParams);
    }

    function dropRescueAbility() external {
        OperatorStorage.Layout storage operatorStorage = OperatorStorage.layout();
        if (msg.sender != operatorStorage.operator) {
            revert Unauthorized();
        }
        operatorStorage.operator = address(0);
    }

    function rescueOperator() external view returns (address) {
        OperatorStorage.Layout storage operatorStorage = OperatorStorage.layout();
        return operatorStorage.operator;
    }
}

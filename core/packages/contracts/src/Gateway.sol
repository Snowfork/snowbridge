// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.20;

import {MerkleProof} from "openzeppelin/utils/cryptography/MerkleProof.sol";
import {Verification} from "./Verification.sol";

import {Features} from "./Features.sol";
import {AgentExecutor} from "./AgentExecutor.sol";
import {Agent} from "./Agent.sol";
import {Channel, InboundMessage, OperatingMode, ParaID} from "./Types.sol";
import {IGateway} from "./IGateway.sol";

import {CoreStorage} from "./storage/CoreStorage.sol";
import {FeaturesStorage} from "./storage/FeaturesStorage.sol";
import {VerificationStorage} from "./storage/VerificationStorage.sol";

import {console} from "forge-std/console.sol";

import {Initializable} from "./Initializable.sol";
import {UUPSUpgradeable} from "openzeppelin/proxy/utils/UUPSUpgradeable.sol";
import {Address} from "openzeppelin/utils/Address.sol";

contract Gateway is IGateway, Initializable, UUPSUpgradeable {
    using Address for address;

    // Inbound messages correspond to these commands
    bytes32 internal constant COMMAND_AGENT_EXECUTE = keccak256("agentExecute");
    bytes32 internal constant COMMAND_CREATE_AGENT = keccak256("createAgent");
    bytes32 internal constant COMMAND_CREATE_CHANNEL = keccak256("createChannel");
    bytes32 internal constant COMMAND_UPDATE_CHANNEL = keccak256("updateChannel");
    bytes32 internal constant COMMAND_UPGRADE = keccak256("upgrade");
    bytes32 internal constant COMMAND_SET_OPERATING_MODE = keccak256("setOperatingMode");
    bytes32 internal constant COMMAND_WITHDRAW_SOVEREIGN_FUNDS = keccak256("withdrawSovereignFunds");
    bytes32 internal constant COMMAND_CONFIGURE = keccak256("configure");

    // After message dispatch, there should be some gas left over for post dispatch logic
    uint256 internal constant GAS_BUFFER = 24000;

    // Ensure that operators don't brick the bridge by reconfiguring `gasToForward` to 0 by mistake
    uint256 internal constant MIN_GAS_TO_FORWARD = 500_000;

    error InvalidProof();
    error InvalidNonce();
    error NotEnoughGas();
    error FeePaymentToLow();
    error FailedPayment();
    error Unauthorized();
    error UnknownChannel();
    error Disabled();
    error AgentAlreadyCreated();
    error AgentDoesNotExist();
    error ChannelAlreadyCreated();
    error ChannelDoesNotExist();
    error InvalidChannelUpdate();
    error WithdrawalFailed();
    error AgentExecutionFailed();
    error InvalidAgentExecutionPayload();
    error InvalidConfig();

    struct InitParams {
        // Agent Executor
        address agentExecutor;
        // default fee & Reward parameters
        uint256 fee;
        uint256 reward;
        // BridgeHub
        ParaID bridgeHubParaID;
        bytes32 bridgeHubAgentID;
        // AssetHub
        ParaID assetHubParaID;
        bytes32 assetHubAgentID;
        // Gas to forward to message handlers
        uint256 gasToForward;
        Features.InitParams features;
        Verification.InitParams verification;
    }

    // handler functions are privileged
    modifier onlySelf() {
        if (msg.sender != address(this)) {
            revert Unauthorized();
        }
        _;
    }

    constructor() {
        _disableInitializers();
    }

    function initialize(InitParams calldata params) external initializer {
        CoreStorage.Layout storage $ = CoreStorage.layout();

        if (!params.agentExecutor.isContract() || params.gasToForward < MIN_GAS_TO_FORWARD) {
            revert InvalidConfig();
        }

        $.mode = OperatingMode.Normal;
        $.agentExecutor = params.agentExecutor;
        $.defaultFee = params.fee;
        $.defaultReward = params.reward;
        $.gasToForward = params.gasToForward;
        $.bridgeHubParaID = params.bridgeHubParaID;

        // Initialize an agent & channel for BridgeHub
        Agent bridgeHubAgent = new Agent(params.bridgeHubAgentID);
        $.agents[params.bridgeHubAgentID] = payable(bridgeHubAgent);
        $.channels[params.bridgeHubParaID] = Channel({
            mode: OperatingMode.Normal,
            agent: address(bridgeHubAgent),
            inboundNonce: 0,
            outboundNonce: 0,
            fee: params.fee,
            reward: params.reward
        });

        // Initialize an agent & channel for AssetHub
        Agent assetHubAgent = new Agent(params.assetHubAgentID);
        $.agents[params.assetHubAgentID] = payable(assetHubAgent);
        $.channels[params.assetHubParaID] = Channel({
            mode: OperatingMode.Normal,
            agent: address(assetHubAgent),
            inboundNonce: 0,
            outboundNonce: 0,
            fee: params.fee,
            reward: params.reward
        });

        Features.initialize(params.features, address(assetHubAgent));
        Verification.initialize(params.verification);
    }

    function submitInbound(
        InboundMessage calldata message,
        bytes32[] calldata leafProof,
        Verification.Proof calldata headerProof
    ) external onlyProxy {
        CoreStorage.Layout storage $ = CoreStorage.layout();
        Channel storage channel = ensureChannel(message.origin);

        bytes32 leafHash = keccak256(abi.encode(message));
        bytes32 commitment = MerkleProof.processProof(leafProof, leafHash);

        if (!Verification.verifyCommitment(commitment, headerProof)) {
            revert InvalidProof();
        }

        if (message.nonce != channel.inboundNonce + 1) {
            revert InvalidNonce();
        }

        // Increment nonce for origin.
        channel.inboundNonce++;

        // reward the relayer from the agent contract
        // Should revert if there are not enough funds. In which case, the origin
        // should top up the funds and have a relayer resend the message.
        if (channel.reward > 0) {
            Agent(payable(channel.agent)).withdrawTo(payable(msg.sender), channel.reward);
        }

        // Ensure relayers pass enough gas for message to execute.
        // Otherwise malicious relayers can break the bridge by allowing handlers to run out gas.
        // Resubmission of the message by honest relayers will fail as the tracked nonce
        // has already been updated.
        if (gasleft() < $.gasToForward + GAS_BUFFER) {
            revert NotEnoughGas();
        }

        bool success = true;

        if (message.command == COMMAND_AGENT_EXECUTE) {
            try Gateway(this).handleAgentExecute{gas: $.gasToForward}(message.params) {}
            catch {
                success = false;
            }
        } else if (message.command == COMMAND_CREATE_AGENT) {
            try Gateway(this).handleCreateAgent{gas: $.gasToForward}(message.params) {}
            catch {
                success = false;
            }
        } else if (message.command == COMMAND_CREATE_CHANNEL) {
            try Gateway(this).handleCreateChannel{gas: $.gasToForward}(message.params) {}
            catch {
                success = false;
            }
        } else if (message.command == COMMAND_UPDATE_CHANNEL) {
            try Gateway(this).handleUpdateChannel{gas: $.gasToForward}(message.params) {}
            catch {
                success = false;
            }
        } else if (message.command == COMMAND_SET_OPERATING_MODE) {
            try Gateway(this).handleSetOperatingMode{gas: $.gasToForward}(message.params) {}
            catch {
                success = false;
            }
        } else if (message.command == COMMAND_WITHDRAW_SOVEREIGN_FUNDS) {
            try Gateway(this).handleWithdrawSovereignFunds{gas: $.gasToForward}(message.params) {}
            catch {
                success = false;
            }
        } else if (message.command == COMMAND_UPGRADE) {
            try Gateway(this).handleUpgrade{gas: $.gasToForward}(message.params) {}
            catch {
                success = false;
            }
        }

        emit IGateway.InboundMessageDispatched(message.origin, message.nonce, success);
    }

    /**
     * Getters
     */

    function operatingMode() external view returns (OperatingMode) {
        return CoreStorage.layout().mode;
    }

    function channelOperatingModeOf(ParaID paraID) external view returns (OperatingMode) {
        CoreStorage.Layout storage $ = CoreStorage.layout();
        return $.channels[paraID].mode;
    }

    function channelNoncesOf(ParaID paraID) external view returns (uint64, uint64) {
        Channel storage ch = CoreStorage.layout().channels[paraID];
        return (ch.inboundNonce, ch.outboundNonce);
    }

    function channelFeeRewardOf(ParaID paraID) external view returns (uint256, uint256) {
        Channel storage ch = CoreStorage.layout().channels[paraID];
        return (ch.fee, ch.reward);
    }

    function agentOf(bytes32 agentID) external view returns (address) {
        CoreStorage.Layout storage $ = CoreStorage.layout();
        return $.agents[agentID];
    }

    /**
     * Handlers
     */

    // Execute code within an agent
    function handleAgentExecute(bytes calldata params) external onlySelf {
        CoreStorage.Layout storage $ = CoreStorage.layout();

        (bytes32 originID, bytes memory payload) = abi.decode(params, (bytes32, bytes));

        address agent = $.agents[originID];
        if (agent == address(0)) {
            revert AgentDoesNotExist();
        }

        if (payload.length == 0) {
            revert InvalidAgentExecutionPayload();
        }

        bytes memory call = abi.encodeCall(AgentExecutor.execute, (address(this), payload));
        try Agent(payable(agent)).invoke($.agentExecutor, call) {}
        catch {
            revert AgentExecutionFailed();
        }
    }

    // Create an agent for a consensus system on Polkadot
    function handleCreateAgent(bytes calldata params) external onlySelf {
        CoreStorage.Layout storage $ = CoreStorage.layout();

        (bytes32 agentID) = abi.decode(params, (bytes32));
        if (address($.agents[agentID]) != address(0)) {
            revert AgentAlreadyCreated();
        }
        address payable agent = payable(new Agent(agentID));

        $.agents[agentID] = agent;
        emit AgentCreated(agentID, agent);
    }

    // Create a messaging channel to a Polkadot parachain
    function handleCreateChannel(bytes calldata params) external onlySelf {
        CoreStorage.Layout storage $ = CoreStorage.layout();

        (ParaID paraID, bytes32 agentID) = abi.decode(params, (ParaID, bytes32));

        // Ensure that specified agent actually exists
        address agent = $.agents[agentID];
        if (agent == address(0)) {
            revert AgentDoesNotExist();
        }

        // Ensure channel has not already been created
        Channel storage ch = $.channels[paraID];
        if (address(ch.agent) != address(0)) {
            revert ChannelAlreadyCreated();
        }

        ch.mode = OperatingMode.Normal;
        ch.agent = $.agents[agentID];
        ch.inboundNonce = 0;
        ch.outboundNonce = 0;
        ch.fee = $.defaultFee;
        ch.reward = $.defaultReward;

        emit ChannelCreated(paraID);
    }

    // Update parameters for a channel
    function handleUpdateChannel(bytes calldata params) external onlySelf {
        CoreStorage.Layout storage $ = CoreStorage.layout();

        (ParaID paraID, OperatingMode mode, uint256 fee, uint256 reward) =
            abi.decode(params, (ParaID, OperatingMode, uint256, uint256));

        Channel storage ch = ensureChannel(paraID);

        // Extra sanity checks when updating the BridgeHub channel. For example, a huge reward could
        // effectively brick the bridge.
        if (paraID == $.bridgeHubParaID && (fee > 1 ether || reward > 1 ether)) {
            revert InvalidChannelUpdate();
        }

        ch.mode = mode;
        ch.fee = fee;
        ch.reward = reward;

        emit ChannelUpdated(paraID);
    }

    // Perform an upgrade
    function handleUpgrade(bytes calldata params) external onlySelf {
        (address logic, bytes memory data) = abi.decode(params, (address, bytes));
        _upgradeToAndCall(logic, data, false);
    }

    // Set the operating mode
    function handleSetOperatingMode(bytes calldata params) external onlySelf {
        CoreStorage.Layout storage $ = CoreStorage.layout();
        (OperatingMode mode) = abi.decode(params, (OperatingMode));
        $.mode = mode;
    }

    // Withdraw funds from an agent to a recipient account
    function handleWithdrawSovereignFunds(bytes calldata params) external onlySelf {
        (ParaID paraID, address payable recipient, uint256 amount) = abi.decode(params, (ParaID, address, uint256));
        Channel storage ch = ensureChannel(paraID);
        Agent(payable(ch.agent)).withdrawTo(recipient, amount);
    }

    // Reconfigure a selection of parameters.
    //
    // Only allow reconfiguring mostly non-critical parameters to prevent future maintainers/operators
    // from bricking the bridge due to careless operational mistakes.
    //
    // More complicated reconfiguration should be done via an upgrade that has been audited.
    //
    function handleConfigure(bytes calldata params) external onlySelf {
        CoreStorage.Layout storage $ = CoreStorage.layout();
        (address agentExecutor, uint256 defaultFee, uint256 defaultReward, uint256 gasToForward, uint256 createTokenFee)
        = abi.decode(params, (address, uint256, uint256, uint256, uint256));

        // sanity checks
        if (!agentExecutor.isContract() || gasToForward < MIN_GAS_TO_FORWARD || defaultFee == 0 || defaultReward == 0) {
            revert InvalidConfig();
        }

        $.agentExecutor = agentExecutor;
        $.defaultFee = defaultFee;
        $.defaultReward = defaultReward;
        $.gasToForward = gasToForward;

        FeaturesStorage.layout().createTokenFee = createTokenFee;
    }

    /**
     * Features (Token transfers, etc)
     */

    // Transfer ERC20 tokens to a Polkadot parachain
    function lockNativeTokens(address token, ParaID finalDestPara, bytes calldata recipient, uint128 amount)
        external
        payable
        onlyProxy
    {
        (ParaID dest, bytes memory payload) =
            Features.lockNativeToken(token, msg.sender, finalDestPara, recipient, amount);
        submitOutbound(dest, payload);
    }

    // Register a token on AssetHub
    function registerNativeToken(address token) external payable onlyProxy {
        (ParaID dest, bytes memory payload) = Features.registerNativeToken(token);
        submitOutbound(dest, payload);
    }

    /* Internal functions */

    function submitOutbound(ParaID dest, bytes memory payload) internal {
        Channel storage channel = ensureChannel(dest);
        ensureOutboundMessagingEnabled(channel);

        if (msg.value < channel.fee) {
            revert FeePaymentToLow();
        }

        channel.outboundNonce = channel.outboundNonce + 1;

        // Deposit fee into agent's contract
        (bool success,) = address(channel.agent).call{value: msg.value}("");
        if (!success) {
            revert FailedPayment();
        }

        emit IGateway.OutboundMessageAccepted(dest, channel.outboundNonce, payload);
    }

    function ensureOutboundMessagingEnabled(Channel storage ch) internal view {
        CoreStorage.Layout storage $ = CoreStorage.layout();
        if ($.mode != OperatingMode.Normal || ch.mode != OperatingMode.Normal) {
            revert Disabled();
        }
    }

    // transfer funds from an agent to a recipient
    function withdrawTo(address agent, address payable recipient, uint256 amount) internal {
        CoreStorage.Layout storage $ = CoreStorage.layout();
        bytes memory data = abi.encodeCall(AgentExecutor.withdrawTo, (recipient, amount));
        try Agent(payable(agent)).invoke($.agentExecutor, data) {}
        catch {
            revert WithdrawalFailed();
        }
    }

    function ensureChannel(ParaID paraID) internal view returns (Channel storage ch) {
        ch = CoreStorage.layout().channels[paraID];
        // A channel always has an agent specified.
        if (ch.agent == address(0)) {
            revert ChannelDoesNotExist();
        }
    }

    function _authorizeUpgrade(address) internal view override {
        if (msg.sender != address(this)) {
            revert Unauthorized();
        }
    }
}

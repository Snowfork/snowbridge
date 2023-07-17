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
import {ERC1967} from "./utils/ERC1967.sol";
import {Address} from "./utils/Address.sol";

import {CoreStorage} from "./storage/CoreStorage.sol";
import {FeaturesStorage} from "./storage/FeaturesStorage.sol";
import {VerificationStorage} from "./storage/VerificationStorage.sol";

contract Gateway is IGateway {
    using Address for address;

    // Inbound messages correspond to these commands
    bytes32 internal constant COMMAND_AGENT_EXECUTE = keccak256("agentExecute");
    bytes32 internal constant COMMAND_CREATE_AGENT = keccak256("createAgent");
    bytes32 internal constant COMMAND_CREATE_CHANNEL = keccak256("createChannel");
    bytes32 internal constant COMMAND_UPDATE_CHANNEL = keccak256("updateChannel");
    bytes32 internal constant COMMAND_UPGRADE = keccak256("upgrade");
    bytes32 internal constant COMMAND_SET_OPERATING_MODE = keccak256("setOperatingMode");
    bytes32 internal constant COMMAND_WITHDRAW_AGENT_FUNDS = keccak256("withdrawAgentFunds");
    bytes32 internal constant COMMAND_CONFIGURE = keccak256("configure");

    // After message dispatch, there should be some gas left over for post dispatch logic
    uint256 internal constant GAS_BUFFER = 24000;

    uint256 internal constant GAS_FOR_DISPATCH = 500_000;

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
    error SetupFailed();
    error NotProxy();
    error InvalidCodeHash();

    // handler functions are privileged
    modifier onlySelf() {
        if (msg.sender != address(this)) {
            revert Unauthorized();
        }
        _;
    }

    function submitInbound(
        InboundMessage calldata message,
        bytes32[] calldata leafProof,
        Verification.Proof calldata headerProof
    ) external {
        Channel storage channel = _ensureChannel(message.origin);

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
        if (gasleft() < GAS_FOR_DISPATCH + GAS_BUFFER) {
            revert NotEnoughGas();
        }

        bool success = true;

        if (message.command == COMMAND_AGENT_EXECUTE) {
            try Gateway(this).agentExecute{gas: GAS_FOR_DISPATCH}(message.params) {}
            catch {
                success = false;
            }
        } else if (message.command == COMMAND_CREATE_AGENT) {
            try Gateway(this).createAgent{gas: GAS_FOR_DISPATCH}(message.params) {}
            catch {
                success = false;
            }
        } else if (message.command == COMMAND_CREATE_CHANNEL) {
            try Gateway(this).createChannel{gas: GAS_FOR_DISPATCH}(message.params) {}
            catch {
                success = false;
            }
        } else if (message.command == COMMAND_UPDATE_CHANNEL) {
            try Gateway(this).updateChannel{gas: GAS_FOR_DISPATCH}(message.params) {}
            catch {
                success = false;
            }
        } else if (message.command == COMMAND_SET_OPERATING_MODE) {
            try Gateway(this).setOperatingMode{gas: GAS_FOR_DISPATCH}(message.params) {}
            catch {
                success = false;
            }
        } else if (message.command == COMMAND_WITHDRAW_AGENT_FUNDS) {
            try Gateway(this).withdrawAgentFunds{gas: GAS_FOR_DISPATCH}(message.params) {}
            catch {
                success = false;
            }
        } else if (message.command == COMMAND_UPGRADE) {
            try Gateway(this).upgrade{gas: GAS_FOR_DISPATCH}(message.params) {}
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

    struct AgentExecuteParams {
        bytes32 agentID;
        bytes payload;
    }

    // Execute code within an agent
    function agentExecute(bytes calldata data) external onlySelf {
        CoreStorage.Layout storage $ = CoreStorage.layout();

        AgentExecuteParams memory params = abi.decode(data, (AgentExecuteParams));

        address agent = $.agents[params.agentID];
        if (agent == address(0)) {
            revert AgentDoesNotExist();
        }

        if (params.payload.length == 0) {
            revert InvalidAgentExecutionPayload();
        }

        bytes memory call = abi.encodeCall(AgentExecutor.execute, (address(this), params.payload));
        try Agent(payable(agent)).invoke($.agentExecutor, call) {}
        catch {
            revert AgentExecutionFailed();
        }
    }

    struct CreateAgentParams {
        bytes32 agentID;
    }

    // Create an agent for a consensus system on Polkadot
    function createAgent(bytes calldata data) external onlySelf {
        CoreStorage.Layout storage $ = CoreStorage.layout();

        CreateAgentParams memory params = abi.decode(data, (CreateAgentParams));

        if (address($.agents[params.agentID]) != address(0)) {
            revert AgentAlreadyCreated();
        }
        address payable agent = payable(new Agent(params.agentID));

        $.agents[params.agentID] = agent;
        emit AgentCreated(params.agentID, agent);
    }

    struct CreateChannelParams {
        ParaID paraID;
        bytes32 agentID;
    }

    // Create a messaging channel to a Polkadot parachain
    function createChannel(bytes calldata data) external onlySelf {
        CoreStorage.Layout storage $ = CoreStorage.layout();

        CreateChannelParams memory params = abi.decode(data, (CreateChannelParams));

        // Ensure that specified agent actually exists
        address agent = $.agents[params.agentID];
        if (agent == address(0)) {
            revert AgentDoesNotExist();
        }

        // Ensure channel has not already been created
        Channel storage ch = $.channels[params.paraID];
        if (address(ch.agent) != address(0)) {
            revert ChannelAlreadyCreated();
        }

        ch.mode = OperatingMode.Normal;
        ch.agent = $.agents[params.agentID];
        ch.inboundNonce = 0;
        ch.outboundNonce = 0;
        ch.fee = $.defaultFee;
        ch.reward = $.defaultReward;

        emit ChannelCreated(params.paraID);
    }

    struct UpdateChannelParams {
        ParaID paraID;
        OperatingMode mode;
        uint256 fee;
        uint256 reward;
    }

    // Update parameters for a channel
    function updateChannel(bytes calldata data) external onlySelf {
        CoreStorage.Layout storage $ = CoreStorage.layout();

        UpdateChannelParams memory params = abi.decode(data, (UpdateChannelParams));

        Channel storage ch = _ensureChannel(params.paraID);

        // Extra sanity checks when updating the BridgeHub channel. For example, a huge reward could
        // effectively brick the bridge permanently.
        if (
            params.paraID == $.bridgeHubParaID
                && (params.mode != OperatingMode.Normal || params.fee > 1 ether || params.reward > 1 ether)
        ) {
            revert InvalidChannelUpdate();
        }

        ch.mode = params.mode;
        ch.fee = params.fee;
        ch.reward = params.reward;

        emit ChannelUpdated(params.paraID);
    }

    struct UpgradeParams {
        address impl;
        bytes32 implCodeHash;
        bytes initParams;
    }

    // Perform an upgrade
    function upgrade(bytes calldata data) external onlySelf {
        UpgradeParams memory params = abi.decode(data, (UpgradeParams));
        if (params.impl.isContract() && params.impl.codehash != params.implCodeHash) {
            revert InvalidCodeHash();
        }
        ERC1967.store(params.impl);
        if (params.initParams.length > 0) {
            (bool success,) =
                params.impl.delegatecall(abi.encodeWithSelector(this.initialize.selector, params.initParams));
            if (!success) {
                revert SetupFailed();
            }
        }
        emit Upgraded(params.impl);
    }

    struct SetOperatingModeParams {
        OperatingMode mode;
    }

    // Set the operating mode
    function setOperatingMode(bytes calldata data) external onlySelf {
        CoreStorage.Layout storage $ = CoreStorage.layout();
        SetOperatingModeParams memory params = abi.decode(data, (SetOperatingModeParams));
        $.mode = params.mode;
    }

    struct WithdrawAgentFundsParams {
        bytes32 agentID;
        address recipient;
        uint256 amount;
    }

    // Withdraw funds from an agent to a recipient account
    function withdrawAgentFunds(bytes calldata data) external onlySelf {
        CoreStorage.Layout storage $ = CoreStorage.layout();

        WithdrawAgentFundsParams memory params = abi.decode(data, (WithdrawAgentFundsParams));

        address agent = $.agents[params.agentID];
        if (agent == address(0)) {
            revert AgentDoesNotExist();
        }

        Agent(payable(agent)).withdrawTo(payable(params.recipient), params.amount);
    }

    /**
     * Features (Token transfers, etc)
     */

    // Transfer ERC20 tokens to a Polkadot parachain
    function lockNativeTokens(address token, ParaID finalDestPara, bytes calldata recipient, uint128 amount)
        external
        payable
    {
        (ParaID dest, bytes memory payload) =
            Features.lockNativeToken(token, msg.sender, finalDestPara, recipient, amount);
        _submitOutbound(dest, payload);
    }

    // Register a token on AssetHub
    function registerNativeToken(address token) external payable {
        (ParaID dest, bytes memory payload) = Features.registerNativeToken(token);
        _submitOutbound(dest, payload);
    }

    /* Internal functions */

    function _submitOutbound(ParaID dest, bytes memory payload) internal {
        Channel storage channel = _ensureChannel(dest);
        _ensureOutboundMessagingEnabled(channel);

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

    function _ensureOutboundMessagingEnabled(Channel storage ch) internal view {
        CoreStorage.Layout storage $ = CoreStorage.layout();
        if ($.mode != OperatingMode.Normal || ch.mode != OperatingMode.Normal) {
            revert Disabled();
        }
    }

    // transfer funds from an agent to a recipient
    function _withdrawTo(address agent, address payable recipient, uint256 amount) internal {
        CoreStorage.Layout storage $ = CoreStorage.layout();
        bytes memory data = abi.encodeCall(AgentExecutor.withdrawTo, (recipient, amount));
        try Agent(payable(agent)).invoke($.agentExecutor, data) {}
        catch {
            revert WithdrawalFailed();
        }
    }

    function _ensureChannel(ParaID paraID) internal view returns (Channel storage ch) {
        ch = CoreStorage.layout().channels[paraID];
        // A channel always has an agent specified.
        if (ch.agent == address(0)) {
            revert ChannelDoesNotExist();
        }
    }

    /**
     * Upgrades
     */

    /// @dev Not publicly accessible as overshadowed in the proxy
    function initialize(bytes memory data) external {
        // Prevent initialization of storage in implementation contract
        if (ERC1967.load() == address(0)) {
            revert Unauthorized();
        }

        InitParams memory params = abi.decode(data, (InitParams));

        CoreStorage.Layout storage $ = CoreStorage.layout();

        $.mode = OperatingMode.Normal;
        $.agentExecutor = params.agentExecutor;
        $.defaultFee = params.fee;
        $.defaultReward = params.reward;
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

    function implementation() public view returns (address) {
        return ERC1967.load();
    }
}

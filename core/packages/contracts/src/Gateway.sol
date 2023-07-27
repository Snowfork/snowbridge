// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.20;

import {MerkleProof} from "openzeppelin/utils/cryptography/MerkleProof.sol";
import {Verification} from "./Verification.sol";

import {Assets} from "./Assets.sol";
import {AgentExecutor} from "./AgentExecutor.sol";
import {Agent} from "./Agent.sol";
import {Channel, InboundMessage, OperatingMode, ParaID, Config, Command} from "./Types.sol";
import {IGateway} from "./interfaces/IGateway.sol";
import {ERC1967} from "./utils/ERC1967.sol";
import {Address} from "./utils/Address.sol";
import {SafeNativeTransfer} from "./utils/SafeTransfer.sol";
import {Call} from "./utils/Call.sol";
import {ScaleCodec} from "./utils/ScaleCodec.sol";

import {CoreStorage} from "./storage/CoreStorage.sol";
import {AssetsStorage} from "./storage/AssetsStorage.sol";

contract Gateway is IGateway {
    using Address for address;
    using SafeNativeTransfer for address payable;

    // After message dispatch, there should be some gas left over for post dispatch logic
    uint256 internal constant BUFFER_GAS = 32_000;
    uint256 internal immutable DISPATCH_GAS;
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
    bytes2 internal immutable CREATE_TOKEN_CALL_ID;

    error InvalidProof();
    error InvalidNonce();
    error InvalidAgentExecutor();
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
    error AgentExecutionFailed(bytes returndata);
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

    constructor(
        address beefyClient,
        address agentExecutor,
        uint256 dispatchGas,
        ParaID bridgeHubParaID,
        bytes32 bridgeHubHubAgentID,
        ParaID assetHubParaID,
        bytes32 assetHubHubAgentID,
        bytes2 createTokenCallID
    ) {
        BEEFY_CLIENT = beefyClient;
        AGENT_EXECUTOR = agentExecutor;
        DISPATCH_GAS = dispatchGas;
        BRIDGE_HUB_PARA_ID_ENCODED = ScaleCodec.encodeU32(uint32(ParaID.unwrap(bridgeHubParaID)));
        BRIDGE_HUB_PARA_ID = bridgeHubParaID;
        BRIDGE_HUB_AGENT_ID = bridgeHubHubAgentID;
        ASSET_HUB_PARA_ID = assetHubParaID;
        ASSET_HUB_AGENT_ID = assetHubHubAgentID;
        CREATE_TOKEN_CALL_ID = createTokenCallID;
    }

    function submitInbound(
        InboundMessage calldata message,
        bytes32[] calldata leafProof,
        Verification.Proof calldata headerProof
    ) external {
        Channel storage channel = _ensureChannel(message.origin);

        bytes32 leafHash = keccak256(abi.encode(message));
        bytes32 commitment = MerkleProof.processProof(leafProof, leafHash);

        if (!verifyCommitment(commitment, headerProof)) {
            revert InvalidProof();
        }

        if (message.nonce != channel.inboundNonce + 1) {
            revert InvalidNonce();
        }

        // Increment nonce for origin. Together with the above nonce check, this should prevent reentrancy.
        channel.inboundNonce++;

        // reward the relayer from the agent contract
        // Should revert if there are not enough funds. In which case, the origin
        // should top up the funds and have a relayer resend the message.
        if (channel.reward > 0) {
            _transferNativeFromAgent(channel.agent, payable(msg.sender), channel.reward);
        }

        // Ensure relayers pass enough gas for message to execute.
        // Otherwise malicious relayers can break the bridge by allowing handlers to run out gas.
        // Resubmission of the message by honest relayers will fail as the tracked nonce
        // has already been updated.
        if (gasleft() < DISPATCH_GAS + BUFFER_GAS) {
            revert NotEnoughGas();
        }

        bool success = true;

        if (message.command == Command.AgentExecute) {
            try Gateway(this).agentExecute{gas: DISPATCH_GAS}(message.params) {}
            catch {
                success = false;
            }
        } else if (message.command == Command.CreateAgent) {
            try Gateway(this).createAgent{gas: DISPATCH_GAS}(message.params) {}
            catch {
                success = false;
            }
        } else if (message.command == Command.CreateChannel) {
            try Gateway(this).createChannel{gas: DISPATCH_GAS}(message.params) {}
            catch {
                success = false;
            }
        } else if (message.command == Command.UpdateChannel) {
            try Gateway(this).updateChannel{gas: DISPATCH_GAS}(message.params) {}
            catch {
                success = false;
            }
        } else if (message.command == Command.SetOperatingMode) {
            try Gateway(this).setOperatingMode{gas: DISPATCH_GAS}(message.params) {}
            catch {
                success = false;
            }
        } else if (message.command == Command.TransferNativeFromAgent) {
            try Gateway(this).transferNativeFromAgent{gas: DISPATCH_GAS}(message.params) {}
            catch {
                success = false;
            }
        } else if (message.command == Command.Upgrade) {
            try Gateway(this).upgrade{gas: DISPATCH_GAS}(message.params) {}
            catch {
                success = false;
            }
        }

        emit IGateway.InboundMessageDispatched(message.origin, message.nonce, success);
    }

    function verifyCommitment(bytes32 commitment, Verification.Proof calldata proof) internal view returns (bool) {
        if (BEEFY_CLIENT != address(0)) {
            return Verification.verifyCommitment(BEEFY_CLIENT, BRIDGE_HUB_PARA_ID_ENCODED, commitment, proof);
        } else {
            // for unit tests, verification is bypassed
            return true;
        }
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

        (bool success, bytes memory returndata) = Agent(payable(agent)).invoke(AGENT_EXECUTOR, call);
        if (!success) {
            revert AgentExecutionFailed(returndata);
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
        UpdateChannelParams memory params = abi.decode(data, (UpdateChannelParams));

        Channel storage ch = _ensureChannel(params.paraID);

        // Extra sanity checks when updating the BridgeHub channel. For example, a huge reward could
        // effectively brick the bridge permanently.
        if (
            params.paraID == BRIDGE_HUB_PARA_ID
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

    struct TransferNativeFromAgentParams {
        bytes32 agentID;
        address recipient;
        uint256 amount;
    }

    // Withdraw funds from an agent to a recipient account
    function transferNativeFromAgent(bytes calldata data) external onlySelf {
        CoreStorage.Layout storage $ = CoreStorage.layout();

        TransferNativeFromAgentParams memory params = abi.decode(data, (TransferNativeFromAgentParams));

        address agent = $.agents[params.agentID];
        if (agent == address(0)) {
            revert AgentDoesNotExist();
        }

        _transferNativeFromAgent(agent, payable(params.recipient), params.amount);
    }

    /**
     * Assets
     */

    // Register a token on AssetHub
    function registerToken(address token) external payable {
        (bytes memory payload, uint256 extraFee) = Assets.registerToken(token, CREATE_TOKEN_CALL_ID);

        _submitOutbound(ASSET_HUB_PARA_ID, payload, extraFee);
    }

    // Transfer ERC20 tokens to a Polkadot parachain
    function sendToken(address token, ParaID destinationChain, bytes32 destinationAddress, uint128 amount)
        external
        payable
    {
        CoreStorage.Layout storage $ = CoreStorage.layout();
        address assetHubAgent = $.agents[ASSET_HUB_AGENT_ID];

        (bytes memory payload, uint256 extraFee) = Assets.sendToken(
            ASSET_HUB_PARA_ID,
            assetHubAgent,
            token,
            msg.sender,
            destinationChain,
            destinationAddress,
            amount
        );

        _submitOutbound(ASSET_HUB_PARA_ID, payload, extraFee);
    }

    // Transfer ERC20 tokens to a Polkadot parachain
    function sendToken(address token, ParaID destinationChain, address destinationAddress, uint128 amount)
        external
        payable
    {
        CoreStorage.Layout storage $ = CoreStorage.layout();
        address assetHubAgent = $.agents[ASSET_HUB_AGENT_ID];

        (bytes memory payload, uint256 extraFee) = Assets.sendToken(
            ASSET_HUB_PARA_ID,
            assetHubAgent,
            token,
            msg.sender,
            destinationChain,
            destinationAddress,
            amount
        );

        _submitOutbound(ASSET_HUB_PARA_ID, payload, extraFee);
    }

    /* Internal functions */

    function _submitOutbound(ParaID dest, bytes memory payload, uint256 extraFee) internal {
        Channel storage channel = _ensureChannel(dest);
        _ensureOutboundMessagingEnabled(channel);

        if (msg.value < channel.fee + extraFee) {
            revert FeePaymentToLow();
        }

        channel.outboundNonce = channel.outboundNonce + 1;

        // Deposit total fee into agent's contract
        payable(channel.agent).safeNativeTransfer(msg.value);

        emit IGateway.OutboundMessageAccepted(dest, channel.outboundNonce, payload);
    }

    function _ensureOutboundMessagingEnabled(Channel storage ch) internal view {
        CoreStorage.Layout storage $ = CoreStorage.layout();
        if ($.mode != OperatingMode.Normal || ch.mode != OperatingMode.Normal) {
            revert Disabled();
        }
    }

    function _ensureChannel(ParaID paraID) internal view returns (Channel storage ch) {
        ch = CoreStorage.layout().channels[paraID];
        // A channel always has an agent specified.
        if (ch.agent == address(0)) {
            revert ChannelDoesNotExist();
        }
    }

    function _invokeOnAgent(address agent, bytes memory data) internal returns (bytes memory) {
        (bool success, bytes memory returndata) = (Agent(payable(agent)).invoke(AGENT_EXECUTOR, data));
        return Call.verifyResult(success, returndata);
    }

    function _transferNativeFromAgent(address agent, address payable recipient, uint256 amount) internal {
        bytes memory call = abi.encodeCall(AgentExecutor.transferNative, (recipient, amount));
        _invokeOnAgent(agent, call);
    }

    /**
     * Upgrades
     */

    /// @dev Not externally accessible as overshadowed in the proxy
    function initialize(bytes memory data) external {
        // Prevent initialization of storage in implementation contract
        if (ERC1967.load() == address(0)) {
            revert Unauthorized();
        }

        (uint256 defaultFee, uint256 defaultReward, uint256 registerTokenFee, uint256 sendTokenFee) =
            abi.decode(data, (uint256, uint256, uint256, uint256));

        CoreStorage.Layout storage $ = CoreStorage.layout();

        $.mode = OperatingMode.Normal;
        $.defaultFee = defaultFee;
        $.defaultReward = defaultReward;

        // Initialize an agent & channel for BridgeHub
        address bridgeHubAgent = address(new Agent(BRIDGE_HUB_AGENT_ID));
        $.agents[BRIDGE_HUB_AGENT_ID] = bridgeHubAgent;
        $.channels[BRIDGE_HUB_PARA_ID] = Channel({
            mode: OperatingMode.Normal,
            agent: bridgeHubAgent,
            inboundNonce: 0,
            outboundNonce: 0,
            fee: defaultFee,
            reward: defaultReward
        });

        // Initialize an agent & channel for AssetHub
        address assetHubAgent = address(new Agent(ASSET_HUB_AGENT_ID));
        $.agents[ASSET_HUB_AGENT_ID] = assetHubAgent;
        $.channels[ASSET_HUB_PARA_ID] = Channel({
            mode: OperatingMode.Normal,
            agent: assetHubAgent,
            inboundNonce: 0,
            outboundNonce: 0,
            fee: defaultFee,
            reward: defaultReward
        });

        Assets.initialize(registerTokenFee, sendTokenFee);
    }
}

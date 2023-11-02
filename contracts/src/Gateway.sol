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
import {IInitializable} from "./interfaces/IInitializable.sol";
import {ERC1967} from "./utils/ERC1967.sol";
import {Address} from "./utils/Address.sol";
import {SafeNativeTransfer} from "./utils/SafeTransfer.sol";
import {Call} from "./utils/Call.sol";
import {Math} from "./utils/Math.sol";
import {ScaleCodec} from "./utils/ScaleCodec.sol";

import {CoreStorage} from "./storage/CoreStorage.sol";
import {AssetsStorage} from "./storage/AssetsStorage.sol";

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

    // Fixed amount of gas used outside the block of code
    // that is measured in submitInbound
    uint256 BASE_GAS_USED = 31_000;

    // Gas used for:
    // 1. Mapping a command id to an implementation function
    // 2. Calling implementation function
    uint256 DISPATCH_OVERHEAD_GAS = 10_000;

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
        bytes32 assetHubAgentID
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
    }

    /// @dev Submit a message from Polkadot for verification and dispatch
    /// @param message A message produced by the OutboundQueue pallet on BridgeHub
    /// @param leafProof A message proof used to verify that the message is in the merkle tree committed by the OutboundQueue pallet
    /// @param headerProof A proof used to verify that the commitment was included in a BridgeHub header that was finalized by BEEFY.
    function submitInbound(
        InboundMessage calldata message,
        bytes32[] calldata leafProof,
        Verification.Proof calldata headerProof
    ) external {
        uint256 startGas = gasleft();

        Channel storage channel = _ensureChannel(message.origin);

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

        emit IGateway.InboundMessageDispatched(message.origin, message.nonce, success);
    }

    /**
     * Getters
     */

    function operatingMode() external view returns (OperatingMode) {
        return CoreStorage.layout().mode;
    }

    function channelOperatingModeOf(ParaID paraID) external view returns (OperatingMode) {
        Channel storage ch = _ensureChannel(paraID);
        return ch.mode;
    }

    function channelNoncesOf(ParaID paraID) external view returns (uint64, uint64) {
        Channel storage ch = _ensureChannel(paraID);
        return (ch.inboundNonce, ch.outboundNonce);
    }

    function channelFeeOf(ParaID paraID) external view returns (uint256) {
        Channel storage ch = _ensureChannel(paraID);
        return ch.fee;
    }

    function agentOf(bytes32 agentID) external view returns (address) {
        address agentAddress = _ensureAgent(agentID);
        return agentAddress;
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

        bytes memory call = abi.encodeCall(AgentExecutor.execute, (address(this), params.payload));

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
        /// @dev The parachain which will own the newly created channel
        ParaID paraID;
        /// @dev The agent ID of the parachain
        bytes32 agentID;
    }

    /// @dev Create a messaging channel for a Polkadot parachain
    function createChannel(bytes calldata data) external onlySelf {
        CoreStorage.Layout storage $ = CoreStorage.layout();

        CreateChannelParams memory params = abi.decode(data, (CreateChannelParams));

        // Ensure that specified agent actually exists
        address agent = _ensureAgent(params.agentID);

        // Ensure channel has not already been created
        Channel storage ch = $.channels[params.paraID];
        if (address(ch.agent) != address(0)) {
            revert ChannelAlreadyCreated();
        }

        ch.mode = OperatingMode.Normal;
        ch.agent = agent;
        ch.inboundNonce = 0;
        ch.outboundNonce = 0;
        ch.fee = $.defaultFee;

        emit ChannelCreated(params.paraID);
    }

    struct UpdateChannelParams {
        /// @dev The parachain used to identify the channel to update
        ParaID paraID;
        /// @dev The new operating mode
        OperatingMode mode;
        /// @dev The new fee for accepting outbound messages
        uint256 fee;
        /// @dev The new reward to be given to relayers for submitting inbound messages
        uint256 reward;
    }

    /// @dev Update the configuration for a channel
    function updateChannel(bytes calldata data) external onlySelf {
        UpdateChannelParams memory params = abi.decode(data, (UpdateChannelParams));

        Channel storage ch = _ensureChannel(params.paraID);

        // Extra sanity checks when updating the BridgeHub channel, which should never be paused.
        if (params.paraID == BRIDGE_HUB_PARA_ID && (params.mode != OperatingMode.Normal || params.fee > 1 ether)) {
            revert InvalidChannelUpdate();
        }

        ch.mode = params.mode;
        ch.fee = params.fee;

        emit ChannelUpdated(params.paraID);
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

        // Verify that the code in the implementation contract was not changed
        // after the upgrade initiated on BridgeHub parachain.
        if (params.impl.codehash != params.implCodeHash) {
            revert InvalidCodeHash();
        }

        // Update the proxy with the address of the new implementation
        ERC1967.store(params.impl);

        // Apply the initialization function of the implementation only if params were provided
        if (params.initParams.length > 0) {
            (bool success,) =
                params.impl.delegatecall(abi.encodeWithSelector(this.initialize.selector, params.initParams));
            if (!success) {
                revert InitializationFailed();
            }
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

    /**
     * Assets
     */

    // Register a token on AssetHub
    function registerToken(address token) external payable {
        (bytes memory payload, uint256 extraFee) = Assets.registerToken(token);

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
            ASSET_HUB_PARA_ID, assetHubAgent, token, msg.sender, destinationChain, destinationAddress, amount
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
            ASSET_HUB_PARA_ID, assetHubAgent, token, msg.sender, destinationChain, destinationAddress, amount
        );

        _submitOutbound(ASSET_HUB_PARA_ID, payload, extraFee);
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

    // Submit an outbound message to Polkadot
    function _submitOutbound(ParaID dest, bytes memory payload, uint256 extraFee) internal {
        Channel storage channel = _ensureChannel(dest);

        // Ensure outbound messaging is allowed
        _ensureOutboundMessagingEnabled(channel);

        // Ensure the user has enough funds for this message to be accepted
        if (msg.value < channel.fee + extraFee) {
            revert FeePaymentToLow();
        }

        channel.outboundNonce = channel.outboundNonce + 1;

        // Deposit total fee into agent's contract
        payable(channel.agent).safeNativeTransfer(channel.fee + extraFee);

        // Reimburse excess fee payment
        if (msg.value > channel.fee + extraFee) {
            payable(msg.sender).safeNativeTransfer(msg.value - channel.fee - extraFee);
        }

        emit IGateway.OutboundMessageAccepted(dest, channel.outboundNonce, payload);
    }

    /// @dev Outbound message can be disabled globally or on a per-channel basis.
    function _ensureOutboundMessagingEnabled(Channel storage ch) internal view {
        CoreStorage.Layout storage $ = CoreStorage.layout();
        if ($.mode != OperatingMode.Normal || ch.mode != OperatingMode.Normal) {
            revert Disabled();
        }
    }

    /// @dev Ensure that the specified parachain has a channel allocated
    function _ensureChannel(ParaID paraID) internal view returns (Channel storage ch) {
        ch = CoreStorage.layout().channels[paraID];
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

    /// @dev Initialize storage in the gateway
    /// NOTE: This is not externally accessible as this function selector is overshadowed in the proxy
    function initialize(bytes memory data) external {
        // Prevent initialization of storage in implementation contract
        if (ERC1967.load() == address(0)) {
            revert Unauthorized();
        }

        (uint256 defaultFee, uint256 registerTokenFee, uint256 sendTokenFee) =
            abi.decode(data, (uint256, uint256, uint256));

        CoreStorage.Layout storage $ = CoreStorage.layout();

        $.mode = OperatingMode.RejectingOutboundMessages;
        $.defaultFee = defaultFee;

        // Initialize an agent & channel for BridgeHub
        address bridgeHubAgent = address(new Agent(BRIDGE_HUB_AGENT_ID));
        $.agents[BRIDGE_HUB_AGENT_ID] = bridgeHubAgent;
        $.channels[BRIDGE_HUB_PARA_ID] = Channel({
            mode: OperatingMode.Normal,
            agent: bridgeHubAgent,
            inboundNonce: 0,
            outboundNonce: 0,
            fee: defaultFee
        });

        // Initialize an agent & channel for AssetHub
        address assetHubAgent = address(new Agent(ASSET_HUB_AGENT_ID));
        $.agents[ASSET_HUB_AGENT_ID] = assetHubAgent;
        $.channels[ASSET_HUB_PARA_ID] = Channel({
            mode: OperatingMode.Normal,
            agent: assetHubAgent,
            inboundNonce: 0,
            outboundNonce: 0,
            fee: defaultFee
        });

        Assets.initialize(registerTokenFee, sendTokenFee);
    }
}

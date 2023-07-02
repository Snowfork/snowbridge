// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.20;

import {MerkleProof} from "openzeppelin/utils/cryptography/MerkleProof.sol";
import {AccessControl} from "openzeppelin/access/AccessControl.sol";
import {IParachainClient, ParachainClient} from "./ParachainClient.sol";
import {Registry} from "./Registry.sol";
import {RegistryLookup} from "./RegistryLookup.sol";
import {Auth} from "./Auth.sol";
import {Vault} from "./Vault.sol";

import {IRecipient} from "./IRecipient.sol";
import {IExecutor} from "./IExecutor.sol";
import {Withdraw} from "./Withdraw.sol";

import {Agent} from "./Agent.sol";
import {ParaID} from "./Types.sol";

contract Gateway2 {
    mapping(ParaID paraID => Channel) public channels;

    struct Channel {
        Agent agent;
        uint64 inboundNonce;
        uint64 outboundNonce;
    }

    // Light client message verifier
    IParachainClient public parachainClient;

    // All agents
    mapping(bytes32 agentID => Agent) public agents;

    Withdraw public withdrawContract;

    address public executor;

    // The relayer reward for submitting a message
    uint256 public reward;

    // Relayers must provide enough gas to cover message dispatch plus a buffer
    uint256 public gasToForward = 500000;
    uint256 public constant GAS_BUFFER = 24000;

    bytes32 internal constant COMMAND_EXECUTE_XCM = keccak256("execute");
    bytes32 internal constant COMMAND_CREATE_SOVEREIGN = keccak256("createSovereign");
    bytes32 internal constant COMMAND_UPGRADE = keccak256("upgrade");

    // Inbound message from BridgeHub parachain
    struct InboundMessage {
        ParaID origin;
        uint64 nonce;
        bytes32 command;
        bytes params;
    }

    event InboundMessageDispatched(bytes32 lane, uint64 nonce, bool success);
    event HandlerUpdated(uint16 id, IRecipient handler);
    event ParachainClientUpdated(address parachainClient);
    event VaultUpdated(address vault);
    event RewardUpdated(uint256 reward);
    event GasToForwardUpdated(uint256 gasToForward);
    event InvalidRecipient(bytes32 recipient);

    error InvalidProof();
    error InvalidNonce();
    error NotEnoughGas();
    error NotSelf();

    constructor(IParachainClient _parachainClient, uint256 _reward) {
        parachainClient = _parachainClient;
        reward = _reward;
    }

    function submitInbound(InboundMessage calldata message, bytes32[] calldata leafProof, bytes calldata headerProof)
        external
    {
        bytes32 leafHash = keccak256(abi.encode(message));
        bytes32 commitment = MerkleProof.processProof(leafProof, leafHash);
        if (!parachainClient.verifyCommitment(commitment, headerProof)) {
            revert InvalidProof();
        }

        Channel storage channel = channels[message.channelID];

        if (message.nonce != channel.inboundNonce + 1) {
            revert InvalidNonce();
        }

        // Increment nonce for origin.
        channel.inboundNonce++;

        // reward the relayer
        // Should revert if there are not enough funds. In which case, the origin
        // should top up the funds and have a relayer resend the message.
        channel.agent.invoke(
            address(withdrawContract), abi.encodeCall(Withdraw.withdraw, (payable(msg.sender), reward))
        );

        // Ensure relayers pass enough gas for message to execute.
        // Otherwise malicious relayers can break the bridge by allowing handlers to run out gas.
        // Resubmission of the message by honest relayers will fail as the tracked nonce
        // has already been updated.
        if (gasleft() < gasToForward + GAS_BUFFER) {
            revert NotEnoughGas();
        }

        bool success = true;

        if (message.command == COMMAND_EXECUTE_XCM) {
            try Gateway2(this).handleExecuteXCM{gas: gasToForward}(message.params) {}
            catch {
                success = false;
            }
        } else if (message.command == COMMAND_CREATE_SOVEREIGN) {
            try Gateway2(this).handleCreateAgent{gas: gasToForward}(message.params) {}
            catch {
                success = false;
            }
        }

        emit InboundMessageDispatched(message.channelID, message.nonce, success);
    }

    function submitOutbound(ParaID dest, bytes calldata payload) external payable {
        if (msg.value < fee) {
            revert FeePaymentToLow();
        }
        nonce[dest] = nonce[dest] + 1;
        vault.deposit{value: msg.value}(dest);
        emit Message(dest, nonce[dest], payload);
    }

    modifier onlySelf() {
        if (msg.sender != address(this)) revert NotSelf();
        _;
    }

    function handleExecuteXCM(bytes calldata params) external onlySelf {
        (bytes32 agentID, bytes memory payload) = abi.decode(params, (bytes32, bytes));
        agents[agentID].invoke(executor, payload);
    }

    function handleCreateAgent(bytes calldata params) external onlySelf {
        (bytes32 agentID) = abi.decode(params, (bytes32));
        agents[agentID] = new Agent();
    }

    function handleCreateChannel(bytes calldata params) external onlySelf {
        (ParaID paraID, bytes32 agentID) = abi.decode(params, (ParaID, bytes32));
        channels[paraID] = new Channel({
            agent: agents[agentID],
            inboundNonce: 0,
            outboundNonce: 0
        });
    }
}

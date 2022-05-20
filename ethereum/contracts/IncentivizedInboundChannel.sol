// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "@openzeppelin/contracts/access/AccessControl.sol";
import "./ParachainClient.sol";
import "./RewardSource.sol";

contract IncentivizedInboundChannel is AccessControl {
    uint64 public nonce;

    struct MessageBundle {
        uint64 nonce;
        Message[] messages;
    }

    struct Message {
        uint64 id;
        address target;
        uint128 fee;
        bytes payload;
    }

    event MessageDispatched(uint64 id, bool result);

    uint256 public constant MAX_GAS_PER_MESSAGE = 100000;
    uint256 public constant GAS_BUFFER = 60000;

    // Governance contracts will administer using this role.
    bytes32 public constant CONFIG_UPDATE_ROLE = keccak256("CONFIG_UPDATE_ROLE");

    RewardSource private rewardSource;

    ParachainClient public parachainClient;

    constructor(ParachainClient client) {
        _setupRole(DEFAULT_ADMIN_ROLE, msg.sender);
        parachainClient = client;
        nonce = 0;
    }

    // Once-off post-construction call to set initial configuration.
    function initialize(address _configUpdater, address _rewardSource)
        external
        onlyRole(DEFAULT_ADMIN_ROLE)
    {
        // Set initial configuration
        grantRole(CONFIG_UPDATE_ROLE, _configUpdater);
        rewardSource = RewardSource(_rewardSource);

        // drop admin privileges
        renounceRole(DEFAULT_ADMIN_ROLE, msg.sender);
    }

    function submit(MessageBundle calldata bundle, bytes calldata proof) external {
        // Proof
        // 1. Compute our parachain's message `commitment` by ABI encoding and hashing the `_messages`
        bytes32 commitment = keccak256(abi.encode(bundle));

        require(parachainClient.verifyCommitment(commitment, proof), "Invalid proof");

        // Require there is enough gas to play all messages
        require(
            gasleft() >= (bundle.messages.length * MAX_GAS_PER_MESSAGE) + GAS_BUFFER,
            "insufficient gas for delivery of all messages"
        );

        processMessages(payable(msg.sender), bundle);
    }

    function processMessages(address payable _relayer, MessageBundle calldata bundle) internal {
        require(bundle.nonce == nonce + 1, "invalid nonce");

        uint128 _rewardAmount = 0;
        for (uint256 i = 0; i < bundle.messages.length; i++) {
            Message calldata message = bundle.messages[i];

            // Deliver the message to the target
            // Delivery will have fixed maximum gas allowed for the target app
            (bool success, ) = message.target.call{ value: 0, gas: MAX_GAS_PER_MESSAGE }(
                message.payload
            );

            _rewardAmount = _rewardAmount + message.fee;
            emit MessageDispatched(message.id, success);
        }

        // reward the relayer
        rewardSource.reward(_relayer, _rewardAmount);
        nonce++;
    }
}

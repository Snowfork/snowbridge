// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "@openzeppelin/contracts/access/AccessControl.sol";
import "./ParachainClient.sol";
import "./RewardController.sol";

contract IncentivizedInboundChannel is AccessControl {
    uint8 public immutable sourceChannelID;
    uint64 public nonce;

    struct MessageBundle {
        uint8 sourceChannelID;
        uint64 nonce;
        uint128 fee;
        Message[] messages;
    }

    struct Message {
        uint64 id;
        address target;
        bytes payload;
    }

    event MessageDispatched(uint64 id, bool result);

    uint256 public constant MAX_GAS_PER_MESSAGE = 100000;
    uint256 public constant GAS_BUFFER = 60000;

    // Governance contracts will administer using this role.
    bytes32 public constant CONFIG_UPDATE_ROLE = keccak256("CONFIG_UPDATE_ROLE");

    RewardController private rewardController;

    ParachainClient public parachainClient;

    constructor(uint8 _sourceChannelID, ParachainClient _parachainClient) {
        _setupRole(DEFAULT_ADMIN_ROLE, msg.sender);
        nonce = 0;
        sourceChannelID = _sourceChannelID;
        parachainClient = _parachainClient;
    }

    // Once-off post-construction call to set initial configuration.
    function initialize(address _configUpdater, address _rewardController)
        external
        onlyRole(DEFAULT_ADMIN_ROLE)
    {
        // Set initial configuration
        grantRole(CONFIG_UPDATE_ROLE, _configUpdater);
        rewardController = RewardController(_rewardController);

        // drop admin privileges
        renounceRole(DEFAULT_ADMIN_ROLE, msg.sender);
    }

    function submit(MessageBundle calldata bundle, bytes calldata proof) external {
        bytes32 commitment = keccak256(abi.encode(bundle));

        require(parachainClient.verifyCommitment(commitment, proof), "Invalid proof");
        require(bundle.sourceChannelID == sourceChannelID, "Invalid source channel");
        require(bundle.nonce == nonce + 1, "Invalid nonce");
        require(
            gasleft() >= (bundle.messages.length * MAX_GAS_PER_MESSAGE) + GAS_BUFFER,
            "insufficient gas for delivery of all messages"
        );
        nonce++;
        dispatch(bundle);
        rewardController.handleReward(payable(msg.sender), bundle.fee);
    }

    function dispatch(MessageBundle calldata bundle) internal {
        for (uint256 i = 0; i < bundle.messages.length; i++) {
            Message calldata message = bundle.messages[i];
            (bool success, ) = message.target.call{ value: 0, gas: MAX_GAS_PER_MESSAGE }(
                message.payload
            );
            emit MessageDispatched(message.id, success);
        }
    }
}

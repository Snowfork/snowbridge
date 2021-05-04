// SPDX-License-Identifier: MIT
pragma solidity >=0.7.6;
pragma experimental ABIEncoderV2;

import "@openzeppelin/contracts/access/AccessControl.sol";
import "@openzeppelin/contracts/math/SafeMath.sol";
import "./RewardSource.sol";

contract IncentivizedInboundChannel is AccessControl {

    uint64 public nonce;

    struct Message {
        address target;
        uint64 nonce;
        uint256 fee;
        bytes payload;
    }

    event MessageDispatched(uint64 nonce, bool result);

    uint256 constant public MAX_GAS_PER_MESSAGE = 100000;

    // Governance contracts will administer using this role.
    bytes32 public constant CONFIG_UPDATE_ROLE = keccak256("CONFIG_UPDATE_ROLE");

    event RelayerNotRewarded(address relayer, uint256 amount);

    RewardSource private rewardSource;

    constructor() {
        _setupRole(DEFAULT_ADMIN_ROLE, msg.sender);
    }

    // Once-off post-construction call to set initial configuration.
    function initialize(
        address _configUpdater,
        address _rewardSource
    )
    external {
        require(hasRole(DEFAULT_ADMIN_ROLE, msg.sender), "Caller is unauthorized");

        // Set initial configuration
        grantRole(CONFIG_UPDATE_ROLE, _configUpdater);
        rewardSource = RewardSource(_rewardSource);

        // drop admin privileges
        renounceRole(DEFAULT_ADMIN_ROLE, msg.sender);
    }

    // TODO: Submit should take in all inputs required for verification
    function submit(Message[] calldata _messages, bytes32 _commitment)
        public
    {
        verifyMessages(_messages, _commitment);
        processMessages(msg.sender, _messages);
    }

    //TODO: verifyMessages should accept all needed proofs
    function verifyMessages(Message[] calldata _messages, bytes32 _commitment)
        internal
        view
        returns (bool success)
    {
        require(
            validateMessagesMatchCommitment(_messages, _commitment),
            "invalid commitment"
        );

        // Require there is enough gas to play all messages
        require(
            gasleft() >= _messages.length * MAX_GAS_PER_MESSAGE,
            "insufficient gas for delivery of all messages"
        );


        return true;
    }

    function processMessages(address payable _relayer, Message[] calldata _messages) internal {
        uint256 _rewardAmount = 0;

        for (uint256 i = 0; i < _messages.length; i++) {
            // Check message nonce is correct and increment nonce for replay protection
            require(_messages[i].nonce == nonce + 1, "invalid nonce");

            nonce = nonce + 1;

            // Deliver the message to the target
            // Delivery will have fixed maximum gas allowed for the target app
            (bool success, ) =
                _messages[i].target.call{value: 0, gas: MAX_GAS_PER_MESSAGE}(_messages[i].payload);

            _rewardAmount = _rewardAmount + _messages[i].fee;
            emit MessageDispatched(_messages[i].nonce, success);
        }

        // Attempt to reward the relayer
        try rewardSource.reward(_relayer, _rewardAmount) { }
        catch {
            emit RelayerNotRewarded(_relayer, _rewardAmount);
        }
    }

    function validateMessagesMatchCommitment(
        Message[] calldata _messages,
        bytes32 _commitment
    ) internal pure returns (bool) {
        return keccak256(abi.encode(_messages)) == _commitment;
    }
}

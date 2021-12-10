// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.5;
pragma experimental ABIEncoderV2;

import "@openzeppelin/contracts/access/AccessControl.sol";
import "./ParachainLightClient.sol";
import "./RewardSource.sol";
import "./SimplifiedMMRVerification.sol";

contract IncentivizedInboundChannel is AccessControl {
    uint64 public nonce;

    struct Message {
        address target;
        uint64 nonce;
        uint256 fee;
        bytes payload;
    }

    event MessageDispatched(uint64 nonce, bool result);

    uint256 public constant MAX_GAS_PER_MESSAGE = 100000;
    uint256 public constant GAS_BUFFER = 60000;

    // Governance contracts will administer using this role.
    bytes32 public constant CONFIG_UPDATE_ROLE =
        keccak256("CONFIG_UPDATE_ROLE");

    RewardSource private rewardSource;

    BeefyLightClient public beefyLightClient;

    constructor(BeefyLightClient _beefyLightClient) {
        _setupRole(DEFAULT_ADMIN_ROLE, msg.sender);
        beefyLightClient = _beefyLightClient;
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

    function submit(
        Message[] calldata _messages,
        ParachainLightClient.ParachainVerifyInput
            calldata _parachainVerifyInput,
        ParachainLightClient.BeefyMMRLeafPartial calldata _beefyMMRLeafPartial,
        SimplifiedMMRProof calldata proof
    ) public {
        // Proof
        // 1. Compute our parachain's message `commitment` by ABI encoding and hashing the `_messages`
        bytes32 commitment = keccak256(abi.encode(_messages));

        ParachainLightClient.verifyCommitmentInParachain(
            commitment,
            _parachainVerifyInput,
            _beefyMMRLeafPartial,
            proof,
            beefyLightClient
        );

        // Require there is enough gas to play all messages
        require(
            gasleft() >= (_messages.length * MAX_GAS_PER_MESSAGE) + GAS_BUFFER,
            "insufficient gas for delivery of all messages"
        );

        processMessages(payable(msg.sender), _messages);
    }

    function processMessages(
        address payable _relayer,
        Message[] calldata _messages
    ) internal {
        uint256 _rewardAmount = 0;
        // Caching nonce for gas optimization
        uint64 cachedNonce = nonce;

        for (uint256 i = 0; i < _messages.length; i++) {
            // Check message nonce is correct and increment nonce for replay protection
            require(_messages[i].nonce == cachedNonce + 1, "invalid nonce");
            cachedNonce = cachedNonce + 1;

            // Deliver the message to the target
            // Delivery will have fixed maximum gas allowed for the target app
            (bool success, ) = _messages[i].target.call{
                value: 0,
                gas: MAX_GAS_PER_MESSAGE
            }(_messages[i].payload);

            _rewardAmount = _rewardAmount + _messages[i].fee;
            emit MessageDispatched(_messages[i].nonce, success);
        }

        // reward the relayer
        rewardSource.reward(_relayer, _rewardAmount);
        nonce = cachedNonce;
    }
}

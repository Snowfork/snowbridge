// SPDX-License-Identifier: MIT
pragma solidity >=0.7.6;
pragma experimental ABIEncoderV2;

import "@openzeppelin/contracts/math/SafeMath.sol";
import "./BeefyLightClient.sol";
import "./ParachainLightClient.sol";

contract BasicInboundChannel {
    uint256 public constant MAX_GAS_PER_MESSAGE = 100000;

    uint64 public nonce;

    struct Message {
        address target;
        uint64 nonce; // TODO: this might cause an error, we use uint256 when encoding on Parachain
        bytes payload;
    }

    event MessageDispatched(uint64 nonce, bool result);

    BeefyLightClient public beefyLightClient;

    constructor(BeefyLightClient _beefyLightClient) {
        nonce = 0;
        beefyLightClient = _beefyLightClient;
    }

    // TODO: add docs
    function submit(
        Message[] calldata _messages,
        ParachainLightClient.OwnParachainHeadPartial
            calldata _ownParachainHeadPartial,
        bytes32[] calldata _parachainHeadsProof,
        BeefyLightClient.BeefyMMRLeafPartial calldata _beefyMMRLeafPartial,
        uint256 _beefyMMRLeafIndex,
        uint256 _beefyMMRLeafCount,
        bytes32[] calldata _beefyMMRLeafProof
    ) public {
        // Require there is enough gas to play all messages
        require(
            gasleft() >= _messages.length * MAX_GAS_PER_MESSAGE,
            "insufficient gas for delivery of all messages"
        );

        // Proof
        // 1. Compute our parachain's message `commitment` by ABI encoding and hashing the `_messages`
        // TODO
        // bytes32 commitment = keccak256(abi.encodePacked(_messages));

        // 2. Compute `ownParachainHead` by hashing the data of the `commitment` together with the contents of
        // `_ownParachainHeadPartial`
        // TODO
        // bytes32 ownParachainHead = keccak256(abi.encodePacked(ParachainLightClient.OwnParachainHead(
        //     _ownParachainHeadPartial.parentHash,
        //     _ownParachainHeadPartial.number,
        //     _ownParachainHeadPartial.stateRoot,
        //     _ownParachainHeadPartial.extrinsicsRoot,
        //     commitment,
        // )));

        // 3. Compute `parachainHeadsRoot` by verifying the merkle proof using `ownParachainHead` and
        // `_parachainHeadsProof`
        // TODO

        // 4. Compute the `beefyMMRLeaf` using `parachainHeadsRoot` and `_beefyMMRLeafPartial`
        // TODO

        // 5. Verify inclusion of the beefy MMR leaf in the beefy MMR root using that `beefyMMRLeaf` as well as
        // `_beefyMMRLeafIndex`, `_beefyMMRLeafCount` and `_beefyMMRLeafProof`
        // require(
        //     beefyLightClient.verifyBeefyMerkleLeaf(
        //         beefyMMRLeaf,
        //         _beefyMMRLeafIndex,
        //         _beefyMMRLeafCount,
        //         _beefyMMRLeafProof
        //     ),
        //     "Invalid proof"
        // );

        processMessages(_messages);
    }

    function processMessages(Message[] calldata _messages) internal {
        for (uint256 i = 0; i < _messages.length; i++) {
            // Check message nonce is correct and increment nonce for replay protection
            require(_messages[i].nonce == nonce + 1, "invalid nonce");

            nonce = nonce + 1;

            // Deliver the message to the target
            (bool success, ) =
                _messages[i].target.call{value: 0, gas: MAX_GAS_PER_MESSAGE}(
                    _messages[i].payload
                );

            emit MessageDispatched(_messages[i].nonce, success);
        }
    }

    function validateMessagesMatchCommitment(
        Message[] calldata _messages,
        bytes32 _commitment
    ) internal pure returns (bool) {
        return keccak256(abi.encode(_messages)) == _commitment;
    }
}

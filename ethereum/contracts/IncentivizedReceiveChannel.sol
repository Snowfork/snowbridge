pragma solidity >=0.6.2;
pragma experimental ABIEncoderV2;

import "@openzeppelin/contracts/math/SafeMath.sol";
import "./Decoder.sol";

contract IncentivizedReceiveChannel {
    using Decoder for bytes;

    uint256 public lastProcessedNonce;

    uint256 public MAX_PAYLOAD_BYTE_SIZE = 1000;
    uint256 public MAX_PAYLOAD_GAS_COST = 500000;
    uint256 public EXTERNAL_CALL_COST = 21000;
    uint256 public MAX_GAS_PER_MESSAGE =
        EXTERNAL_CALL_COST + MAX_PAYLOAD_BYTE_SIZE + EXTERNAL_CALL_COST;

    struct Message {
        uint256 nonce;
        string senderApplicationId;
        address targetApplicationAddress;
        bytes payload;
    }

    struct Commitment {
        bytes commitmentHash;
    }

    struct CommitmentContents {
        Message[] messages;
    }

    constructor() public {
        lastProcessedNonce = 0;
    }

    event MessageDelivered(uint256 _nonce, bool _result);

    function newParachainCommitment(
        Commitment memory commitment,
        CommitmentContents memory commitmentContents,
        uint256 parachainBlockNumber,
        bytes memory ourParachainMerkleProof,
        bytes memory parachainHeadsMMRProof
    ) public {
        verifyCommitment(
            commitment,
            commitmentContents,
            ourParachainMerkleProof,
            parachainHeadsMMRProof
        );
        processCommitmentContents(commitmentContents);
    }

    function verifyCommitment(
        Commitment memory commitment,
        CommitmentContents memory commitmentContents,
        bytes memory ourParachainMerkleProof,
        bytes memory parachainHeadsMMRProof
    ) internal returns (bool success) {
        // Prove we can get the MMRLeaf that is claimed to contain our Parachain Block Header
        // BEEFYLightClient.verifyMMRLeaf(parachainHeadsMMRProof)
        // BeefyLightClient{
        //   verifyMMRLeaf(parachainHeadsMMRProof) {
        //   MMRVerification.verifyInclusionProof(latestMMRRoot, parachainHeadsMMRProof)
        // }
        //}
        //}
        // returns mmrLeaf;

        // Prove we can get the claimed parachain block header from the MMRLeaf
        // allParachainHeadsMerkleTreeRoot = mmrLeaf.parachain_heads;
        // MerkeTree.verify(allParachainHeadsMerkleTreeRoot, ourParachainMerkleProof)
        // returns parachainBlockHeader

        // Prove that the commitment is in fact in the parachain block header
        // require(parachainBlockHeader.commitment == commitment)

        // Validate that the commitment matches the commitment contents
        require(
            validateCommitment(commitment, commitmentContents),
            "invalid commitment"
        );

        // Require there is enough gas to play all messages
        require(
            gasleft() >=
                commitmentContents.messages.length * MAX_GAS_PER_MESSAGE,
            "insufficient gas for delivery of all messages"
        );

        // Require all payloads are smaller than max_payload_size
        for (uint256 i = 0; i < commitmentContents.messages.length; i++) {
            require(
                commitmentContents.messages[i].payload.length <=
                    MAX_PAYLOAD_BYTE_SIZE,
                "message payload bytesize exceeds maximum payload size"
            );
        }
        return true;
    }

    function processCommitmentContents(
        CommitmentContents memory commitmentContents
    ) internal {
        for (uint256 i = 0; i < commitmentContents.messages.length; i++) {
            // Check message nonce is correct and increment nonce for replay protection
            Message memory message = commitmentContents.messages[i];
            require(message.nonce == lastProcessedNonce + 1, "invalid nonce");

            lastProcessedNonce = lastProcessedNonce + 1;

            // Deliver the message to the destination
            // Delivery will have fixed maximum gas allowed for the destination app.
            address targetApplicationAddress = message.targetApplicationAddress;
            uint256 allowedGas = MAX_GAS_PER_MESSAGE;
            bytes memory callInput = message.payload;

            bool success;
            bytes memory result;
            (success, result) = targetApplicationAddress.call.value(0).gas(
                allowedGas
            )(callInput);

            emit MessageDelivered(message.nonce, success);
        }
    }

    function validateCommitment(
        Commitment memory commitment,
        CommitmentContents memory commitmentContents
    ) internal returns (bool) {
        bytes32 commitmentHash;
        for (uint256 i = 0; i < commitmentContents.messages.length; i++) {
            if (i == 0) {
                commitmentHash = hashMessage(commitmentContents.messages[i]);
            } else {
                bytes32 messageHash =
                    hashMessage(commitmentContents.messages[i]);
                commitmentHash = keccak256(
                    abi.encodePacked(commitmentHash, messageHash)
                );
            }
        }
        return
            keccak256(abi.encodePacked(commitmentHash)) ==
            keccak256(abi.encodePacked(commitment.commitmentHash));
    }

    function hashMessage(Message memory message) internal returns (bytes32) {
        return
            keccak256(
                abi.encodePacked(
                    message.nonce,
                    message.senderApplicationId,
                    message.targetApplicationAddress,
                    message.payload
                )
            );
    }
}

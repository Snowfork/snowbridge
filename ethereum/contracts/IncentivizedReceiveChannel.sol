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
        // verifyMMRLeaf(parachainHeadsMMRProof) {
        //MMRVerification.verifyInclusionProof(latestMMRRoot, parachainHeadsMMRProof)
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

        // Prove that the commitmentContents match the commitment
        // require(commitment == hash(commitmentContents))

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

            bool result;
            uint256 callInputLength = callInput.length;
            assembly {
                let callInputDataPointer := add(callInput, 32) // Move 32 ahead of callInput length slot to get to actual data slot (TODO: verify this works in all cases)

                // Dispatch call to the receiver - it is expected to be fire and forget. If the call reverts, runs out of gas, error,
                // etc, its the fault of the sender
                result := call(
                    allowedGas, // Allowed gas
                    targetApplicationAddress, // To addr
                    0, // No ether value being sent
                    callInputDataPointer, // Inputs are stored at callInputDataPointer
                    callInputLength, // Input length
                    callInputDataPointer, // Store output over input (saves space)
                    callInputLength // Outputs can be as long as input length
                )
            }

            emit MessageDelivered(message.nonce, result);
        }
    }
}

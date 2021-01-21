pragma solidity >=0.6.2;
pragma experimental ABIEncoderV2;

import "@openzeppelin/contracts/math/SafeMath.sol";

contract IncentivizedReceiveChannel {
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
        for (uint256 i = 0; i < commitmentContents.length; i++) {
            // Check message nonce is correct and increment nonce for replay protection
            Message memory message = commitmentContents[i];
            require(message.nonce == lastProcessedNonce + 1, "invalid nonce");

            lastProcessedNonce = lastProcessedNonce + 1;

            // Deliver the message to the destination
            // Delivery will have fixed maximum gas allowed for the destination app.
            // TODO: payload needs format: [bytes4(sha3("function_name(arg1_type,arg2_type)")) + bytes input]
            bytes4 sig = bytes4(sha3("add(int256,int256)"));

            assembly {
                let x := mload(0x40) // Find empty storage location using "free memory pointer"
                mstore(x, sig) // Place signature at begining of empty storage
                mstore(add(x, 0x04), a) // Place first argument directly next to signature
                mstore(add(x, 0x24), b) // Place second argument next to first, padded to 32 bytes

                // Dispatch call to the receiver - it is expected to be fire and forget. If the call reverts, runs out of gas, error,
                // etc, its the fault of the sender
                let success := call(
                    // Pop the top stack value
                    MAX_GAS_PER_MESSAGE, // Allowed gas
                    message.targetApplicationAddress, // To addr
                    0, // No value
                    x, // Inputs are stored at location x
                    0x44, // Inputs are 68 bytes long
                    x, // Store output over input (saves space)
                    0x20
                ) // Outputs are 32 bytes long

                c := mload(x) // Assign output value to c
                mstore(0x40, add(x, 0x44)) // Set storage pointer to empty space
            }
        }
    }
}

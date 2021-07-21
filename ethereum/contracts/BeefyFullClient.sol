// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.5;
pragma experimental ABIEncoderV2;

import "./ValidatorRegistryFull.sol";
import "./MMRVerification.sol";
import "./Blake2b.sol";
import "./ScaleCodec.sol";

/**
 * @title A entry contract for the Ethereum light client
 */
contract BeefyFullClient {
    using ScaleCodec for uint256;
    using ScaleCodec for uint64;
    using ScaleCodec for uint32;
    using ScaleCodec for uint16;

    /* Events */

    event NewMMRRoot(bytes32 mmrRoot, uint64 blockNumber);

    /* Types */

    /**
     * The Commitment, with its payload, is the core thing we are trying to verify with
     * this contract. It contains a MMR root that commits to the polkadot history, including
     * past blocks and parachain blocks and can be used to verify both polkadot and parachain blocks.
     * @param payload the payload of the new commitment in beefy justifications (in
     * our case, this is a new MMR root for all past polkadot blocks)
     * @param blockNumber block number for the given commitment
     * @param validatorSetId validator set id that signed the given commitment
     */
    struct Commitment {
        bytes32 payload;
        uint64 blockNumber;
        uint32 validatorSetId;
    }

    /**
     * The BeefyMMRLeaf is the structure of each leaf in each MMR that each commitment's payload commits to.
     * @param parentNumber parent number of the block this leaf describes
     * @param parentHash parent hash of the block this leaf describes
     * @param parachainHeadsRoot merkle root of all parachain headers in this block
     * @param nextAuthoritySetId validator set id that will be part of consensus for the next block
     * @param nextAuthoritySetLen length of that validator set
     * @param nextAuthoritySetRoot merkle root of all public keys in that validator set
     */
    struct BeefyMMRLeaf {
        uint32 parentNumber;
        bytes32 parentHash;
        bytes32 parachainHeadsRoot;
        uint64 nextAuthoritySetId;
        uint32 nextAuthoritySetLen;
        bytes32 nextAuthoritySetRoot;
    }

    /* State */

    ValidatorRegistryFull public validatorRegistryFull;
    MMRVerification public mmrVerification;
    Blake2b public blake2b;
    bytes32 public latestMMRRoot;
    uint64 public latestBeefyBlock;

    /* Constants */

    // THRESHOLD_NUMERATOR - numerator for percent of validator signatures required
    // THRESHOLD_DENOMINATOR - denominator for percent of validator signatures required
    uint256 public constant THRESHOLD_NUMERATOR = 3;
    uint256 public constant THRESHOLD_DENOMINATOR = 250;

    // We must ensure at least one block is processed every session,
    // so these constants are checked to enforce a maximum gap between commitments.
    uint64 public constant NUMBER_OF_BLOCKS_PER_SESSION = 100;
    uint64 public constant ERROR_AND_SAFETY_BUFFER = 10;
    uint64 public constant MAXIMUM_BLOCK_GAP =
        NUMBER_OF_BLOCKS_PER_SESSION - ERROR_AND_SAFETY_BUFFER;

    /**
     * @notice Deploys the BeefyLightClient contract
     * @param _validatorRegistryFull The contract to be used as the validator registry
     * @param _mmrVerification The contract to be used for MMR verification
     */
    constructor(
        ValidatorRegistryFull _validatorRegistryFull,
        MMRVerification _mmrVerification,
        Blake2b _blake2b,
        uint64 _startingBeefyBlock
    ) {
        validatorRegistryFull = _validatorRegistryFull;
        mmrVerification = _mmrVerification;
        blake2b = _blake2b;
        latestBeefyBlock = _startingBeefyBlock;
    }

    /* Public Functions */

    /**
     * @notice Executed by the incoming channel in order to verify commitment
     * @param beefyMMRLeaf contains the merkle leaf to be verified
     * @param beefyMMRLeafIndex contains the merkle leaf index
     * @param beefyMMRLeafCount contains the merkle leaf count
     * @param beefyMMRLeafProof contains the merkle proof to verify against
     */
    function verifyBeefyMerkleLeaf(
        bytes32 beefyMMRLeaf,
        uint256 beefyMMRLeafIndex,
        uint256 beefyMMRLeafCount,
        bytes32[] calldata beefyMMRLeafProof
    ) external returns (bool) {
        return
            mmrVerification.verifyInclusionProof(
                latestMMRRoot,
                beefyMMRLeaf,
                beefyMMRLeafIndex,
                beefyMMRLeafCount,
                beefyMMRLeafProof
            );
    }

    function newBeefyBlock(
        Commitment calldata commitment,
        bytes[] calldata signatures,
        BeefyMMRLeaf calldata latestMMRLeaf,
        bytes32[] calldata mmrProofItems
    ) public {
        verifyCommitment(commitment, signatures);
        verifyNewestMMRLeaf(
            latestMMRLeaf,
            mmrProofItems,
            commitment.payload,
            commitment.blockNumber
        );

        processPayload(commitment.payload, commitment.blockNumber);

        applyValidatorSetChanges(
            latestMMRLeaf.nextAuthoritySetId,
            latestMMRLeaf.nextAuthoritySetLen,
            latestMMRLeaf.nextAuthoritySetRoot
        );
    }

    /* Private Functions */

    function verifyNewestMMRLeaf(
        BeefyMMRLeaf calldata leaf,
        bytes32[] calldata proof,
        bytes32 root,
        uint64 length
    ) public {
        bytes memory encodedLeaf = encodeMMRLeaf(leaf);
        bytes32 hashedLeaf = hashMMRLeaf(encodedLeaf);

        mmrVerification.verifyInclusionProof(
            root,
            hashedLeaf,
            length - 1,
            length,
            proof
        );
    }

    /**
     * @notice Perform some operation[s] using the payload
     * @param payload The payload variable passed in via the initial function
     */
    function processPayload(bytes32 payload, uint64 blockNumber) private {
        // Check that payload.leaf.block_number is > last_known_block_number;
        require(
            blockNumber > latestBeefyBlock,
            "Payload blocknumber is too old"
        );

        // Check that payload is within the current or next session
        // to ensure we get at least one payload each session
        require(
            blockNumber < latestBeefyBlock + MAXIMUM_BLOCK_GAP,
            "Payload blocknumber is too new"
        );

        latestMMRRoot = payload;
        latestBeefyBlock = blockNumber;
        emit NewMMRRoot(latestMMRRoot, blockNumber);
    }

    /**
     * @notice Check if the payload includes a new validator set,
     * and if it does then update the new validator set
     * @dev This function should call out to the validator registry contract
     * @param nextAuthoritySetId The id of the next authority set
     * @param nextAuthoritySetLen The number of validators in the next authority set
     * @param nextAuthoritySetRoot The merkle root of the merkle tree of the next validators
     */
    function applyValidatorSetChanges(
        uint64 nextAuthoritySetId,
        uint32 nextAuthoritySetLen,
        bytes32 nextAuthoritySetRoot
    ) internal {
        if (nextAuthoritySetId != validatorRegistryFull.id()) {
            // TODO: do update full
        }
    }

    function requiredNumberOfSignatures() public view returns (uint256) {
        return
            (validatorRegistryFull.numOfValidators() *
                THRESHOLD_NUMERATOR +
                THRESHOLD_DENOMINATOR -
                1) / THRESHOLD_DENOMINATOR;
    }

    function requiredNumberOfSignatures(uint256 numValidators)
        public
        pure
        returns (uint256)
    {
        return
            (numValidators * THRESHOLD_NUMERATOR + THRESHOLD_DENOMINATOR - 1) /
            THRESHOLD_DENOMINATOR;
    }

    function verifyCommitment(
        Commitment calldata commitment,
        bytes[] calldata signatures
    ) internal view {
        uint256 numberOfValidators = validatorRegistryFull.numOfValidators();
        uint256 requiredNumOfSignatures = requiredNumberOfSignatures(
            numberOfValidators
        );

        bytes32 commitmentHash = createCommitmentHash(commitment);

        uint256 correctSignatures = validatorRegistryFull.checkSignatures(
            signatures,
            commitmentHash
        );

        require(
            correctSignatures >= requiredNumOfSignatures,
            "Error: Number of signatures does not match required"
        );
    }

    function createCommitmentHash(Commitment calldata commitment)
        public
        view
        returns (bytes32)
    {
        return
            blake2b.formatOutput(
                blake2b.blake2b(
                    abi.encodePacked(
                        commitment.payload,
                        commitment.blockNumber.encode64(),
                        commitment.validatorSetId.encode32()
                    ),
                    "",
                    32
                )
            )[0];
    }

    bytes2 public constant MMR_LEAF_LENGTH_SCALE_ENCODED =
        bytes2(uint16(0xc101));

    function encodeMMRLeaf(BeefyMMRLeaf calldata leaf)
        public
        pure
        returns (bytes memory)
    {
        bytes memory scaleEncodedMMRLeaf = abi.encodePacked(
            ScaleCodec.encode32(leaf.parentNumber),
            leaf.parentHash,
            leaf.parachainHeadsRoot,
            ScaleCodec.encode64(leaf.nextAuthoritySetId),
            ScaleCodec.encode32(leaf.nextAuthoritySetLen),
            leaf.nextAuthoritySetRoot
        );

        return bytes.concat(MMR_LEAF_LENGTH_SCALE_ENCODED, scaleEncodedMMRLeaf);
    }

    function hashMMRLeaf(bytes memory leaf) public pure returns (bytes32) {
        return keccak256(leaf);
    }
}

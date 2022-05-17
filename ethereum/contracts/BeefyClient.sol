// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "./utils/Bits.sol";
import "./utils/Bitfield.sol";
import "./utils/MMRProofVerification.sol";
import "./ScaleCodec.sol";
import "./utils/MerkleProof.sol";

/**
 * @title A entry contract for the BEEFY light client
 */
contract BeefyClient is Ownable {
    using Bits for uint256;
    using Bitfield for uint256[];
    using ScaleCodec for uint256;
    using ScaleCodec for uint64;
    using ScaleCodec for uint32;
    using ScaleCodec for uint16;

    /* Events */

    /**
     * @dev Emitted when a pre-submission request is validated
     * @param id the identifier for the submission request
     * @param sender The address of the sender
     */
    event NewRequest(uint256 id, address sender);

    /**
     * @dev Emitted when the MMR root is updated
     * @param mmrRoot the updated MMR root
     * @param blockNumber the beefy block number of the updated MMR root
     */
    event NewMMRRoot(bytes32 mmrRoot, uint64 blockNumber);

    /* Types */

    /**
     * @dev The Commitment, with its payload, is the core thing we are trying to verify with
     * this contract. It contains a MMR root that commits to the polkadot history, including
     * past blocks and parachain blocks and can be used to verify both polkadot and parachain blocks.
     * @param payload the payload of the new commitment in beefy justifications (in
     * our case, this is a new MMR root for all past polkadot blocks)
     * @param blockNumber block number for the given commitment
     * @param validatorSetID validator set id that signed the given commitment
     */
    struct Commitment {
        uint32 blockNumber;
        uint64 validatorSetID;
        Payload payload;
    }

    struct Payload {
        bytes32 mmrRootHash;
        bytes prefix;
        bytes suffix;
    }

    /**
     * @dev The ValidatorMultiProof is a collection of proofs used to verify a commitment signature
     * @param signatures an array of validator signatures
     * @param indices an array of the leaf indices
     * @param addrs an array of each validator address
     * @param merkleProofs an array of merkle proofs from the chosen validators
     */
    struct ValidatorMultiProof {
        bytes[] signatures;
        uint256[] indices;
        address[] addrs;
        bytes32[][] merkleProofs;
    }

    /**
     * @dev The ValidatorProof is a proof used to verify a commitment signature
     * @param signature validator signature
     * @param index index of the validator address
     * @param addr validator address
     * @param merkleProof merkle proof for the validator
     */
    struct ValidatorProof {
        bytes signature;
        uint256 index;
        address addr;
        bytes32[] merkleProof;
    }

    /**
     * @dev A request is used to link initial and final submission of a commitment
     * @param sender the sender of the initial transaction
     * @param commitmentHash the hash of the commitment they are claiming has been signed
     * @param bitfield a bitfield signalling which validators they claim have signed
     * @param blockNumber the block number for this commitment
     */
    struct Request {
        address sender;
        bytes32 commitmentHash;
        uint256[] bitfield;
        uint256 blockNumber;
        ValidatorSet vset;
    }

    /**
     * @dev The MMRLeaf is the structure of each leaf in each MMR that each commitment's payload commits to.
     * @param version version of the leaf type
     * @param parentNumber parent number of the block this leaf describes
     * @param parentHash parent hash of the block this leaf describes
     * @param parachainHeadsRoot merkle root of all parachain headers in this block
     * @param nextAuthoritySetID validator set id that will be part of consensus for the next block
     * @param nextAuthoritySetLen length of that validator set
     * @param nextAuthoritySetRoot merkle root of all public keys in that validator set
     */
    struct MMRLeaf {
        uint8 version;
        uint32 parentNumber;
        bytes32 parentHash;
        uint64 nextAuthoritySetID;
        uint32 nextAuthoritySetLen;
        bytes32 nextAuthoritySetRoot;
        bytes32 parachainHeadsRoot;
    }

    /**
     * @dev The ValidatorSet describes a BEEFY validator set
     * @param id identifier for the set
     * @param root Merkle root of BEEFY validator addresses
     * @param length number of validators in the set
     */
    struct ValidatorSet {
        uint256 id;
        bytes32 root;
        uint256 length;
    }

    /* State */

    bytes32 public latestMMRRoot;
    uint64 public latestBeefyBlock;

    ValidatorSet public currentValidatorSet;
    ValidatorSet public nextValidatorSet;

    uint256 public nextRequestID;
    mapping(uint256 => Request) public requests;

    /* Constants */

    // Used for calculating minimum number of required signatures
    uint256 public constant THRESHOLD_NUMERATOR = 3;
    uint256 public constant THRESHOLD_DENOMINATOR = 250;
    uint64 public constant BLOCK_WAIT_PERIOD = 3;

    /**
     * @dev Deploys the BeefyClient contract
     */
    constructor() {
        nextRequestID = 0;
    }

    // Once-off post-construction call to set initial configuration.
    function initialize(
        uint64 _initialBeefyBlock,
        ValidatorSet calldata _initialValidatorSet,
        ValidatorSet calldata _nextValidatorSet
    ) external onlyOwner {
        latestBeefyBlock = _initialBeefyBlock;
        currentValidatorSet = _initialValidatorSet;
        nextValidatorSet = _nextValidatorSet;
        renounceOwnership();
    }

    /* Public Functions */

    /**
     * @notice Executed by the prover in order to begin the process of block
     * acceptance by the light client
     * @param commitmentHash contains the commitmentHash signed by the validator(s)
     * @param bitfield a bitfield containing a membership status of each
     * validator who has claimed to have signed the commitmentHash
     * @param proof the validator proof
     */
    function submitInitial(
        bytes32 commitmentHash,
        uint64 validatorSetID,
        uint256[] calldata bitfield,
        ValidatorProof calldata proof
    ) external payable {
        // for pre-submission, we accept commitments from either the current or next validator set
        ValidatorSet memory vset;
        if (validatorSetID == currentValidatorSet.id) {
            vset = currentValidatorSet;
        } else if (validatorSetID == nextValidatorSet.id) {
            vset = nextValidatorSet;
        } else {
            revert("Unknown validator set");
        }

        // Check if merkle proof is valid based on the validatorSetRoot
        require(
            isValidatorInSet(vset, proof.addr, proof.index, proof.merkleProof),
            "invalid validator proof"
        );

        // Check if validatorSignature is correct, ie. check if it matches
        // the signature of senderPublicKey on the commitmentHash
        require(ECDSA.recover(commitmentHash, proof.signature) == proof.addr, "Invalid signature");

        // Check that the bitfield actually contains enough claims to be successful, ie, >= 2/3
        require(
            bitfield.countSetBits() >= minimumSignatureThreshold(vset),
            "Not enough claims"
        );

        // Accept and save the commitment
        requests[nextRequestID] = Request(
            msg.sender,
            commitmentHash,
            bitfield,
            block.number,
            vset
        );

        emit NewRequest(nextRequestID, msg.sender);

        nextRequestID = nextRequestID + 1;
    }

    /**
     * @dev Submit a commitment for final verification
     * @param requestID identifier for the request generated by the initial submission
     * @param commitment contains the full commitment that was used for the commitmentHash
     * @param proof a struct containing the data needed to verify all validator signatures
     */
    function submitFinal(
        uint256 requestID,
        Commitment calldata commitment,
        ValidatorMultiProof calldata proof
    ) public {
        Request storage request = requests[requestID];

        require(commitment.validatorSetID == currentValidatorSet.id);

        verifyCommitment(currentValidatorSet, request, commitment, proof);

        latestMMRRoot = commitment.payload.mmrRootHash;
        latestBeefyBlock = commitment.blockNumber;
        emit NewMMRRoot(commitment.payload.mmrRootHash, commitment.blockNumber);

        delete requests[requestID];
    }

    /**
     * @dev Submit a commitment and leaf for final verification
     * @param requestID identifier for the request generated by the initial submission
     * @param commitment contains the full commitment that was used for the commitmentHash
     * @param proof a struct containing the data needed to verify all validator signatures
     * @param leaf an MMR leaf provable using the MMR root in the commitment payload
     * @param leafProof an MMR leaf proof
     */
    function submitFinal(
        uint256 requestID,
        Commitment calldata commitment,
        ValidatorMultiProof calldata proof,
        MMRLeaf calldata leaf,
        MMRProof calldata leafProof
    ) public {
        Request storage request = requests[requestID];

        require(commitment.validatorSetID == nextValidatorSet.id);
        require(leaf.nextAuthoritySetID == nextValidatorSet.id + 1);

        verifyCommitment(nextValidatorSet, request, commitment, proof);

        require(
            MMRProofVerification.verifyLeafProof(
                commitment.payload.mmrRootHash,
                keccak256(encodeMMRLeaf(leaf)),
                leafProof
            ),
            "Invalid leaf proof"
        );

        currentValidatorSet = nextValidatorSet;
        nextValidatorSet.id = leaf.nextAuthoritySetID;
        nextValidatorSet.root = leaf.nextAuthoritySetRoot;
        nextValidatorSet.length = leaf.nextAuthoritySetLen;

        latestMMRRoot = commitment.payload.mmrRootHash;
        latestBeefyBlock = commitment.blockNumber;
        emit NewMMRRoot(commitment.payload.mmrRootHash, commitment.blockNumber);

        delete requests[requestID];
    }

    /**
     * @dev Executed by the incoming channel in order to verify leaf inclusion in the MMR.
     * @param leafHash contains the merkle leaf to be verified
     * @param proof contains simplified mmr proof
     */
    function verifyMMRLeafProof(bytes32 leafHash, MMRProof calldata proof)
        external
        view
        returns (bool)
    {
        return MMRProofVerification.verifyLeafProof(latestMMRRoot, leafHash, proof);
    }

    /* Private Functions */

    /**
     * @notice Deterministically generates a seed from the block hash at the block number of creation of the validation
     * request plus BLOCK_WAIT_PERIOD.
     * @dev Note that `blockhash(blockNum)` will only work for the 256 most recent blocks. If
     * `submit` is called too late, a new call to `presubmit` is necessary to reset
     * validation request's block number
     * @param request a storage reference to the requests struct
     * @return uint256 the derived seed
     */
    function deriveSeed(Request storage request) internal view returns (uint256) {
        return uint256(blockhash(request.blockNumber + BLOCK_WAIT_PERIOD));
    }

    function minimumSignatureThreshold(ValidatorSet memory vset) internal pure returns (uint256) {
        return
            (vset.length * THRESHOLD_NUMERATOR + THRESHOLD_DENOMINATOR - 1) / THRESHOLD_DENOMINATOR;
    }

    /**
     * @dev Verify commitment using the validator multiproof
     */
    function verifyCommitment(
        ValidatorSet memory vset,
        Request storage request,
        Commitment calldata commitment,
        ValidatorMultiProof calldata proof
    ) internal view {
        // Verify that sender is the same as in `submitInitial`
        require(msg.sender == request.sender, "Sender address invalid");

        // Verify that block wait period has passed
        require(
            block.number >= request.blockNumber + BLOCK_WAIT_PERIOD,
            "Block wait period not over"
        );

        // Check that payload.leaf.block_number is > last_known_block_number;
        require(commitment.blockNumber > latestBeefyBlock, "Commitment blocknumber is too old");

        // verify the validator multiproof
        uint256 signatureCount = minimumSignatureThreshold(vset);
        uint256[] memory finalBitfield = Bitfield.randomNBitsWithPriorCheck(
            deriveSeed(request),
            request.bitfield,
            signatureCount,
            vset.length
        );
        bytes32 commitmentHash = keccak256(encodeCommitment(commitment));
        verifyValidatorMultiProof(proof, signatureCount, vset, finalBitfield, commitmentHash);
    }

    function verifyValidatorMultiProof(
        ValidatorMultiProof calldata proof,
        uint256 signatureCount,
        ValidatorSet memory vset,
        uint256[] memory bitfield,
        bytes32 commitmentHash
    ) internal pure {
        require(
            proof.signatures.length == signatureCount &&
                proof.indices.length == signatureCount &&
                proof.addrs.length == signatureCount &&
                proof.merkleProofs.length == signatureCount,
            "Validator proof is malformed"
        );

        for (uint256 i = 0; i < signatureCount; i++) {
            (
                bytes calldata signature,
                uint256 index,
                address addr,
                bytes32[] calldata merkleProof
            ) = (proof.signatures[i], proof.indices[i], proof.addrs[i], proof.merkleProofs[i]);

            // Check if validator in bitfield
            require(bitfield.isSet(index), "Validator not in bitfield");

            // Remove validator from bitfield such that no validator can appear twice in signatures
            bitfield.clear(index);

            // Check if merkle proof is valid
            require(isValidatorInSet(vset, addr, index, merkleProof), "invalid validator proof");

            // Check if signature is correct
            require(ECDSA.recover(commitmentHash, signature) == addr, "Invalid signature");
        }
    }

    function encodeCommitment(Commitment calldata commitment) internal pure returns (bytes memory) {
        return
            bytes.concat(
                commitment.payload.prefix,
                commitment.payload.mmrRootHash,
                commitment.payload.suffix,
                commitment.blockNumber.encode32(),
                commitment.validatorSetID.encode64()
            );
    }

    function encodeMMRLeaf(MMRLeaf calldata leaf) internal pure returns (bytes memory) {
        return
            bytes.concat(
                ScaleCodec.encode8(leaf.version),
                ScaleCodec.encode32(leaf.parentNumber),
                leaf.parentHash,
                ScaleCodec.encode64(leaf.nextAuthoritySetID),
                ScaleCodec.encode32(leaf.nextAuthoritySetLen),
                leaf.nextAuthoritySetRoot,
                leaf.parachainHeadsRoot
            );
    }

    /**
     * @dev Checks if a validators address is a member of the merkle tree
     * @param addr The address of the validator to check
     * @param index The index of the validator to check, starting at 0
     * @param proof Merkle proof required for validation of the address
     * @return true if the validator is in the set
     */
    function isValidatorInSet(
        ValidatorSet memory vset,
        address addr,
        uint256 index,
        bytes32[] memory proof
    ) internal pure returns (bool) {
        bytes32 hashedLeaf = keccak256(abi.encodePacked(addr));
        return
            MerkleProof.verifyMerkleLeafAtPosition(
                vset.root,
                hashedLeaf,
                index,
                vset.length,
                proof
            );
    }

    /**
     * @dev Helper to create an initial validator bitfield.
     */
    function createInitialBitfield(uint256[] calldata bitsToSet, uint256 length)
        external
        pure
        returns (uint256[] memory)
    {
        return Bitfield.createBitfield(bitsToSet, length);
    }

    /**
     * @dev Helper to create a final bitfield, with random validator selections.
     */
    function createFinalBitfield(uint256 requestID) external view returns (uint256[] memory) {
        Request storage request = requests[requestID];

        // verify that block wait period has passed
        require(block.number >= request.blockNumber + BLOCK_WAIT_PERIOD, "wait period not over");

        return
            Bitfield.randomNBitsWithPriorCheck(
                deriveSeed(request),
                request.bitfield,
                minimumSignatureThreshold(request.vset),
                request.vset.length
            );
    }
}

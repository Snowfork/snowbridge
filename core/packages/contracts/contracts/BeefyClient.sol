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
 * @title BeefyClient
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
        ValidatorSignature[] signatures;
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
        ValidatorSignature signature;
        uint256 index;
        address addr;
        bytes32[] merkleProof;
    }

    /**
     * @dev The ValidatorSignature is the separated components of a secp256k1 signature as
     * mentioned in EIP-2098: https://eips.ethereum.org/EIPS/eip-2098
     * @param v the parity bit to specify the indended solution
     * @param r the x component on the secp256k1 curve
     * @param s the challenge solution
     */
    struct ValidatorSignature {
        uint8 v;
        bytes32 r;
        bytes32 s;
    }

    /**
     * @dev A request is used to link initial and final submission of a commitment
     * @param sender the sender of the initial transaction
     * @param blockNumber the block number for this commitment
     * @param validatorSetLen the length of the validator set for this commitment
     * @param bitfield a bitfield signalling which validators they claim have signed
     */
    struct Request {
        address sender;
        uint64 blockNumber;
        uint32 validatorSetLen;
        uint256[] bitfield;
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

    /**
     * TODO: Review this constant (SNO-355)
     *
     * @dev The minimum number of blocks a relayer must wait between submissions
     * in the interactive update protcol. The longer the period, the greater the
     * crypto-economic security.
     */
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

        // NOTE: Disabling renouncing of ownership to support lean BEEFY.
        // This will be added back once full BEEFY is supported.
        // See SNO-294, SNO-297
        //
        // renounceOwnership();
    }

    /* Public Functions */

    /**
     * @dev Executed by the prover in order to begin the process of block
     * acceptance by the light client
     * @param commitmentHash contains the commitmentHash signed by the validator(s)
     * @param validatorSetID the id of the validator set which signed the commitment
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
        require(ECDSA.recover(commitmentHash, proof.signature.v, proof.signature.r, proof.signature.s) == proof.addr, "Invalid signature");

        // For the initial commitment, more than two thirds of the validator set should claim to sign the commitment
        require(
            bitfield.countSetBits() >= vset.length - (vset.length - 1) / 3,
            "Not enough claims"
        );

        // Accept and save the commitment
        requests[nextRequestID] = Request(
            msg.sender,
            uint64(block.number),
            uint32(vset.length),
            bitfield
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

        require(commitment.validatorSetID == currentValidatorSet.id, "invalid commitment");

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
    function submitFinalWithLeaf(
        uint256 requestID,
        Commitment calldata commitment,
        ValidatorMultiProof calldata proof,
        MMRLeaf calldata leaf,
        MMRProof calldata leafProof
    ) public {
        Request storage request = requests[requestID];

        require(commitment.validatorSetID == nextValidatorSet.id, "invalid commitment");
        require(leaf.nextAuthoritySetID == nextValidatorSet.id + 1, "invalid MMR leaf");

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
     * TODO: Ensure request is not too old as the blockhash function
     * only works for the 256 most recent blocks (SNO-354).
     *
     * @dev Deterministically generate a seed based on the initial request
     * @param request a storage reference to the requests struct
     * @return uint256 the seed
     */
    function deriveSeed(Request storage request) internal virtual view returns (uint256) {
        return uint256(blockhash(request.blockNumber + BLOCK_WAIT_PERIOD));
    }

    /**
     * @dev Calculate minimum number of required signatures for the current validator set.
     *
     * This function approximates f(x) defined below for x in [1, 21_846):
     *
     *  x <= 10: f(x) = x * (2/3)
     *  x  > 10: f(x) = max(10, ceil(log2(3 * x))
     *
     * Research by W3F suggests that `ceil(log2(3 * x))` is a minimum number of signatures required to make an
     * attack unfeasible. We put a further minimum bound of 10 on this value for extra security.
     *
     * If the session has less than 10 active validators it's definitely some sort of local testnet
     * and we use different logic (minimum 2/3 + 1 validators must sign).
     *
     * One assumption is that Polkadot/Kusama will never have more than 21_845 active validators in a session.
     * As of writing this comment, Polkadot has 300 validators and Kusama has around 1000 validators,
     * so we are well within those limits.
     *
     * In any case, an order of magnitude increase in validator set sizes will likely require a re-architecture
     * of Polkadot that would make this contract obsolete well before the assumption becomes a problem.
     *
     * Constants generated with the help of scripts/minsigs.py
     */
    function minimumSignatureThreshold(uint256 validatorSetLen) internal pure returns (uint256) {
        if (validatorSetLen <= 10) {
            return validatorSetLen - (validatorSetLen - 1) / 3;
        } else if (validatorSetLen < 342) {
            return 10;
        } else if (validatorSetLen < 683) {
            return 11;
        } else if (validatorSetLen < 1366) {
            return 12;
        } else if (validatorSetLen < 2731) {
            return 13;
        } else if (validatorSetLen < 5462) {
            return 14;
        } else if (validatorSetLen < 10923) {
            return 15;
        } else if (validatorSetLen < 21846) {
            return 16;
        } else {
            return 17;
        }
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
        require(commitment.blockNumber > latestBeefyBlock, "Commitment is too old");

        // verify the validator multiproof
        uint256 signatureCount = minimumSignatureThreshold(vset.length);
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
                ValidatorSignature calldata signature,
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
            require(ECDSA.recover(commitmentHash, signature.v, signature.r, signature.s) == addr, "Invalid signature");
        }
    }

    function encodeCommitment(Commitment calldata commitment) internal pure returns (bytes memory) {
        return
            bytes.concat(
                commitment.payload.prefix,
                commitment.payload.mmrRootHash,
                commitment.payload.suffix,
                commitment.blockNumber.encodeU32(),
                commitment.validatorSetID.encodeU64()
            );
    }

    function encodeMMRLeaf(MMRLeaf calldata leaf) internal pure returns (bytes memory) {
        return
            bytes.concat(
                ScaleCodec.encodeU8(leaf.version),
                ScaleCodec.encodeU32(leaf.parentNumber),
                leaf.parentHash,
                ScaleCodec.encodeU64(leaf.nextAuthoritySetID),
                ScaleCodec.encodeU32(leaf.nextAuthoritySetLen),
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
                minimumSignatureThreshold(request.validatorSetLen),
                request.validatorSetLen
            );
    }
}

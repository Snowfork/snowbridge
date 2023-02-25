// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/utils/cryptography/MerkleProof.sol";
import "./utils/Bitfield.sol";
import "./utils/MMRProof.sol";
import "./ScaleCodec.sol";

/**
 * @title BeefyClient
 */
contract BeefyClient is Ownable {
    /* Events */

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
     * @dev The ValidatorProof is a proof used to verify a commitment signature
     * @param v the parity bit to specify the intended solution
     * @param r the x component on the secp256k1 curve
     * @param s the challenge solution
     * @param index index of the validator address
     * @param addr validator address
     * @param proof merkle proof for the validator
     */
    struct ValidatorProof {
        uint8 v;
        bytes32 r;
        bytes32 s;
        uint256 index;
        address account;
        bytes32[] proof;
    }

    /**
     * @dev A task is used to link initial and final submission of a commitment
     * @param sender the sender of the initial transaction
     * @param bitfield a bitfield signalling which validators they claim have signed
     * @param blockNumber the block number for this commitment
     * @param validatorSetLen the length of the validator set for this commitment
     * @param bitfield a bitfield signalling which validators they claim have signed
     */
    struct Task {
        address account;
        uint64 blockNumber;
        uint32 validatorSetLen;
        uint256 prevRandao;
        bytes32 bitfieldHash;
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
     * @param length number of validators in the set
     * @param root Merkle root of BEEFY validator addresses
     */
    struct ValidatorSet {
        uint128 id;
        uint128 length;
        bytes32 root;
    }

    /* State */

    bytes32 public latestMMRRoot;
    uint64 public latestBeefyBlock;

    ValidatorSet public currentValidatorSet;
    ValidatorSet public nextValidatorSet;

    mapping(bytes32 => Task) public tasks;

    /* Constants */

    /**
     * @dev Minimum delay in number of blocks that a relayer must wait between calling
     * submitInitial and commitPrevRandao. In production this should be set to MAX_SEED_LOOKAHEAD:
     * https://eth2book.info/altair/part3/config/preset#max_seed_lookahead
     */
    uint256 public immutable randaoCommitDelay;

    /**
     * @dev after randaoCommitDelay is reached, relayer must
     * call commitPrevRandao within this number of blocks.
     */
    uint256 public immutable randaoCommitExpiration;

    /* Errors */

    error InvalidCommitment();
    error StaleCommitment();
    error InvalidValidatorProof();
    error InvalidSignature();
    error NotEnoughClaims();
    error InvalidMMRLeaf();
    error InvalidMMRLeafProof();
    error InvalidTask();
    error InvalidBitfield();
    error WaitPeriodNotOver();
    error TaskExpired();
    error PrevRandaoAlreadyCaptured();
    error PrevRandaoNotCaptured();

    constructor(uint256 _randaoCommitDelay, uint256 _randaoCommitExpiration) {
        randaoCommitDelay = _randaoCommitDelay;
        randaoCommitExpiration = _randaoCommitExpiration;
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
     * @dev Begin submission of signature proof for current session
     * @param commitmentHash contains the commitmentHash signed by the validators
     * @param bitfield a bitfield claiming which validators have signed the commitment
     * @param proof the validator proof
     */
    function submitInitial(
        bytes32 commitmentHash,
        uint256[] calldata bitfield,
        ValidatorProof calldata proof
    ) external payable {
        doSubmitInitial(currentValidatorSet, commitmentHash, bitfield, proof);
    }

    /**
     * @dev Begin submission of signature proof for next session
     * @param commitmentHash contains the commitmentHash signed by the validators
     * @param bitfield a bitfield claiming which validators have signed the commitment
     * @param proof the validator proof
     */
    function submitInitialWithHandover(
        bytes32 commitmentHash,
        uint256[] calldata bitfield,
        ValidatorProof calldata proof
    ) external payable {
        doSubmitInitial(nextValidatorSet, commitmentHash, bitfield, proof);
    }

    function doSubmitInitial(
        ValidatorSet memory vset,
        bytes32 commitmentHash,
        uint256[] calldata bitfield,
        ValidatorProof calldata proof
    ) internal {
        // Check if merkle proof is valid based on the validatorSetRoot
        if (!isValidatorInSet(vset, proof.account, proof.proof)) {
            revert InvalidValidatorProof();
        }

        // Check if validatorSignature is correct, ie. check if it matches
        // the signature of senderPublicKey on the commitmentHash
        if (ECDSA.recover(commitmentHash, proof.v, proof.r, proof.s) != proof.account) {
            revert InvalidSignature();
        }

        // For the initial commitment, the bitfield should claim that more than
        // two thirds of the validator set have sign the commitment
        if (Bitfield.countSetBits(bitfield) < vset.length - (vset.length - 1) / 3) {
            revert NotEnoughClaims();
        }

        tasks[createTaskID(msg.sender, commitmentHash)] = Task(
            msg.sender,
            uint64(block.number),
            uint32(vset.length),
            0,
            keccak256(abi.encodePacked(bitfield))
        );
    }

    /**
     * @dev Capture PREVRANDAO
     * @param commitmentHash contains the commitmentHash signed by the validators
     */
    function commitPrevRandao(bytes32 commitmentHash) external {
        bytes32 taskID = createTaskID(msg.sender, commitmentHash);
        Task storage task = tasks[taskID];

        if (task.prevRandao != 0) {
            revert PrevRandaoAlreadyCaptured();
        }

        if (block.number < task.blockNumber + randaoCommitDelay) {
            revert WaitPeriodNotOver();
        }

        if (block.number > task.blockNumber + randaoCommitDelay + randaoCommitExpiration) {
            delete tasks[taskID];
            revert TaskExpired();
        }

        // Post-merge, the difficulty opcode now returns PREVRANDAO
        task.prevRandao = block.difficulty;
    }

    /**
     * @dev Submit a commitment for final verification
     * @param commitment contains the full commitment that was used for the commitmentHash
     * @param proofs a struct containing the data needed to verify all validator signatures
     */
    function submitFinal(
        Commitment calldata commitment,
        uint256[] calldata bitfield,
        ValidatorProof[] calldata proofs
    ) public {
        bytes32 commitmentHash = keccak256(encodeCommitment(commitment));
        bytes32 taskID = createTaskID(msg.sender, commitmentHash);
        Task storage task = tasks[taskID];

        if (task.prevRandao == 0) {
            revert PrevRandaoNotCaptured();
        }

        if (commitment.validatorSetID != currentValidatorSet.id) {
            revert InvalidCommitment();
        }

        if (commitment.blockNumber <= latestBeefyBlock) {
            revert StaleCommitment();
        }

        if (task.bitfieldHash != keccak256(abi.encodePacked(bitfield))) {
            revert InvalidBitfield();
        }

        verifyCommitment(commitmentHash, bitfield, currentValidatorSet, task, proofs);

        latestMMRRoot = commitment.payload.mmrRootHash;
        latestBeefyBlock = commitment.blockNumber;
        emit NewMMRRoot(commitment.payload.mmrRootHash, commitment.blockNumber);
        delete tasks[taskID];
    }

    /**
     * @dev Submit a commitment and leaf for final verification
     * @param commitment contains the full commitment that was used for the commitmentHash
     * @param proofs a struct containing the data needed to verify all validator signatures
     * @param leaf an MMR leaf provable using the MMR root in the commitment payload
     * @param leafProof an MMR leaf proof
     */
    function submitFinalWithHandover(
        Commitment calldata commitment,
        uint256[] calldata bitfield,
        ValidatorProof[] calldata proofs,
        MMRLeaf calldata leaf,
        bytes32[] calldata leafProof,
        uint256 leafProofOrder
    ) public {
        bytes32 commitmentHash = keccak256(encodeCommitment(commitment));
        bytes32 taskID = createTaskID(msg.sender, commitmentHash);
        Task storage task = tasks[taskID];

        if (task.prevRandao == 0) {
            revert PrevRandaoNotCaptured();
        }

        if (commitment.validatorSetID != nextValidatorSet.id) {
            revert InvalidCommitment();
        }

        if (commitment.blockNumber <= latestBeefyBlock) {
            revert StaleCommitment();
        }

        if (
            leaf.nextAuthoritySetID != nextValidatorSet.id &&
            leaf.nextAuthoritySetID != nextValidatorSet.id + 1
        ) {
            revert InvalidMMRLeaf();
        }

        if (task.bitfieldHash != keccak256(abi.encodePacked(bitfield))) {
            revert InvalidBitfield();
        }

        verifyCommitment(commitmentHash, bitfield, nextValidatorSet, task, proofs);

        bool leafIsValid = MMRProof.verifyLeafProof(
            commitment.payload.mmrRootHash,
            keccak256(encodeMMRLeaf(leaf)),
            leafProof,
            leafProofOrder
        );
        if (!leafIsValid) {
            revert InvalidMMRLeafProof();
        }

        currentValidatorSet = nextValidatorSet;
        // todo: workaround and need to check if a bug in substrate
        nextValidatorSet.id = leaf.nextAuthoritySetID + 1;
        nextValidatorSet.length = leaf.nextAuthoritySetLen;
        nextValidatorSet.root = leaf.nextAuthoritySetRoot;

        latestMMRRoot = commitment.payload.mmrRootHash;
        latestBeefyBlock = commitment.blockNumber;
        emit NewMMRRoot(commitment.payload.mmrRootHash, commitment.blockNumber);
        delete tasks[taskID];
    }

    /**
     * @dev Executed by the incoming channel in order to verify leaf inclusion in the MMR.
     * @param leafHash contains the merkle leaf to be verified
     * @param proof contains simplified mmr proof
     */
    function verifyMMRLeafProof(
        bytes32 leafHash,
        bytes32[] calldata proof,
        uint256 proofOrder
    ) external view returns (bool) {
        return MMRProof.verifyLeafProof(latestMMRRoot, leafHash, proof, proofOrder);
    }

    /* Private Functions */

    function createTaskID(
        address account,
        bytes32 commitmentHash
    ) internal pure returns (bytes32 value) {
        assembly {
            mstore(0x00, account)
            mstore(0x20, commitmentHash)
            value := keccak256(0x0, 0x40)
        }
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
        bytes32 commitmentHash,
        uint256[] calldata bitfield,
        ValidatorSet memory vset,
        Task storage task,
        ValidatorProof[] calldata proofs
    ) internal view {
        // verify the validator multiproof
        uint256 signatureCount = minimumSignatureThreshold(vset.length);
        uint256[] memory finalbitfield = Bitfield.randomNBitsWithPriorCheck(
            task.prevRandao,
            bitfield,
            signatureCount,
            vset.length
        );

        if (proofs.length != signatureCount) {
            revert InvalidValidatorProof();
        }

        for (uint256 i = 0; i < signatureCount; ) {
            ValidatorProof calldata proof = proofs[i];

            (uint256 x, uint8 y) = Bitfield.toLocation(proof.index);

            if (!Bitfield.isSet(finalbitfield, x, y)) {
                revert InvalidValidatorProof();
            }

            if (!isValidatorInSet(vset, proof.account, proof.proof)) {
                revert InvalidValidatorProof();
            }

            if (ECDSA.recover(commitmentHash, proof.v, proof.r, proof.s) != proof.account) {
                revert InvalidSignature();
            }

            // Ensure no validator can appear more than once
            Bitfield.clear(finalbitfield, x, y);

            unchecked {
                i++;
            }
        }
    }

    function encodeCommitment(Commitment calldata commitment) internal pure returns (bytes memory) {
        return
            bytes.concat(
                commitment.payload.prefix,
                commitment.payload.mmrRootHash,
                commitment.payload.suffix,
                ScaleCodec.encodeU32(commitment.blockNumber),
                ScaleCodec.encodeU64(commitment.validatorSetID)
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
     * @param proof Merkle proof required for validation of the address
     * @return true if the validator is in the set
     */
    function isValidatorInSet(
        ValidatorSet memory vset,
        address addr,
        bytes32[] calldata proof
    ) internal pure returns (bool) {
        bytes32 hashedLeaf = keccak256(abi.encodePacked(addr));
        return MerkleProof.verify(proof, vset.root, hashedLeaf);
    }

    /**
     * @dev Helper to create an initial validator bitfield.
     */
    function createInitialBitfield(
        uint256[] calldata bitsToSet,
        uint256 length
    ) external pure returns (uint256[] memory) {
        return Bitfield.createBitfield(bitsToSet, length);
    }

    /**
     * @dev Helper to create a final bitfield, with random validator selections.
     */
    function createFinalBitfield(
        bytes32 commitmentHash,
        uint256[] calldata bitfield
    ) external view returns (uint256[] memory) {
        Task storage task = tasks[createTaskID(msg.sender, commitmentHash)];
        return
            Bitfield.randomNBitsWithPriorCheck(
                task.prevRandao,
                bitfield,
                minimumSignatureThreshold(task.validatorSetLen),
                task.validatorSetLen
            );
    }
}

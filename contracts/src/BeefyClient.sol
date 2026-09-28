// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.34;

import {ECDSA} from "openzeppelin/utils/cryptography/ECDSA.sol";
import {SubstrateMerkleProof} from "./utils/SubstrateMerkleProof.sol";
import {Bitfield} from "./utils/Bitfield.sol";
import {Uint16Array, createUint16Array} from "./utils/Uint16Array.sol";
import {Math} from "./utils/Math.sol";
import {MMRProof} from "./utils/MMRProof.sol";
import {ScaleCodec} from "./utils/ScaleCodec.sol";

/**
 * @title BeefyClient
 * @dev This is a client for verifying BEEFY commitments from the Polkadot network.
 * it contains two ways to verify BEEFY commitments from a Substrate-based chain,
 * one through an interactive protocol, and one through Fiat-Shamir transformation.
 *
 * The interactive protocol is defined in https://eprint.iacr.org/2025/057.pdf. Higher level documentation
 * is available at https://docs.snowbridge.network/architecture/verification/polkadot.
 *
 * To submit new commitments, relayers must call the following methods sequentially:
 * 1. submitInitial: Initializes the session for interactive submission and waits for the Randao delay period
 * 2. commitPrevRandao: Commit to a random seed for generating a validator subsampling
 * 3. createFinalBitfield: Generate the validator subsampling
 * 4. submitFinal: Complete submission after providing the request validator signatures
 *
 *
 * The non-interactive protocol eliminates the need for interaction by applying Fiat-Shamir transform.
 * It is defined in Section 6 of https://eprint.iacr.org/2025/057.pdf, with higher-level documentation
 * available at https://hackmd.io/8Jd7V74iSSeeHOIG76REWw
 *
 * To submit new commitments using the Fiat-Shamir approach, relayers call the following methods sequentially:
 * 1. createFiatShamirFinalBitfield: Generate the validator subsampling using Fiat-Shamir
 * 2. submitFiatShamir: Complete submission after providing the request validator signatures
 */
contract BeefyClient {
    using Math for uint16;
    using Math for uint256;

    /* Events */

    /**
     * @dev Emitted when the MMR root is updated
     * @param mmrRoot the updated MMR root
     * @param blockNumber the beefy block number of the updated MMR root
     */
    event NewMMRRoot(bytes32 mmrRoot, uint64 blockNumber);

    /**
     * @dev Emitted when a new ticket has been created
     * @param relayer The relayer who created the ticket
     * @param blockNumber the parent block number of the candidate MMR root
     */
    event NewTicket(address relayer, uint64 blockNumber);

    /**
     * @dev Interactive session has expired
     */
    event TicketExpired();

    /* Types */

    /**
     * @dev The Commitment, with its payload, is the core thing we are trying to verify with
     * this contract. It contains an MMR root that commits to the polkadot history, including
     * past blocks and parachain blocks and can be used to verify both polkadot and parachain blocks.
     */
    struct Commitment {
        // Relay chain block number
        uint32 blockNumber;
        // ID of the validator set that signed the commitment
        uint64 validatorSetID;
        // The payload of the new commitment in beefy justifications (in
        // our case, this is a new MMR root for all past polkadot blocks)
        PayloadItem[] payload;
    }

    /**
     * @dev Each PayloadItem is a piece of data signed by validators at a particular block.
     */
    struct PayloadItem {
        // An ID that references a description of the data in the payload item.
        // Known payload ids can be found [upstream](https://github.com/paritytech/substrate/blob/fe1f8ba1c4f23931ae89c1ada35efb3d908b50f5/primitives/consensus/beefy/src/payload.rs#L27).
        bytes2 payloadID;
        // The contents of the payload item
        bytes data;
    }

    /**
     * @dev The ValidatorProof is a proof used to verify a commitment signature
     */
    struct ValidatorProof {
        // The parity bit to specify the intended solution
        uint8 v;
        // The x component on the secp256k1 curve
        bytes32 r;
        // The challenge solution
        bytes32 s;
        // Leaf index of the validator address in the merkle tree
        uint256 index;
        // Validator address
        address account;
        // Merkle proof for the validator
        bytes32[] proof;
    }

    /**
     * @dev A ticket tracks working state for the interactive submission of new commitments.
     *
     * Each relayer has one ticket, and its two slots are reused from one submission to the
     * next. It is never deleted: a used or expired ticket only has `blockNumber` set to zero.
     * The slots then stay non-zero, so the next ticket updates them instead of creating them,
     * which is much cheaper once state creation is repriced.
     *
     * Slot 0 holds everything that changes during a session, so `commitPrevRandao` writes one
     * slot. `submitInitial` rewrites slot 0 whole, which resets `seed` to zero: a new ticket
     * can never use the previous ticket's seed. Slot 1 binds the ticket to its commitment and
     * bitfield. It must stay a full 256-bit hash: `submitFinal` does not re-check the bitfield
     * quorum, so a collision would let a relayer swap in a bitfield that claims only
     * attacker-controlled validators.
     */
    struct Ticket {
        // The block number this ticket was issued. Zero when no ticket is active.
        uint64 blockNumber;
        // Length of the validator set that signed the commitment
        uint32 validatorSetLen;
        // The number of signatures required
        uint32 numRequiredSignatures;
        // Sampling seed: the low 120 bits of PREVRANDAO, never zero once captured. Zero means
        // `commitPrevRandao` has not run for this ticket.
        uint120 seed;
        // keccak256(commitmentHash, keccak256(bitfield)), see `computeClaimHash`
        bytes32 claimHash;
    }

    /// @dev The MMRLeaf describes the leaf structure of the MMR
    struct MMRLeaf {
        // Version of the leaf type
        uint8 version;
        // Parent number of the block this leaf describes
        uint32 parentNumber;
        // Parent hash of the block this leaf describes
        bytes32 parentHash;
        // Validator set id that will be part of consensus for the next block
        uint64 nextAuthoritySetID;
        // Length of that validator set
        uint32 nextAuthoritySetLen;
        // Merkle root of all public keys in that validator set
        bytes32 nextAuthoritySetRoot;
        // Merkle root of all parachain headers in this block
        bytes32 parachainHeadsRoot;
    }

    /**
     * @dev The ValidatorSet describes a BEEFY validator set
     */
    struct ValidatorSet {
        // Identifier for the set
        uint128 id;
        // Number of validators in the set
        uint128 length;
        // Merkle root of BEEFY validator addresses
        bytes32 root;
    }

    /**
     * @dev The ValidatorSetState describes a BEEFY validator set along with signature usage counters
     */
    struct ValidatorSetState {
        // Identifier for the set
        uint128 id;
        // Number of validators in the set
        uint128 length;
        // Merkle root of BEEFY validator addresses
        bytes32 root;
        // Number of times a validator signature has been used
        Uint16Array usageCounters;
    }

    /* State */

    /// @dev The latest verified MMR root
    bytes32 public latestMMRRoot;

    /// @dev The block number in the relay chain in which the latest MMR root was emitted
    uint64 public latestBeefyBlock;

    /// @dev State of the current validator set
    ValidatorSetState public currentValidatorSet;

    /// @dev State of the next validator set
    ValidatorSetState public nextValidatorSet;

    /// @dev The ticket of each relayer. At most one is active per relayer.
    mapping(address relayer => Ticket) public tickets;

    /* Constants */

    /**
     * @dev Fiat-Shamir domain separator ID
     */
    bytes public constant FIAT_SHAMIR_DOMAIN_ID = bytes("SNOWBRIDGE-FIAT-SHAMIR-V1");

    /**
     * @dev Beefy payload id for MMR Root payload items:
     * https://github.com/paritytech/substrate/blob/fe1f8ba1c4f23931ae89c1ada35efb3d908b50f5/primitives/consensus/beefy/src/payload.rs#L33
     */
    // forge-lint: disable-next-line(unsafe-typecast)
    bytes2 public constant MMR_ROOT_ID = bytes2("mh");

    /**
     * @dev Minimum delay in number of blocks that a relayer must wait between calling
     * submitInitial and commitPrevRandao. In production this should be set to MAX_SEED_LOOKAHEAD:
     * https://eth2book.info/altair/part3/config/preset#max_seed_lookahead
     */
    uint256 public immutable randaoCommitDelay;

    /**
     * @dev after randaoCommitDelay is reached, relayer must
     * call commitPrevRandao within this number of blocks.
     * Without this expiration, relayers can roll the dice infinitely to get the subsampling
     * they desire.
     */
    uint256 public immutable randaoCommitExpiration;

    /**
     * @dev The lower bound on the number of signatures required to validate a new commitment. Note
     * that the final number of signatures is calculated dynamically with Randao randomness.
     */
    uint256 public immutable minNumRequiredSignatures;

    /**
     * @dev The signatures required to validate a new commitment using Fiat-Shamir transform. Note
     * that Fiat-shamir comes from the assumption on the hash-power of adversary
     * and its not a crypto-economic argument.
     */
    uint256 public immutable fiatShamirRequiredSignatures;

    /* Errors */
    error InvalidBitfield();
    error InvalidBitfieldLength();
    error InvalidCommitment();
    error InvalidMMRLeaf();
    error InvalidMMRLeafProof();
    error InvalidMMRRootLength();
    error InvalidSignature();
    error InvalidTicket();
    error InvalidValidatorProof();
    error InvalidValidatorProofLength();
    error CommitmentNotRelevant();
    error PrevRandaoAlreadyCaptured();
    error PrevRandaoNotCaptured();
    error StaleCommitment();
    error WaitPeriodNotOver();

    constructor(
        uint256 _randaoCommitDelay,
        uint256 _randaoCommitExpiration,
        uint256 _minNumRequiredSignatures,
        uint256 _fiatShamirRequiredSignatures,
        uint64 _initialBeefyBlock,
        ValidatorSet memory _initialValidatorSet,
        ValidatorSet memory _nextValidatorSet
    ) {
        if (_nextValidatorSet.id != _initialValidatorSet.id + 1) {
            revert("invalid-constructor-params");
        }
        randaoCommitDelay = _randaoCommitDelay;
        randaoCommitExpiration = _randaoCommitExpiration;
        minNumRequiredSignatures = _minNumRequiredSignatures;
        fiatShamirRequiredSignatures = _fiatShamirRequiredSignatures;
        latestBeefyBlock = _initialBeefyBlock;
        currentValidatorSet.id = _initialValidatorSet.id;
        currentValidatorSet.length = _initialValidatorSet.length;
        currentValidatorSet.root = _initialValidatorSet.root;
        currentValidatorSet.usageCounters = createUint16Array(currentValidatorSet.length);
        nextValidatorSet.id = _nextValidatorSet.id;
        nextValidatorSet.length = _nextValidatorSet.length;
        nextValidatorSet.root = _nextValidatorSet.root;
        nextValidatorSet.usageCounters = createUint16Array(nextValidatorSet.length);
    }

    /* External Functions */

    /**
     * @dev Begin submission of commitment
     * @param commitment contains the commitment signed by the validators
     * @param bitfield a bitfield claiming which validators have signed the commitment
     * @param proof a proof that a single validator from currentValidatorSet has signed the commitment
     */
    function submitInitial(
        Commitment calldata commitment,
        uint256[] calldata bitfield,
        ValidatorProof calldata proof
    ) external {
        if (commitment.blockNumber <= latestBeefyBlock) {
            revert StaleCommitment();
        }

        ValidatorSetState storage vset = currentValidatorSet;
        uint16 signatureUsageCount;
        if (commitment.validatorSetID == currentValidatorSet.id) {
            signatureUsageCount = currentValidatorSet.usageCounters.get(proof.index);
            currentValidatorSet.usageCounters
                .set(proof.index, signatureUsageCount.saturatingAdd(1));
        } else if (commitment.validatorSetID == nextValidatorSet.id) {
            signatureUsageCount = nextValidatorSet.usageCounters.get(proof.index);
            nextValidatorSet.usageCounters.set(proof.index, signatureUsageCount.saturatingAdd(1));
            vset = nextValidatorSet;
        } else {
            revert InvalidCommitment();
        }

        // Check if merkle proof is valid based on the validatorSetRoot and if proof is included in bitfield
        if (
            !isValidatorInSet(vset, proof.account, proof.index, proof.proof)
                || !Bitfield.isSet(bitfield, proof.index)
        ) {
            revert InvalidValidatorProof();
        }

        // Check if validatorSignature is correct, ie. check if it matches
        // the signature of senderPublicKey on the commitmentHash
        bytes32 commitmentHash = keccak256(encodeCommitment(commitment));
        if (ECDSA.recover(commitmentHash, proof.v, proof.r, proof.s) != proof.account) {
            revert InvalidSignature();
        }

        // For the initial submission, the supplied bitfield should claim that more than
        // two thirds of the validator set have sign the commitment
        if (
            bitfield.length != Bitfield.containerLength(vset.length)
                || Bitfield.countSetBits(bitfield, vset.length) < computeQuorum(vset.length)
        ) {
            revert InvalidBitfield();
        }

        // Validate that all padding bits (beyond vset.length) are zero
        // This ensures the bitfield was created by createInitialBitfield or equivalent
        Bitfield.validatePadding(bitfield, vset.length);

        // Replaces any ticket this relayer already has, with `seed` reset to zero.
        tickets[msg.sender] = Ticket({
            blockNumber: uint64(block.number),
            validatorSetLen: uint32(vset.length),
            numRequiredSignatures: uint32(
                computeNumRequiredSignatures(
                    vset.length, signatureUsageCount, minNumRequiredSignatures
                )
            ),
            seed: 0,
            claimHash: computeClaimHash(commitmentHash, bitfield)
        });

        emit NewTicket(msg.sender, commitment.blockNumber);
    }

    /**
     * @dev Capture PREVRANDAO for the caller's open ticket.
     *
     * The parameter is kept for ABI compatibility and is not checked: a relayer has only one
     * ticket, and `submitFinal` checks the commitment and bitfield against `claimHash`.
     */
    function commitPrevRandao(bytes32) external {
        Ticket storage ticket = tickets[msg.sender];

        if (ticket.blockNumber == 0) {
            revert InvalidTicket();
        }

        if (ticket.seed != 0) {
            revert PrevRandaoAlreadyCaptured();
        }

        // relayer must wait `randaoCommitDelay` blocks
        if (block.number < ticket.blockNumber + randaoCommitDelay) {
            revert WaitPeriodNotOver();
        }

        // relayer can capture within `randaoCommitExpiration` blocks
        if (block.number > ticket.blockNumber + randaoCommitDelay + randaoCommitExpiration) {
            ticket.blockNumber = 0;
            emit TicketExpired();
            return;
        }

        // Post-merge, the difficulty opcode now returns PREVRANDAO. 120 bits is ample for a
        // sampling seed; zero is reserved for "not captured".
        // forge-lint: disable-next-line(unsafe-typecast)
        uint120 seed = uint120(block.prevrandao);
        ticket.seed = seed == 0 ? 1 : seed;
    }

    /**
     * @dev Submit a commitment and leaf for final verification
     * @param commitment contains the full commitment that was used for the commitmentHash
     * @param bitfield claiming which validators have signed the commitment
     * @param proofs a struct containing the data needed to verify all validator signatures
     * @param leaf an MMR leaf provable using the MMR root in the commitment payload
     * @param leafProof an MMR leaf proof
     * @param leafProofOrder a bitfield describing the order of each item (left vs right)
     */
    function submitFinal(
        Commitment calldata commitment,
        uint256[] calldata bitfield,
        ValidatorProof[] calldata proofs,
        MMRLeaf calldata leaf,
        bytes32[] calldata leafProof,
        uint256 leafProofOrder
    ) external {
        bytes32 commitmentHash = keccak256(encodeCommitment(commitment));
        Ticket storage ticket = validateTicket(commitmentHash, commitment, bitfield);

        bool is_next_session = false;
        ValidatorSetState storage vset = currentValidatorSet;
        if (commitment.validatorSetID == nextValidatorSet.id) {
            is_next_session = true;
            vset = nextValidatorSet;
        } else if (commitment.validatorSetID != currentValidatorSet.id) {
            revert InvalidCommitment();
        }

        // Validate that all padding bits (beyond vset.length) are zero
        // This ensures the bitfield was created by createInitialBitfield or equivalent
        Bitfield.validatePadding(bitfield, vset.length);

        verifyCommitment(commitmentHash, ticket, bitfield, vset, proofs);

        bytes32 newMMRRoot = ensureProvidesMMRRoot(commitment);

        if (is_next_session) {
            // The id for candidate nextValidatorSet should be greater than the current
            // nextValidatorSet id
            if (leaf.nextAuthoritySetID <= nextValidatorSet.id) {
                revert InvalidMMRLeaf();
            }
            bool leafIsValid = MMRProof.verifyLeafProof(
                newMMRRoot, keccak256(encodeMMRLeaf(leaf)), leafProof, leafProofOrder
            );
            if (!leafIsValid) {
                revert InvalidMMRLeafProof();
            }
            currentValidatorSet = nextValidatorSet;
            nextValidatorSet.id = leaf.nextAuthoritySetID;
            nextValidatorSet.length = leaf.nextAuthoritySetLen;
            nextValidatorSet.root = leaf.nextAuthoritySetRoot;
            nextValidatorSet.usageCounters = createUint16Array(leaf.nextAuthoritySetLen);
        }

        latestMMRRoot = newMMRRoot;
        latestBeefyBlock = commitment.blockNumber;
        // Close the ticket but keep its slots non-zero for the relayer's next ticket. The stale
        // seed is unreachable while `blockNumber` is zero, and `submitInitial` resets it.
        ticket.blockNumber = 0;

        emit NewMMRRoot(newMMRRoot, commitment.blockNumber);
    }

    /**
     * @dev Verify that the supplied MMR leaf is included in the latest verified MMR root.
     * @param leafHash contains the merkle leaf to be verified
     * @param proof contains simplified mmr proof
     * @param proofOrder a bitfield describing the order of each item (left vs right)
     */
    function verifyMMRLeafProof(bytes32 leafHash, bytes32[] calldata proof, uint256 proofOrder)
        external
        view
        returns (bool)
    {
        return MMRProof.verifyLeafProof(latestMMRRoot, leafHash, proof, proofOrder);
    }

    /**
     * @dev Helper to create an initial validator bitfield.
     * @param bitsToSet contains indexes of all signed validators, should be deduplicated
     * @param length of validator set
     */
    function createInitialBitfield(uint256[] calldata bitsToSet, uint256 length)
        external
        pure
        returns (uint256[] memory)
    {
        if (length < bitsToSet.length) {
            revert InvalidBitfieldLength();
        }
        return Bitfield.createBitfield(bitsToSet, length);
    }

    /**
     * @dev Compute the hash of a commitment
     * @param commitment the commitment to hash
     */
    function computeCommitmentHash(Commitment calldata commitment) external pure returns (bytes32) {
        return keccak256(encodeCommitment(commitment));
    }

    /**
     * @dev Helper to create a final bitfield, with subsampled validator selections
     * @param commitmentHash contains the commitmentHash signed by the validators
     * @param bitfield claiming which validators have signed the commitment
     */
    function createFinalBitfield(bytes32 commitmentHash, uint256[] calldata bitfield)
        external
        view
        returns (uint256[] memory)
    {
        Ticket storage ticket = claimedTicket(commitmentHash, bitfield);
        if (ticket.seed == 0) {
            revert PrevRandaoNotCaptured();
        }
        return Bitfield.subsample(
            ticket.seed, bitfield, ticket.validatorSetLen, ticket.numRequiredSignatures
        );
    }

    /**
     * @dev Helper to create a final bitfield with subsampled validator selections using the Fiat-Shamir approach
     * @param commitment contains the full commitment that was used for the commitmentHash
     * @param bitfield claiming which validators have signed the commitment
     */
    function createFiatShamirFinalBitfield(
        Commitment calldata commitment,
        uint256[] calldata bitfield
    ) external view returns (uint256[] memory) {
        ValidatorSetState storage vset = currentValidatorSet;
        if (commitment.validatorSetID == nextValidatorSet.id) {
            vset = nextValidatorSet;
        } else if (commitment.validatorSetID != currentValidatorSet.id) {
            revert InvalidCommitment();
        }

        if (
            bitfield.length != Bitfield.containerLength(vset.length)
                || Bitfield.countSetBits(bitfield, vset.length) < computeQuorum(vset.length)
        ) {
            revert InvalidBitfield();
        }

        bytes32 commitmentHash = keccak256(encodeCommitment(commitment));

        return fiatShamirFinalBitfield(commitmentHash, bitfield, vset);
    }

    /**
     * @dev Submit a commitment and leaf using the Fiat-Shamir approach
     * @param commitment contains the full commitment that was used for the commitmentHash
     * @param bitfield claiming which validators have signed the commitment
     * @param proofs a struct containing the data needed to verify all validator signatures
     * @param leaf an MMR leaf provable using the MMR root in the commitment payload
     * @param leafProof an MMR leaf proof
     * @param leafProofOrder a bitfield describing the order of each item (left vs right)
     */
    function submitFiatShamir(
        Commitment calldata commitment,
        uint256[] calldata bitfield,
        ValidatorProof[] calldata proofs,
        MMRLeaf calldata leaf,
        bytes32[] calldata leafProof,
        uint256 leafProofOrder
    ) external {
        if (commitment.blockNumber <= latestBeefyBlock) {
            revert StaleCommitment();
        }

        bool is_next_session = false;
        ValidatorSetState storage vset = currentValidatorSet;
        if (commitment.validatorSetID == nextValidatorSet.id) {
            is_next_session = true;
            vset = nextValidatorSet;
        } else if (commitment.validatorSetID != currentValidatorSet.id) {
            revert InvalidCommitment();
        }

        if (
            bitfield.length != Bitfield.containerLength(vset.length)
                || Bitfield.countSetBits(bitfield, vset.length) < computeQuorum(vset.length)
        ) {
            revert InvalidBitfield();
        }
        // Validate that all padding bits (beyond vset.length) are zero
        // This ensures the bitfield was created by createInitialBitfield or equivalent
        Bitfield.validatePadding(bitfield, vset.length);

        bytes32 newMMRRoot = ensureProvidesMMRRoot(commitment);

        bytes32 commitmentHash = keccak256(encodeCommitment(commitment));
        verifyFiatShamirCommitment(commitmentHash, bitfield, vset, proofs);

        if (is_next_session) {
            // The id for candidate nextValidatorSet should be greater than the current
            // nextValidatorSet id
            if (leaf.nextAuthoritySetID <= nextValidatorSet.id) {
                revert InvalidMMRLeaf();
            }
            bool leafIsValid = MMRProof.verifyLeafProof(
                newMMRRoot, keccak256(encodeMMRLeaf(leaf)), leafProof, leafProofOrder
            );
            if (!leafIsValid) {
                revert InvalidMMRLeafProof();
            }
            currentValidatorSet = nextValidatorSet;
            nextValidatorSet.id = leaf.nextAuthoritySetID;
            nextValidatorSet.length = leaf.nextAuthoritySetLen;
            nextValidatorSet.root = leaf.nextAuthoritySetRoot;
            nextValidatorSet.usageCounters = createUint16Array(leaf.nextAuthoritySetLen);
        }

        latestMMRRoot = newMMRRoot;
        latestBeefyBlock = commitment.blockNumber;

        emit NewMMRRoot(newMMRRoot, commitment.blockNumber);
    }

    /* Internal Functions */

    /**
     * @dev The caller's ticket, which must be open and for this commitment and bitfield.
     */
    function claimedTicket(bytes32 commitmentHash, uint256[] calldata bitfield)
        internal
        view
        returns (Ticket storage ticket)
    {
        ticket = tickets[msg.sender];
        if (
            ticket.blockNumber == 0
                || ticket.claimHash != computeClaimHash(commitmentHash, bitfield)
        ) {
            revert InvalidTicket();
        }
    }

    /**
     * @dev Binds a ticket to the commitment and the bitfield claimed in `submitInitial`.
     */
    function computeClaimHash(bytes32 commitmentHash, uint256[] calldata bitfield)
        internal
        pure
        returns (bytes32)
    {
        return keccak256(abi.encode(commitmentHash, keccak256(abi.encodePacked(bitfield))));
    }

    /**
     * @dev Calculates the number of required signatures for `submitFinal`.
     * @param validatorSetLen The length of the validator set
     * @param signatureUsageCount A counter of the number of times the validator signature was previously used in a call to `submitInitial` within the session.
     * @param minRequiredSignatures The minimum amount of signatures to verify
     */
    // For more details on the calculation, read the following:
    // 1. https://docs.snowbridge.network/architecture/verification/polkadot#signature-sampling
    // 2. https://hackmd.io/9OedC7icR5m-in_moUZ_WQ
    function computeNumRequiredSignatures(
        uint256 validatorSetLen,
        uint256 signatureUsageCount,
        uint256 minRequiredSignatures
    ) internal pure returns (uint256) {
        // Start with the minimum number of signatures.
        uint256 numRequiredSignatures = minRequiredSignatures;
        // Add signatures based on the number of validators in the validator set.
        numRequiredSignatures += Math.log2(validatorSetLen, Math.Rounding.Ceil);
        // Add signatures based on the signature usage count.
        numRequiredSignatures += 1 + (2 * Math.log2(signatureUsageCount, Math.Rounding.Ceil));
        // Never require more signatures than 1/3 + 1, which is sufficient to ensure at least one honest validator.
        return Math.min(numRequiredSignatures, computeMaxRequiredSignatures(validatorSetLen));
    }

    /**
     * @dev Calculates 2/3 majority required for quorum for a given number of validators.
     * @param numValidators The number of validators in the validator set.
     */
    function computeQuorum(uint256 numValidators) internal pure returns (uint256) {
        return numValidators - (numValidators - 1) / 3;
    }

    /**
     * @dev We have 2/3rd +1 honesty assumption on polkadot validators. Hence it is sufficient (for both random sampling and Fiat Shamir) to check 1/3rd +1 validator signatures to ensure at least 1 honest validator signed the payload.
     * @param numValidators The number of validators in the validator set.
     */
    function computeMaxRequiredSignatures(uint256 numValidators) internal pure returns (uint256) {
        return numValidators / 3 + 1;
    }

    /**
     * @dev Verify commitment using the supplied signature proofs
     */
    function verifyCommitment(
        bytes32 commitmentHash,
        Ticket storage ticket,
        uint256[] calldata bitfield,
        ValidatorSetState storage vset,
        ValidatorProof[] calldata proofs
    ) internal view {
        // Verify that enough signature proofs have been supplied
        uint256 numRequiredSignatures = ticket.numRequiredSignatures;
        if (proofs.length != numRequiredSignatures) {
            revert InvalidValidatorProofLength();
        }

        // Generate final bitfield indicating which validators need to be included in the proofs.
        uint256[] memory finalbitfield =
            Bitfield.subsample(ticket.seed, bitfield, vset.length, numRequiredSignatures);

        for (uint256 i = 0; i < proofs.length; i++) {
            ValidatorProof calldata proof = proofs[i];

            // Check that validator is actually in a validator set
            if (!isValidatorInSet(vset, proof.account, proof.index, proof.proof)) {
                revert InvalidValidatorProof();
            }

            // Check that validator is in bitfield
            if (!Bitfield.isSet(finalbitfield, proof.index)) {
                revert InvalidValidatorProof();
            }

            // Check that validator signed the commitment
            if (ECDSA.recover(commitmentHash, proof.v, proof.r, proof.s) != proof.account) {
                revert InvalidSignature();
            }

            // Ensure no validator can appear more than once in bitfield
            Bitfield.unset(finalbitfield, proof.index);
        }
    }

    /**
     * @dev Verify commitment with the sampled signatures using the Fiat-Shamir hash
     */
    function verifyFiatShamirCommitment(
        bytes32 commitmentHash,
        uint256[] calldata bitfield,
        ValidatorSetState storage vset,
        ValidatorProof[] calldata proofs
    ) internal view {
        uint256 requiredSignatures = Math.min(
            fiatShamirRequiredSignatures, computeMaxRequiredSignatures(vset.length)
        );
        if (proofs.length != requiredSignatures) {
            revert InvalidValidatorProofLength();
        }

        uint256[] memory finalbitfield = fiatShamirFinalBitfield(commitmentHash, bitfield, vset);

        for (uint256 i = 0; i < proofs.length; i++) {
            ValidatorProof calldata proof = proofs[i];

            // Check that validator is in bitfield
            if (!Bitfield.isSet(finalbitfield, proof.index)) {
                revert InvalidValidatorProof();
            }

            // Check that validator is actually in a validator set
            if (!isValidatorInSet(vset, proof.account, proof.index, proof.proof)) {
                revert InvalidValidatorProof();
            }

            // Check that validator signed the commitment
            if (ECDSA.recover(commitmentHash, proof.v, proof.r, proof.s) != proof.account) {
                revert InvalidSignature();
            }

            // Ensure no validator can appear more than once in bitfield
            Bitfield.unset(finalbitfield, proof.index);
        }
    }

    function createFiatShamirHash(
        bytes32 commitmentHash,
        bytes32 bitFieldHash,
        ValidatorSetState storage vset
    ) internal view returns (bytes32) {
        return sha256(
            bytes.concat(
                FIAT_SHAMIR_DOMAIN_ID,
                sha256(
                    bytes.concat(
                        commitmentHash,
                        bitFieldHash,
                        vset.root,
                        bytes32(uint256(vset.id)),
                        bytes32(uint256(vset.length))
                    )
                )
            )
        );
    }

    /**
     * @dev Helper to create a final bitfield with subsampled validator selections using the Fiat-Shamir approach
     * @param commitmentHash the hash of the full commitment that was used for the commitmentHash
     * @param bitfield claiming which validators have signed the commitment
     * @param vset the validator set state
     */
    function fiatShamirFinalBitfield(
        bytes32 commitmentHash,
        uint256[] calldata bitfield,
        ValidatorSetState storage vset
    ) internal view returns (uint256[] memory) {
        bytes32 bitFieldHash = keccak256(abi.encodePacked(bitfield));
        bytes32 fiatShamirHash = createFiatShamirHash(commitmentHash, bitFieldHash, vset);
        uint256 requiredSignatures =
            Math.min(fiatShamirRequiredSignatures, computeMaxRequiredSignatures(vset.length));
        return
            Bitfield.subsample(uint256(fiatShamirHash), bitfield, vset.length, requiredSignatures);
    }

    // Ensure that the commitment provides a new MMR root
    function ensureProvidesMMRRoot(Commitment calldata commitment)
        internal
        pure
        returns (bytes32)
    {
        for (uint256 i = 0; i < commitment.payload.length; i++) {
            if (commitment.payload[i].payloadID == MMR_ROOT_ID) {
                return bytes32(commitment.payload[i].data);
            }
        }
        revert CommitmentNotRelevant();
    }

    function encodeCommitment(Commitment calldata commitment)
        internal
        pure
        returns (bytes memory)
    {
        return bytes.concat(
            encodeCommitmentPayload(commitment.payload),
            ScaleCodec.encodeU32(commitment.blockNumber),
            ScaleCodec.encodeU64(commitment.validatorSetID)
        );
    }

    function encodeCommitmentPayload(PayloadItem[] calldata items)
        internal
        pure
        returns (bytes memory)
    {
        bytes memory payload = ScaleCodec.checkedEncodeCompactU32(items.length);
        for (uint256 i = 0; i < items.length; i++) {
            payload = bytes.concat(
                payload,
                items[i].payloadID,
                ScaleCodec.checkedEncodeCompactU32(items[i].data.length),
                items[i].data
            );
        }

        return payload;
    }

    function encodeMMRLeaf(MMRLeaf calldata leaf) internal pure returns (bytes memory) {
        return bytes.concat(
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
     * @param vset The validator set
     * @param account The address of the validator to check for inclusion in `vset`.
     * @param index The leaf index of the account in the merkle tree of validator set addresses.
     * @param proof Merkle proof required for validation of the address
     * @return true if the validator is in the set
     */
    function isValidatorInSet(
        ValidatorSetState storage vset,
        address account,
        uint256 index,
        bytes32[] calldata proof
    ) internal view returns (bool) {
        bytes32 hashedLeaf = keccak256(abi.encodePacked(account));
        return SubstrateMerkleProof.verify(vset.root, hashedLeaf, index, vset.length, proof);
    }

    /**
     * @dev Basic validation of a ticket for submitFinal
     */
    function validateTicket(
        bytes32 commitmentHash,
        Commitment calldata commitment,
        uint256[] calldata bitfield
    ) internal view returns (Ticket storage ticket) {
        // Reverts unless submitInitial opened a ticket for this commitment and bitfield
        ticket = claimedTicket(commitmentHash, bitfield);

        if (ticket.seed == 0) {
            // commitPrevRandao hasn't been called yet
            revert PrevRandaoNotCaptured();
        }

        if (commitment.blockNumber <= latestBeefyBlock) {
            // ticket is obsolete
            revert StaleCommitment();
        }
    }
}

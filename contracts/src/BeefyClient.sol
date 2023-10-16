// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.20;

import {ECDSA} from "openzeppelin/utils/cryptography/ECDSA.sol";
import {SubstrateMerkleProof} from "./utils/SubstrateMerkleProof.sol";
import {Bitfield} from "./utils/Bitfield.sol";
import {Counter} from "./utils/Counter.sol";
import {Math} from "./utils/Math.sol";
import {MMRProof} from "./utils/MMRProof.sol";
import {ScaleCodec} from "./utils/ScaleCodec.sol";

/**
 * @title BeefyClient
 *
 * High-level documentation at https://docs.snowbridge.network/architecture/verification/polkadot
 *
 * To submit new commitments signed by the current validator set, relayers must call
 * the following methods sequentially:
 * 1. submitInitial
 * 2. commitPrevRandao
 * 3. createFinalBitfield (this is just a call, not a transaction, to generate the validator subsampling)
 * 4. submitFinal (with signature proofs specified by (3))
 *
 * If the a commitment is signed by the next validator set, relayers must call
 * the following methods sequentially:
 * 1. submitInitialWithHandover
 * 2. commitPrevRandao
 * 3. createFinalBitfield (this is just a call, not a transaction, to generate the validator subsampling)
 * 4. submitFinalWithHandover (with signature proofs specified by (3))
 *
 */
contract BeefyClient {
    using Counter for uint256[];
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
     * this contract. It contains an MMR root that commits to the polkadot history, including
     * past blocks and parachain blocks and can be used to verify both polkadot and parachain blocks.
     * @param blockNumber relay chain block number
     * @param validatorSetID id of the validator set that signed the commitment
     * @param payload the payload of the new commitment in beefy justifications (in
     * our case, this is a new MMR root for all past polkadot blocks)
     */
    struct Commitment {
        uint32 blockNumber;
        uint64 validatorSetID;
        PayloadItem[] payload;
    }

    /**
     * @dev Each PayloadItem is a piece of data signed by validators at a particular block.
     * This includes the relay chain's MMR root.
     * @param payloadID an ID that references a description of the data in the payload item.
     * Known payload ids can be found [upstream](https://github.com/paritytech/substrate/blob/fe1f8ba1c4f23931ae89c1ada35efb3d908b50f5/primitives/consensus/beefy/src/payload.rs#L27).
     * @param data the contents of the payload item.
     */
    struct PayloadItem {
        bytes2 payloadID;
        bytes data;
    }

    /**
     * @dev The ValidatorProof is a proof used to verify a commitment signature
     * @param v the parity bit to specify the intended solution
     * @param r the x component on the secp256k1 curve
     * @param s the challenge solution
     * @param index index of the validator address in the merkle tree
     * @param account validator address
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
     * @dev A ticket tracks working state for the interactive submission of new commitments
     * @param sender the sender of the initial transaction
     * @param bitfield a bitfield signalling which validators they claim have signed
     * @param blockNumber the block number for this commitment
     * @param validatorSetLen the length of the validator set for this commitment
     * @param signatureCountRequired the number of signatures required
     * @param bitfield a bitfield signalling which validators they claim have signed
     */
    struct Ticket {
        address account;
        uint64 blockNumber;
        uint32 validatorSetLen;
        uint256 signatureCountRequired;
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

    // The latest verified MMRRoot and corresponding BlockNumber from the Polkadot relay chain
    bytes32 public latestMMRRoot;
    uint64 public latestBeefyBlock;

    ValidatorSet public currentValidatorSet;
    ValidatorSet public nextValidatorSet;

    uint256[] currentValidatorSetCounters;
    uint256[] nextValidatorSetCounters;

    // Currently pending tickets for commitment submission
    mapping(bytes32 => Ticket) public tickets;

    /* Constants */

    /**
     * @dev Beefy payload id for MMR Root payload items:
     * https://github.com/paritytech/substrate/blob/fe1f8ba1c4f23931ae89c1ada35efb3d908b50f5/primitives/consensus/beefy/src/payload.rs#L33
     */
    bytes2 public constant MMR_ROOT_ID = bytes2("mh");

    /**
     * @dev Minimum number of signatures required to validate a new commitment.
     */
    uint256 public immutable minimumSignatureSamples;

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
    error NoMMRRootInCommitment();
    error NotEnoughClaims();
    error PrevRandaoAlreadyCaptured();
    error PrevRandaoNotCaptured();
    error StaleCommitment();
    error TicketExpired();
    error WaitPeriodNotOver();

    constructor(
        uint256 _randaoCommitDelay,
        uint256 _randaoCommitExpiration,
        uint256 _minimumSignatureSamples,
        uint64 _initialBeefyBlock,
        ValidatorSet memory _initialValidatorSet,
        ValidatorSet memory _nextValidatorSet
    ) {
        randaoCommitDelay = _randaoCommitDelay;
        randaoCommitExpiration = _randaoCommitExpiration;
        minimumSignatureSamples = _minimumSignatureSamples;
        latestBeefyBlock = _initialBeefyBlock;
        currentValidatorSet = _initialValidatorSet;
        currentValidatorSetCounters = Counter.createCounter(currentValidatorSet.length);
        nextValidatorSet = _nextValidatorSet;
        nextValidatorSetCounters = Counter.createCounter(nextValidatorSet.length);
    }

    /* External Functions */

    /**
     * @dev Begin submission of commitment
     * @param commitment contains the commitment signed by the validators
     * @param bitfield a bitfield claiming which validators have signed the commitment
     * @param proof a proof that a single validator from currentValidatorSet has signed the commitment
     */
    function submitInitial(Commitment calldata commitment, uint256[] calldata bitfield, ValidatorProof calldata proof)
        external
    {
        ValidatorSet memory vset;
        uint16 signatureCount;
        if (commitment.validatorSetID == currentValidatorSet.id) {
            signatureCount = currentValidatorSetCounters.get(proof.index);
            currentValidatorSetCounters.set(proof.index, signatureCount + 1);
            vset = currentValidatorSet;
        } else if (commitment.validatorSetID == nextValidatorSet.id) {
            signatureCount = nextValidatorSetCounters.get(proof.index);
            nextValidatorSetCounters.set(proof.index, signatureCount + 1);
            vset = nextValidatorSet;
        } else {
            revert InvalidCommitment();
        }

        // Check if merkle proof is valid based on the validatorSetRoot and if proof is included in bitfield
        if (!isValidatorInSet(vset, proof.account, proof.index, proof.proof) || !Bitfield.isSet(bitfield, proof.index))
        {
            revert InvalidValidatorProof();
        }

        if (commitment.validatorSetID != vset.id) {
            revert InvalidCommitment();
        }

        // Check if validatorSignature is correct, ie. check if it matches
        // the signature of senderPublicKey on the commitmentHash
        bytes32 commitmentHash = keccak256(encodeCommitment(commitment));
        if (ECDSA.recover(commitmentHash, proof.v, proof.r, proof.s) != proof.account) {
            revert InvalidSignature();
        }

        // For the initial submission, the supplied bitfield should claim that more than
        // two thirds of the validator set have sign the commitment
        if (Bitfield.countSetBits(bitfield) < vset.length - (vset.length - 1) / 3) {
            revert NotEnoughClaims();
        }

        tickets[createTicketID(msg.sender, commitmentHash)] = Ticket({
            account: msg.sender,
            blockNumber: uint64(block.number),
            validatorSetLen: uint32(vset.length),
            signatureCountRequired: signatureSamples(vset.length, signatureCount),
            prevRandao: 0,
            bitfieldHash: keccak256(abi.encodePacked(bitfield))
        });
    }

    /**
     * @dev Capture PREVRANDAO
     * @param commitmentHash contains the commitmentHash signed by the validators
     */
    function commitPrevRandao(bytes32 commitmentHash) external {
        bytes32 ticketID = createTicketID(msg.sender, commitmentHash);
        Ticket storage ticket = tickets[ticketID];

        if (ticket.prevRandao != 0) {
            revert PrevRandaoAlreadyCaptured();
        }

        // relayer must wait `randaoCommitDelay` blocks
        if (block.number < ticket.blockNumber + randaoCommitDelay) {
            revert WaitPeriodNotOver();
        }

        // relayer can capture within `randaoCommitExpiration` blocks
        if (block.number > ticket.blockNumber + randaoCommitDelay + randaoCommitExpiration) {
            delete tickets[ticketID];
            revert TicketExpired();
        }

        // Post-merge, the difficulty opcode now returns PREVRANDAO
        ticket.prevRandao = block.prevrandao;
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
        (bytes32 commitmentHash, bytes32 ticketID) = validate(commitment, bitfield);

        bool is_next_session = false;
        ValidatorSet memory vset;
        if (commitment.validatorSetID == nextValidatorSet.id) {
            is_next_session = true;
            vset = nextValidatorSet;
        } else if (commitment.validatorSetID == currentValidatorSet.id) {
            vset = currentValidatorSet;
        } else {
            revert InvalidCommitment();
        }

        verifyCommitment(commitmentHash, ticketID, bitfield, vset, proofs);

        bytes32 newMMRRoot = getFirstMMRRoot(commitment);

        if (is_next_session) {
            if (leaf.nextAuthoritySetID != nextValidatorSet.id + 1) {
                revert InvalidMMRLeaf();
            }
            bool leafIsValid =
                MMRProof.verifyLeafProof(newMMRRoot, keccak256(encodeMMRLeaf(leaf)), leafProof, leafProofOrder);
            if (!leafIsValid) {
                revert InvalidMMRLeafProof();
            }
            currentValidatorSet = nextValidatorSet;
            nextValidatorSet.id = leaf.nextAuthoritySetID;
            nextValidatorSet.length = leaf.nextAuthoritySetLen;
            nextValidatorSet.root = leaf.nextAuthoritySetRoot;
            currentValidatorSetCounters = nextValidatorSetCounters;
            nextValidatorSetCounters = Counter.createCounter(leaf.nextAuthoritySetLen);
        }

        uint64 newBeefyBlock = commitment.blockNumber;
        latestMMRRoot = newMMRRoot;
        latestBeefyBlock = newBeefyBlock;
        delete tickets[ticketID];
        emit NewMMRRoot(newMMRRoot, newBeefyBlock);
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
     * @dev Helper to create a final bitfield, with subsampled validator selections
     * @param commitmentHash contains the commitmentHash signed by the validators
     * @param bitfield claiming which validators have signed the commitment
     */
    function createFinalBitfield(bytes32 commitmentHash, uint256[] calldata bitfield)
        external
        view
        returns (uint256[] memory)
    {
        Ticket storage ticket = tickets[createTicketID(msg.sender, commitmentHash)];
        if (ticket.bitfieldHash != keccak256(abi.encodePacked(bitfield))) {
            revert InvalidBitfield();
        }
        return Bitfield.subsample(ticket.prevRandao, bitfield, ticket.signatureCountRequired, ticket.validatorSetLen);
    }

    /* Internal Functions */

    // Creates a unique ticket ID for a new interactive prover-verifier session
    function createTicketID(address account, bytes32 commitmentHash) internal pure returns (bytes32 value) {
        assembly {
            mstore(0x00, account)
            mstore(0x20, commitmentHash)
            value := keccak256(0x0, 0x40)
        }
    }

    // Calculates the number of signature samples required by validator set length and the number of times a validator
    // signature was used.
    //
    // ceil(log2(validatorSetLen)) + 1 * 2 ceil(log2(signatureUseCount))
    //
    // See https://hackmd.io/9OedC7icR5m-in_moUZ_WQ for full analysis.
    function signatureSamples(uint256 validatorSetLen, uint256 signatureUseCount) internal view returns (uint256) {
        // There are less validators than the minimum signatures so validate 2/3 majority.
        if (validatorSetLen <= minimumSignatureSamples) {
            return validatorSetLen - (validatorSetLen - 1) / 3;
        }

        // Start with the minimum number of signatures.
        uint256 samples = minimumSignatureSamples;

        // We must substrate minimumSignatures from the number of validators or we might end up
        // requiring more signatures than there are validators.
        samples += Math.ceilingOfLog2(validatorSetLen - minimumSignatureSamples);

        // To address the concurrency issue specified in the link below:
        // https://hackmd.io/wsVcL0tZQA-Ks3b5KJilFQ?view#Solution-2-Signature-Checks-dynamically-depend-on-the-No-of-initial-Claims-per-session
        // It must be harder for a mallicious relayer to spam submitInitial to bias the RANDAO.
        // If we detect that a signature is used many times (spam), we increase the number of signature samples required on submitFinal.
        if (signatureUseCount > 0) {
            // Based on formula provided here: https://hackmd.io/9OedC7icR5m-in_moUZ_WQ
            samples += 1 + 2 * Math.ceilingOfLog2(signatureUseCount);
        }

        return Math.min(samples, validatorSetLen);
    }

    /**
     * @dev Verify commitment using the supplied signature proofs
     */
    function verifyCommitment(
        bytes32 commitmentHash,
        bytes32 ticketID,
        uint256[] calldata bitfield,
        ValidatorSet memory vset,
        ValidatorProof[] calldata proofs
    ) internal view {
        Ticket storage ticket = tickets[ticketID];
        // Verify that enough signature proofs have been supplied
        uint256 signatureCount = ticket.signatureCountRequired;
        if (proofs.length != signatureCount) {
            revert InvalidValidatorProof();
        }

        // Generate final bitfield indicating which validators need to be included in the proofs.
        uint256[] memory finalbitfield = Bitfield.subsample(ticket.prevRandao, bitfield, signatureCount, vset.length);

        for (uint256 i = 0; i < proofs.length;) {
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

            unchecked {
                i++;
            }
        }
    }

    function getFirstMMRRoot(Commitment calldata commitment) internal pure returns (bytes32) {
        for (uint256 i = 0; i < commitment.payload.length; i++) {
            if (commitment.payload[i].payloadID == MMR_ROOT_ID) {
                if (commitment.payload[i].data.length != 32) {
                    revert InvalidMMRRootLength();
                } else {
                    return bytes32(commitment.payload[i].data);
                }
            }
        }

        revert NoMMRRootInCommitment();
    }

    function encodeCommitment(Commitment calldata commitment) internal pure returns (bytes memory) {
        return bytes.concat(
            encodeCommitmentPayload(commitment.payload),
            ScaleCodec.encodeU32(commitment.blockNumber),
            ScaleCodec.encodeU64(commitment.validatorSetID)
        );
    }

    function encodeCommitmentPayload(PayloadItem[] calldata items) internal pure returns (bytes memory) {
        bytes memory payload = ScaleCodec.checkedEncodeCompactU32(uint32(items.length));
        for (uint256 i = 0; i < items.length; i++) {
            payload = bytes.concat(
                payload,
                items[i].payloadID,
                ScaleCodec.checkedEncodeCompactU32(uint32(items[i].data.length)),
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
    function isValidatorInSet(ValidatorSet memory vset, address account, uint256 index, bytes32[] calldata proof)
        internal
        pure
        returns (bool)
    {
        bytes32 hashedLeaf = keccak256(abi.encodePacked(account));
        return SubstrateMerkleProof.verify(vset.root, hashedLeaf, index, vset.length, proof);
    }

    // Basic checks for commitment
    function validate(Commitment calldata commitment, uint256[] calldata bitfield)
        internal
        view
        returns (bytes32, bytes32)
    {
        bytes32 commitmentHash = keccak256(encodeCommitment(commitment));
        bytes32 ticketID = createTicketID(msg.sender, commitmentHash);
        Ticket storage ticket = tickets[ticketID];

        if (ticket.blockNumber == 0) {
            // Zero value ticket: submitInitial hasn't run for this commitment
            revert InvalidTicket();
        }

        if (ticket.prevRandao == 0) {
            revert PrevRandaoNotCaptured();
        }

        if (commitment.blockNumber <= latestBeefyBlock) {
            revert StaleCommitment();
        }

        if (ticket.bitfieldHash != keccak256(abi.encodePacked(bitfield))) {
            revert InvalidBitfield();
        }
        return (commitmentHash, ticketID);
    }
}

// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.5;
pragma experimental ABIEncoderV2;

import "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";
import "@openzeppelin/contracts/access/AccessControl.sol";
import "./utils/Bits.sol";
import "./utils/Bitfield.sol";
import "./SimplifiedMMRVerification.sol";
import "./ScaleCodec.sol";
import "./utils/MerkleProof.sol";

/**
 * @title A entry contract for the Ethereum light client
 */
contract BeefyLightClient is AccessControl {
    using Bits for uint256;
    using Bitfield for uint256[];
    using ScaleCodec for uint256;
    using ScaleCodec for uint64;
    using ScaleCodec for uint32;
    using ScaleCodec for uint16;

    /* Events */

    enum Phase {
        Initial,
        Final
    }

    /**
     * @notice Verification event
     * @param id the identifier for the verification task
     * @param prover The address of the successful prover
     */
    event CommitmentVerified(
        uint256 id,
        Phase phase,
        bytes32 commitmentHash,
        address prover
    );

    event NewMMRRoot(bytes32 mmrRoot, uint64 blockNumber);

    event NewSession(
        uint256 validatorSetID,
        bytes32 validatorSetRoot,
        uint256 validatorSetLength
    );

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
        uint32 blockNumber;
        uint64 validatorSetId;
        Payload payload;
    }

    struct Payload {
        bytes32 mmrRootHash;
        bytes prefix;
        bytes suffix;
    }

    /**
     * The ValidatorProof is a collection of proofs used to verify the signatures from the validators signing
     * each new justification.
     * @param signatures an array of signatures from the randomly chosen validators
     * @param positions an array of the positions of the randomly chosen validators
     * @param publicKeys an array of the public key of each signer
     * @param publicKeyMerkleProofs an array of merkle proofs from the chosen validators proving that their public
     * keys are in the validator set
     */
    struct ValidatorProof {
        bytes[] signatures;
        uint256[] positions;
        address[] publicKeys;
        bytes32[][] publicKeyMerkleProofs;
    }

    /**
     * The ValidationData is the set of data used to link each pair of initial and complete verification transactions.
     * @param senderAddress the sender of the initial transaction
     * @param commitmentHash the hash of the commitment they are claiming has been signed
     * @param validatorClaimsBitfield a bitfield signalling which validators they claim have signed
     * @param blockNumber the block number for this commitment
     */
    struct ValidationData {
        address senderAddress;
        bytes32 commitmentHash;
        uint256 validatorSetID;
        uint256[] validatorClaimsBitfield;
        uint256 blockNumber;
    }

    /**
     * The MMRLeaf is the structure of each leaf in each MMR that each commitment's payload commits to.
     * @param version version of the leaf type
     * @param parentNumber parent number of the block this leaf describes
     * @param parentHash parent hash of the block this leaf describes
     * @param parachainHeadsRoot merkle root of all parachain headers in this block
     * @param nextAuthoritySetId validator set id that will be part of consensus for the next block
     * @param nextAuthoritySetLen length of that validator set
     * @param nextAuthoritySetRoot merkle root of all public keys in that validator set
     */
    struct MMRLeaf {
        uint8 version;
        uint32 parentNumber;
        bytes32 parentHash;
        uint64 nextAuthoritySetId;
        uint32 nextAuthoritySetLen;
        bytes32 nextAuthoritySetRoot;
        bytes32 parachainHeadsRoot;
    }

    /* State */

    bytes32 public latestMMRRoot;
    uint64 public latestBeefyBlock;

    struct ValidatorSet {
        uint256 id;
        bytes32 root;
        uint256 length;
    }

    ValidatorSet public currentValidatorSet;
    ValidatorSet public nextValidatorSet;

    uint256 public nextID;
    mapping(uint256 => ValidationData) public validationData;

    SimplifiedMMRVerification public mmrVerification;

    /* Constants */

    // Used for calculating minimum number of required signatures
    uint256 public constant THRESHOLD_NUMERATOR = 3;
    uint256 public constant THRESHOLD_DENOMINATOR = 250;

    uint64 public constant BLOCK_WAIT_PERIOD = 3;

    /**
     * @notice Deploys the BeefyLightClient contract
     * @param _mmrVerification The contract to be used for MMR verification
     */
    constructor(SimplifiedMMRVerification _mmrVerification) {
        _setupRole(DEFAULT_ADMIN_ROLE, msg.sender);
        mmrVerification = _mmrVerification;
        nextID = 0;
    }

    // Once-off post-construction call to set initial configuration.
    function initialize(
        uint64 _startingBeefyBlock,
        ValidatorSet calldata _initialValidatorSet,
        ValidatorSet calldata _nextValidatorSet
    ) external onlyRole(DEFAULT_ADMIN_ROLE) {
        latestBeefyBlock = _startingBeefyBlock;
        currentValidatorSet = _initialValidatorSet;
        nextValidatorSet = _nextValidatorSet;

        // drop admin privileges
        renounceRole(DEFAULT_ADMIN_ROLE, msg.sender);
    }

    /* Public Functions */

    /**
     * @notice Executed by the incoming channel in order to verify commitment
     * @param beefyMMRLeaf contains the merkle leaf to be verified
     * @param proof contains simplified mmr proof
     */
    function verifyBeefyMerkleLeaf(
        bytes32 beefyMMRLeaf,
        SimplifiedMMRProof memory proof
    ) external view returns (bool) {
        return
            mmrVerification.verifyInclusionProof(
                latestMMRRoot,
                beefyMMRLeaf,
                proof
            );
    }

    /**
     * @notice Executed by the prover in order to begin the process of block
     * acceptance by the light client
     * @param commitmentHash contains the commitmentHash signed by the validator(s)
     * @param validatorClaimsBitfield a bitfield containing a membership status of each
     * validator who has claimed to have signed the commitmentHash
     * @param validatorSignature the signature of one validator
     * @param validatorPosition the position of the validator, index starting at 0
     * @param validatorPublicKey the public key of the validator
     * @param validatorPublicKeyMerkleProof proof required for validation of the public key in the validator merkle tree
     */
    function newSignatureCommitment(
        bytes32 commitmentHash,
        uint64 validatorSetID,
        uint256[] memory validatorClaimsBitfield,
        bytes memory validatorSignature,
        uint256 validatorPosition,
        address validatorPublicKey,
        bytes32[] calldata validatorPublicKeyMerkleProof
    ) public payable {
        ValidatorSet memory vset = getValidatorSet(validatorSetID);

        // Check if validatorPublicKeyMerkleProof is valid based on validatorSetRoot
        require(
            isValidatorInSet(
                vset,
                validatorPublicKey,
                validatorPosition,
                validatorPublicKeyMerkleProof
            ),
            "Sender must be in validator set at correct position"
        );

        // Check if validatorSignature is correct, ie. check if it matches
        // the signature of senderPublicKey on the commitmentHash
        require(
            ECDSA.recover(commitmentHash, validatorSignature) ==
                validatorPublicKey,
            "Invalid Signature"
        );

        // Check that the bitfield actually contains enough claims to be successful, ie, >= 2/3
        require(
            validatorClaimsBitfield.countSetBits() >=
                requiredNumberOfSignatures(vset),
            "Not enough claims in bitfield"
        );

        // Accept and save the commitment
        validationData[nextID] = ValidationData(
            msg.sender,
            commitmentHash,
            vset.id,
            validatorClaimsBitfield,
            block.number
        );

        emit CommitmentVerified(
            nextID,
            Phase.Initial,
            commitmentHash,
            msg.sender
        );

        nextID = nextID + 1;
    }

    function createRandomBitfield(uint256 id)
        public
        view
        returns (uint256[] memory)
    {
        ValidationData storage data = validationData[id];

        ValidatorSet memory vset = getValidatorSet(data.validatorSetID);

        // verify that block wait period has passed
        require(
            block.number >= data.blockNumber + BLOCK_WAIT_PERIOD,
            "Block wait period not over"
        );

        return
            Bitfield.randomNBitsWithPriorCheck(
                getSeed(data),
                data.validatorClaimsBitfield,
                requiredNumberOfSignatures(vset),
                vset.length
            );
    }

    function createInitialBitfield(uint256[] calldata bitsToSet, uint256 length)
        public
        pure
        returns (uint256[] memory)
    {
        return Bitfield.createBitfield(bitsToSet, length);
    }

    /**
     * @notice Performs the final step in the validation, and then applies the commitment and optional authority update
     * @param id an identifying value generated in the previous transaction
     * @param commitment contains the full commitment that was used for the commitmentHash
     * @param proof a struct containing the data needed to verify all validator signatures
     */
    function completeSignatureCommitment(
        uint256 id,
        Commitment calldata commitment,
        ValidatorProof calldata proof
    ) public {
        ValidationData storage data = validationData[id];

        bytes32 commitmentHash = verifyCommitment(data, commitment, proof);

        latestMMRRoot = commitment.payload.mmrRootHash;
        latestBeefyBlock = commitment.blockNumber;
        emit NewMMRRoot(commitment.payload.mmrRootHash, commitment.blockNumber);

        // Check if commitment signals an authority handover (new validator session)
        if (commitment.validatorSetId == nextValidatorSet.id) {
            // Handover to the next authority set
            currentValidatorSet = nextValidatorSet;
            emit NewSession(
                nextValidatorSet.id,
                nextValidatorSet.root,
                nextValidatorSet.length
            );
        }

        // Obtain a gas refund
        delete validationData[id];

        emit CommitmentVerified(id, Phase.Final, commitmentHash, msg.sender);
    }

    function updateValidatorSet(
        MMRLeaf calldata leaf,
        SimplifiedMMRProof calldata proof
    ) public {
        require(
            leaf.nextAuthoritySetId == nextValidatorSet.id + 1,
            "Leaf is invalid"
        );

        // Verify that the leaf suppied by the relayer is part of the MMR
        bytes32 leafHash = keccak256(encodeMMRLeaf(leaf));
        require(
            mmrVerification.verifyInclusionProof(
                latestMMRRoot,
                leafHash,
                proof
            ),
            "Invalid leaf proof"
        );

        nextValidatorSet.id = leaf.nextAuthoritySetId;
        nextValidatorSet.root = leaf.nextAuthoritySetRoot;
        nextValidatorSet.length = leaf.nextAuthoritySetLen;
    }

    /* Private Functions */

    function getValidatorSet(uint256 id)
        internal
        view
        returns (ValidatorSet memory)
    {
        if (id == currentValidatorSet.id) {
            return currentValidatorSet;
        } else if (id == nextValidatorSet.id) {
            return nextValidatorSet;
        } else {
            revert("unknown validator set");
        }
    }

    /**
     * @notice Deterministically generates a seed from the block hash at the block number of creation of the validation
     * data plus MAXIMUM_NUM_SIGNERS
     * @dev Note that `blockhash(blockNum)` will only work for the 256 most recent blocks. If
     * `completeSignatureCommitment` is called too late, a new call to `newSignatureCommitment` is necessary to reset
     * validation data's block number
     * @param data a storage reference to the validationData struct
     * @return onChainRandNums an array storing the random numbers generated inside this function
     */
    function getSeed(ValidationData storage data)
        private
        view
        returns (uint256)
    {
        // @note Get payload.blocknumber, add BLOCK_WAIT_PERIOD
        uint256 randomSeedBlockNum = data.blockNumber + BLOCK_WAIT_PERIOD;
        // @note Create a hash seed from the block number
        bytes32 randomSeedBlockHash = blockhash(randomSeedBlockNum);

        return uint256(randomSeedBlockHash);
    }

    function requiredNumberOfSignatures(ValidatorSet memory vset)
        internal
        view
        returns (uint256)
    {
        return
            (vset.length * THRESHOLD_NUMERATOR + THRESHOLD_DENOMINATOR - 1) /
            THRESHOLD_DENOMINATOR;
    }

    function verifyCommitment(
        ValidationData storage data,
        Commitment calldata commitment,
        ValidatorProof calldata proof
    ) internal view returns (bytes32) {
        // Verify that sender is the same as in `newSignatureCommitment`
        require(
            msg.sender == data.senderAddress,
            "Sender address does not match original validation data"
        );

        // Verify that block wait period has passed
        require(
            block.number >= data.blockNumber + BLOCK_WAIT_PERIOD,
            "Block wait period not over"
        );

        // Check that payload.leaf.block_number is > last_known_block_number;
        require(
            commitment.blockNumber > latestBeefyBlock,
            "Commitment blocknumber is too old"
        );

        ValidatorSet memory vset = getValidatorSet(commitment.validatorSetId);

        uint256 requiredNumOfSignatures = requiredNumberOfSignatures(vset);

        require(
            proof.signatures.length == requiredNumOfSignatures &&
                proof.positions.length == requiredNumOfSignatures &&
                proof.publicKeys.length == requiredNumOfSignatures &&
                proof.publicKeyMerkleProofs.length == requiredNumOfSignatures,
            "Number of signatures does not match required"
        );

        uint256[] memory randomBitfield = Bitfield.randomNBitsWithPriorCheck(
            getSeed(data),
            data.validatorClaimsBitfield,
            requiredNumOfSignatures,
            vset.length
        );

        bytes32 commitmentHash = keccak256(encodeCommitment(commitment));

        // Validate signatures
        for (uint256 i = 0; i < requiredNumOfSignatures; i++) {
            verifyValidatorSignature(
                vset,
                randomBitfield,
                proof.signatures[i],
                proof.positions[i],
                proof.publicKeys[i],
                proof.publicKeyMerkleProofs[i],
                commitmentHash
            );
        }

        return commitmentHash;
    }

    function verifyValidatorSignature(
        ValidatorSet memory vset,
        uint256[] memory randomBitfield,
        bytes calldata signature,
        uint256 position,
        address publicKey,
        bytes32[] calldata publicKeyMerkleProof,
        bytes32 commitmentHash
    ) internal view {
        // Check if validator in randomBitfield
        require(
            randomBitfield.isSet(position),
            "Validator must be once in bitfield"
        );

        // Remove validator from randomBitfield such that no validator can appear twice in signatures
        randomBitfield.clear(position);

        // Check if merkle proof is valid
        require(
            isValidatorInSet(vset, publicKey, position, publicKeyMerkleProof),
            "Validator must be in validator set at correct position"
        );

        // Check if signature is correct
        require(
            ECDSA.recover(commitmentHash, signature) == publicKey,
            "Invalid Signature"
        );
    }

    function encodeCommitment(Commitment calldata commitment)
        internal
        pure
        returns (bytes memory)
    {
        return
            bytes.concat(
                commitment.payload.prefix,
                commitment.payload.mmrRootHash,
                commitment.payload.suffix,
                commitment.blockNumber.encode32(),
                commitment.validatorSetId.encode64()
            );
    }

    function encodeMMRLeaf(MMRLeaf calldata leaf)
        internal
        pure
        returns (bytes memory)
    {
        return
            bytes.concat(
                ScaleCodec.encode8(leaf.version),
                ScaleCodec.encode32(leaf.parentNumber),
                leaf.parentHash,
                ScaleCodec.encode64(leaf.nextAuthoritySetId),
                ScaleCodec.encode32(leaf.nextAuthoritySetLen),
                leaf.nextAuthoritySetRoot,
                leaf.parachainHeadsRoot
            );
    }

    /**
     * @notice Checks if a validators address is a member of the merkle tree
     * @param addr The address of the validator to check
     * @param pos The position of the validator to check, index starting at 0
     * @param proof Merkle proof required for validation of the address
     * @return true if the validator is in the set
     */
    function isValidatorInSet(
        ValidatorSet memory vset,
        address addr,
        uint256 pos,
        bytes32[] memory proof
    ) internal view returns (bool) {
        bytes32 hashedLeaf = keccak256(abi.encodePacked(addr));
        return
            MerkleProof.verifyMerkleLeafAtPosition(
                vset.root,
                hashedLeaf,
                pos,
                vset.length,
                proof
            );
    }
}

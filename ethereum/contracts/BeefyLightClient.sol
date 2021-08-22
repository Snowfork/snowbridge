// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.5;
pragma experimental ABIEncoderV2;

import "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";
import "./utils/Bits.sol";
import "./utils/Bitfield.sol";
import "./ValidatorRegistry.sol";
import "./SimplifiedMMRVerification.sol";
import "./ScaleCodec.sol";

/**
 * @title A entry contract for the Ethereum light client
 */
contract BeefyLightClient {
    using Bits for uint256;
    using Bitfield for uint256[];
    using ScaleCodec for uint256;
    using ScaleCodec for uint64;
    using ScaleCodec for uint32;
    using ScaleCodec for uint16;

    /* Events */

    /**
     * @notice Notifies an observer that the prover's attempt at initital
     * verification was successful.
     * @dev Note that the prover must wait until `n` blocks have been mined
     * subsequent to the generation of this event before the 2nd tx can be sent
     * @param prover The address of the calling prover
     * @param blockNumber The blocknumber in which the initial validation
     * succeeded
     * @param id An identifier to provide disambiguation
     */
    event InitialVerificationSuccessful(
        address prover,
        uint256 blockNumber,
        uint256 id
    );

    /**
     * @notice Notifies an observer that the complete verification process has
     *  finished successfuly and the new commitmentHash will be accepted
     * @param prover The address of the successful prover
     * @param id the identifier used
     */
    event FinalVerificationSuccessful(address prover, uint256 id);

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
        uint256[] validatorClaimsBitfield;
        uint256 blockNumber;
    }

    /**
     * The BeefyMMRLeaf is the structure of each leaf in each MMR that each commitment's payload commits to.
     * @param version version of the leaf type
     * @param parentNumber parent number of the block this leaf describes
     * @param parentHash parent hash of the block this leaf describes
     * @param parachainHeadsRoot merkle root of all parachain headers in this block
     * @param nextAuthoritySetId validator set id that will be part of consensus for the next block
     * @param nextAuthoritySetLen length of that validator set
     * @param nextAuthoritySetRoot merkle root of all public keys in that validator set
     */
    struct BeefyMMRLeaf {
        uint8 version;
        uint32 parentNumber;
        bytes32 parentHash;
        bytes32 parachainHeadsRoot;
        uint64 nextAuthoritySetId;
        uint32 nextAuthoritySetLen;
        bytes32 nextAuthoritySetRoot;
    }

    /* State */

    ValidatorRegistry public validatorRegistry;
    SimplifiedMMRVerification public mmrVerification;
    uint256 public currentId;
    bytes32 public latestMMRRoot;
    uint64 public latestBeefyBlock;
    mapping(uint256 => ValidationData) public validationData;

    /* Constants */

    // THRESHOLD_NUMERATOR - numerator for percent of validator signatures required
    // THRESHOLD_DENOMINATOR - denominator for percent of validator signatures required
    uint256 public constant THRESHOLD_NUMERATOR = 3;
    uint256 public constant THRESHOLD_DENOMINATOR = 250;
    uint64 public constant BLOCK_WAIT_PERIOD = 3;

    // We must ensure at least one block is processed every session,
    // so these constants are checked to enforce a maximum gap between commitments.
    uint64 public constant NUMBER_OF_BLOCKS_PER_SESSION = 2400;
    uint64 public constant ERROR_AND_SAFETY_BUFFER = 10;
    uint64 public constant MAXIMUM_BLOCK_GAP =
        NUMBER_OF_BLOCKS_PER_SESSION - ERROR_AND_SAFETY_BUFFER;

    /**
     * @notice Deploys the BeefyLightClient contract
     * @param _validatorRegistry The contract to be used as the validator registry
     * @param _mmrVerification The contract to be used for MMR verification
     */
    constructor(
        ValidatorRegistry _validatorRegistry,
        SimplifiedMMRVerification _mmrVerification,
        uint64 _startingBeefyBlock
    ) {
        validatorRegistry = _validatorRegistry;
        mmrVerification = _mmrVerification;
        currentId = 0;
        latestBeefyBlock = _startingBeefyBlock;
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
        uint256[] memory validatorClaimsBitfield,
        bytes memory validatorSignature,
        uint256 validatorPosition,
        address validatorPublicKey,
        bytes32[] calldata validatorPublicKeyMerkleProof
    ) public payable {
        /**
         * @dev Check if validatorPublicKeyMerkleProof is valid based on ValidatorRegistry merkle root
         */
        require(
            validatorRegistry.checkValidatorInSet(
                validatorPublicKey,
                validatorPosition,
                validatorPublicKeyMerkleProof
            ),
            "Error: Sender must be in validator set at correct position"
        );

        /**
         * @dev Check if validatorSignature is correct, ie. check if it matches
         * the signature of senderPublicKey on the commitmentHash
         */
        require(
            ECDSA.recover(commitmentHash, validatorSignature) ==
                validatorPublicKey,
            "Error: Invalid Signature"
        );

        /**
         * @dev Check that the bitfield actually contains enough claims to be succesful, ie, >= 2/3
         */
        require(
            validatorClaimsBitfield.countSetBits() >=
                requiredNumberOfSignatures(),
            "Error: Bitfield not enough validators"
        );

        // Accept and save the commitment
        validationData[currentId] = ValidationData(
            msg.sender,
            commitmentHash,
            validatorClaimsBitfield,
            block.number
        );

        emit InitialVerificationSuccessful(msg.sender, block.number, currentId);

        currentId = currentId + 1;
    }

    function createRandomBitfield(uint256 id)
        public
        view
        returns (uint256[] memory)
    {
        ValidationData storage data = validationData[id];

        /**
         * @dev verify that block wait period has passed
         */
        require(
            block.number >= data.blockNumber + BLOCK_WAIT_PERIOD,
            "Error: Block wait period not over"
        );

        uint256 numberOfValidators = validatorRegistry.numOfValidators();

        return
            Bitfield.randomNBitsWithPriorCheck(
                getSeed(data),
                data.validatorClaimsBitfield,
                requiredNumberOfSignatures(numberOfValidators),
                numberOfValidators
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
     * @notice Performs the second step in the validation logic
     * @param id an identifying value generated in the previous transaction
     * @param commitment contains the full commitment that was used for the commitmentHash
     * @param validatorProof a struct containing the data needed to verify all validator signatures
     */
    function completeSignatureCommitment(
        uint256 id,
        Commitment calldata commitment,
        ValidatorProof calldata validatorProof,
        BeefyMMRLeaf calldata latestMMRLeaf,
        SimplifiedMMRProof calldata proof
    ) public {
        verifyCommitment(id, commitment, validatorProof);
        verifyNewestMMRLeaf(
            latestMMRLeaf,
            commitment.payload,
            proof
        );

        processPayload(commitment.payload, commitment.blockNumber);

        applyValidatorSetChanges(
            latestMMRLeaf.nextAuthoritySetId,
            latestMMRLeaf.nextAuthoritySetLen,
            latestMMRLeaf.nextAuthoritySetRoot
        );

        emit FinalVerificationSuccessful(msg.sender, id);

        /**
         * @dev We no longer need the data held in state, so delete it for a gas refund
         */
        delete validationData[id];
    }

    /* Private Functions */

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

    function verifyNewestMMRLeaf(
        BeefyMMRLeaf calldata leaf,
        bytes32 root,
        SimplifiedMMRProof calldata proof
    ) public view {
        bytes memory encodedLeaf = encodeMMRLeaf(leaf);
        bytes32 hashedLeaf = hashMMRLeaf(encodedLeaf);

        mmrVerification.verifyInclusionProof(
            root,
            hashedLeaf,
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
        if (nextAuthoritySetId != validatorRegistry.id()) {
            validatorRegistry.update(
                nextAuthoritySetRoot,
                nextAuthoritySetLen,
                nextAuthoritySetId
            );
        }
    }

    function requiredNumberOfSignatures() public view returns (uint256) {
        return
            (validatorRegistry.numOfValidators() *
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
        uint256 id,
        Commitment calldata commitment,
        ValidatorProof calldata proof
    ) internal view {
        ValidationData storage data = validationData[id];

        // Verify that sender is the same as in `newSignatureCommitment`
        require(
            msg.sender == data.senderAddress,
            "Error: Sender address does not match original validation data"
        );

        uint256 numberOfValidators = validatorRegistry.numOfValidators();
        uint256 requiredNumOfSignatures = requiredNumberOfSignatures(
            numberOfValidators
        );

        /**
         * @dev verify that block wait period has passed
         */
        require(
            block.number >= data.blockNumber + BLOCK_WAIT_PERIOD,
            "Error: Block wait period not over"
        );

        uint256[] memory randomBitfield = Bitfield.randomNBitsWithPriorCheck(
            getSeed(data),
            data.validatorClaimsBitfield,
            requiredNumOfSignatures,
            numberOfValidators
        );

        verifyValidatorProofLengths(requiredNumOfSignatures, proof);

        verifyValidatorProofSignatures(
            randomBitfield,
            proof,
            requiredNumOfSignatures,
            commitment
        );
    }

    function verifyValidatorProofLengths(
        uint256 requiredNumOfSignatures,
        ValidatorProof calldata proof
    ) internal pure {
        /**
         * @dev verify that required number of signatures, positions, public keys and merkle proofs are
         * submitted
         */
        require(
            proof.signatures.length == requiredNumOfSignatures,
            "Error: Number of signatures does not match required"
        );
        require(
            proof.positions.length == requiredNumOfSignatures,
            "Error: Number of validator positions does not match required"
        );
        require(
            proof.publicKeys.length == requiredNumOfSignatures,
            "Error: Number of validator public keys does not match required"
        );
        require(
            proof.publicKeyMerkleProofs.length == requiredNumOfSignatures,
            "Error: Number of validator public keys does not match required"
        );
    }

    function verifyValidatorProofSignatures(
        uint256[] memory randomBitfield,
        ValidatorProof calldata proof,
        uint256 requiredNumOfSignatures,
        Commitment calldata commitment
    ) internal view {
        // Encode and hash the commitment
        bytes32 commitmentHash = createCommitmentHash(commitment);

        /**
         *  @dev For each randomSignature, do:
         */
        for (uint256 i = 0; i < requiredNumOfSignatures; i++) {
            verifyValidatorSignature(
                randomBitfield,
                proof.signatures[i],
                proof.positions[i],
                proof.publicKeys[i],
                proof.publicKeyMerkleProofs[i],
                commitmentHash
            );
        }
    }

    function verifyValidatorSignature(
        uint256[] memory randomBitfield,
        bytes calldata signature,
        uint256 position,
        address publicKey,
        bytes32[] calldata publicKeyMerkleProof,
        bytes32 commitmentHash
    ) internal view {
        /**
         * @dev Check if validator in randomBitfield
         */
        require(
            randomBitfield.isSet(position),
            "Error: Validator must be once in bitfield"
        );

        /**
         * @dev Remove validator from randomBitfield such that no validator can appear twice in signatures
         */
        randomBitfield.clear(position);

        /**
         * @dev Check if merkle proof is valid
         */
        require(
            validatorRegistry.checkValidatorInSet(
                publicKey,
                position,
                publicKeyMerkleProof
            ),
            "Error: Validator must be in validator set at correct position"
        );

        /**
         * @dev Check if signature is correct
         */
        require(
            ECDSA.recover(commitmentHash, signature) == publicKey,
            "Error: Invalid Signature"
        );
    }

    function createCommitmentHash(Commitment calldata commitment)
        public
        pure
        returns (bytes32)
    {
        return
            keccak256(
                abi.encodePacked(
                    commitment.payload,
                    commitment.blockNumber.encode64(),
                    commitment.validatorSetId.encode32()
                )
            );
    }

    // To scale encode the byte array, we need to prefix it
    // with it's length. This is the expected current length of a leaf.
    // The length here is 113 bytes:
    // - 1 byte for the version
    // - 4 bytes for the block number
    // - 32 bytes for the block hash
    // - 8 bytes for the next validator set ID
    // - 4 bytes for the length of it
    // - 32 bytes for the root hash of it
    // - 32 bytes for the parachain heads merkle root
    // That number is then compact encoded unsigned integer - see SCALE spec
    bytes2 public constant MMR_LEAF_LENGTH_SCALE_ENCODED =
        bytes2(uint16(0xc501));

    function encodeMMRLeaf(BeefyMMRLeaf calldata leaf)
        public
        pure
        returns (bytes memory)
    {
        bytes memory scaleEncodedMMRLeaf = abi.encodePacked(
            ScaleCodec.encode8(leaf.version),
            ScaleCodec.encode32(leaf.parentNumber),
            leaf.parentHash,
            ScaleCodec.encode64(leaf.nextAuthoritySetId),
            ScaleCodec.encode32(leaf.nextAuthoritySetLen),
            leaf.nextAuthoritySetRoot,
            leaf.parachainHeadsRoot
        );

        return bytes.concat(MMR_LEAF_LENGTH_SCALE_ENCODED, scaleEncodedMMRLeaf);
    }

    function hashMMRLeaf(bytes memory leaf) public pure returns (bytes32) {
        return keccak256(leaf);
    }
}

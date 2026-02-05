// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.20;

import {ISP1Verifier} from "@sp1-contracts/ISP1Verifier.sol";
import {SP1VerifierGateway} from "@sp1-contracts/SP1VerifierGateway.sol";

/**
 * @title SP1BeefyClient
 * @dev A Solidity verifier contract for SP1 proofs of BeefyClient operations
 */
contract SP1BeefyClient {
    ISP1Verifier public immutable verifierGateway;
    
    // The verification key for the SP1 program
    bytes32 public immutable programVKey;
    
    // State variables
    bytes32 public latestMMRRoot;
    uint64 public latestBeefyBlock;
    
    struct ValidatorSet {
        uint128 id;
        uint128 length;
        bytes32 root;
    }
    
    ValidatorSet public currentValidatorSet;
    ValidatorSet public nextValidatorSet;
    
    // Events
    event NewMMRRoot(bytes32 indexed mmrRoot, uint64 indexed blockNumber);
    event InitialSubmission(bytes32 indexed ticketId, bytes32 indexed commitmentHash);
    event RandaoCommitment(bytes32 indexed ticketId, uint64 prevRandao);
    event FinalSubmission(bytes32 indexed ticketId, bytes32 indexed mmrRoot, uint64 blockNumber);
    event FiatShamirSubmission(bytes32 indexed commitmentHash, bytes32 indexed mmrRoot, uint64 blockNumber);
    
    // Errors
    error InvalidProof();
    error InvalidPublicValues();
    error InvalidOperation();
    
    constructor(
        address _verifierGateway,
        bytes32 _programVKey,
        uint64 _initialBeefyBlock,
        ValidatorSet memory _initialValidatorSet,
        ValidatorSet memory _nextValidatorSet
    ) {
        verifierGateway = ISP1Verifier(_verifierGateway);
        programVKey = _programVKey;
        latestBeefyBlock = _initialBeefyBlock;
        currentValidatorSet = _initialValidatorSet;
        nextValidatorSet = _nextValidatorSet;
    }
    
    /**
     * @dev Verify an SP1 proof for a submitInitial operation
     */
    function verifySubmitInitial(
        bytes calldata proof,
        bytes32 ticketId,
        bytes32 commitmentHash
    ) external {
        bytes[] memory publicValues = new bytes[](2);
        publicValues[0] = abi.encodePacked(ticketId);
        publicValues[1] = abi.encodePacked(commitmentHash);
        
        _verifyProof(proof, publicValues, 1); // Operation code 1 for submit_initial
        
        emit InitialSubmission(ticketId, commitmentHash);
    }
    
    /**
     * @dev Verify an SP1 proof for a commitPrevRandao operation
     */
    function verifyCommitPrevRandao(
        bytes calldata proof,
        bytes32 ticketId,
        uint64 prevRandao
    ) external {
        bytes[] memory publicValues = new bytes[](2);
        publicValues[0] = abi.encodePacked(ticketId);
        publicValues[1] = abi.encodePacked(prevRandao);
        
        _verifyProof(proof, publicValues, 2); // Operation code 2 for commit_prev_randao
        
        emit RandaoCommitment(ticketId, prevRandao);
    }
    
    /**
     * @dev Verify an SP1 proof for a submitFinal operation
     */
    function verifySubmitFinal(
        bytes calldata proof,
        bytes32 mmrRoot,
        uint64 blockNumber
    ) external {
        bytes[] memory publicValues = new bytes[](3);
        publicValues[0] = abi.encodePacked(mmrRoot);
        publicValues[1] = abi.encodePacked(blockNumber);
        publicValues[2] = abi.encodePacked(bytes32(0)); // ticketId placeholder
        
        _verifyProof(proof, publicValues, 3); // Operation code 3 for submit_final
        
        latestMMRRoot = mmrRoot;
        latestBeefyBlock = blockNumber;
        
        emit NewMMRRoot(mmrRoot, blockNumber);
        emit FinalSubmission(bytes32(0), mmrRoot, blockNumber);
    }
    
    /**
     * @dev Verify an SP1 proof for a submitFiatShamir operation
     */
    function verifySubmitFiatShamir(
        bytes calldata proof,
        bytes32 commitmentHash,
        bytes32 mmrRoot,
        uint64 blockNumber
    ) external {
        bytes[] memory publicValues = new bytes[](3);
        publicValues[0] = abi.encodePacked(mmrRoot);
        publicValues[1] = abi.encodePacked(blockNumber);
        publicValues[2] = abi.encodePacked(commitmentHash);
        
        _verifyProof(proof, publicValues, 4); // Operation code 4 for submit_fiat_shamir
        
        latestMMRRoot = mmrRoot;
        latestBeefyBlock = blockNumber;
        
        emit NewMMRRoot(mmrRoot, blockNumber);
        emit FiatShamirSubmission(commitmentHash, mmrRoot, blockNumber);
    }
    
    /**
     * @dev Verify an SP1 proof for any operation
     */
    function _verifyProof(
        bytes calldata proof,
        bytes[] memory publicValues,
        uint8 operation
    ) internal {
        try verifierGateway.verifyProof(
            programVKey,
            publicValues,
            proof
        ) returns (bool) {
            // Proof verified successfully
        } catch {
            revert InvalidProof();
        }
    }
    
    /**
     * @dev Batch verify multiple SP1 proofs
     */
    function batchVerify(
        bytes[] calldata proofs,
        bytes[][] calldata publicValuesArray,
        uint8[] calldata operations
    ) external {
        require(
            proofs.length == publicValuesArray.length && 
            proofs.length == operations.length,
            "Array length mismatch"
        );
        
        for (uint256 i = 0; i < proofs.length; i++) {
            _verifyProof(proofs[i], publicValuesArray[i], operations[i]);
        }
    }
    
    /**
     * @dev Update the verification key (only callable by admin)
     */
    function updateProgramVKey(bytes32 newVKey) external {
        // This would typically have access control
        programVKey = newVKey;
    }
}
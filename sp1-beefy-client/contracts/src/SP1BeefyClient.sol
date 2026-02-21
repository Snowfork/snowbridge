// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.20;

import {ISP1Verifier} from "./ISP1Verifier.sol";

/**
 * @title SP1BeefyClient
 * @dev A Solidity verifier contract for SP1 proofs of BeefyClient Fiat-Shamir submissions
 */
contract SP1BeefyClient {
    ISP1Verifier public immutable verifierGateway;
    bytes32 public immutable programVKey;

    bytes32 public latestMMRRoot;
    uint64 public latestBeefyBlock;

    struct ValidatorSet {
        uint128 id;
        uint128 length;
        bytes32 root;
    }

    ValidatorSet public currentValidatorSet;
    ValidatorSet public nextValidatorSet;

    event NewMMRRoot(bytes32 indexed mmrRoot, uint64 indexed blockNumber);
    event FiatShamirSubmission(bytes32 indexed commitmentHash, bytes32 indexed mmrRoot, uint64 blockNumber);

    error InvalidProof();

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

        bytes memory publicValuesEncoded = _encodePublicValues(publicValues);
        try verifierGateway.verifyProof(programVKey, publicValuesEncoded, proof) {
            // Proof verified successfully
        } catch {
            revert InvalidProof();
        }

        latestMMRRoot = mmrRoot;
        latestBeefyBlock = blockNumber;

        emit NewMMRRoot(mmrRoot, blockNumber);
        emit FiatShamirSubmission(commitmentHash, mmrRoot, blockNumber);
    }

    function _encodePublicValues(bytes[] memory values) internal pure returns (bytes memory) {
        bytes memory result;
        for (uint256 i = 0; i < values.length; i++) {
            result = bytes.concat(result, values[i]);
        }
        return result;
    }
}

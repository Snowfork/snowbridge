// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.5;
pragma experimental ABIEncoderV2;

import "../utils/MMRProofVerification.sol";

contract MMRProofVerifier {
    function verifyLeafProof(
        bytes32 root,
        bytes32 leafNodeHash,
        MMRProof memory proof
    ) public pure returns (bool) {
        return MMRProofVerification.verifyLeafProof(root, leafNodeHash, proof);
    }
}

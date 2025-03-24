// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {ParachainVerification} from "../../src/ParachainVerification.sol";

contract VerificationWrapper {
    function createParachainHeaderMerkleLeaf(
        bytes4 encodedParachainID,
        ParachainVerification.ParachainHeader calldata header
    ) external pure returns (bytes32) {
        return ParachainVerification.createParachainHeaderMerkleLeaf(encodedParachainID, header);
    }

    function isCommitmentInHeaderDigest(bytes32 commitment, ParachainVerification.ParachainHeader calldata header)
        external
        pure
        returns (bool)
    {
        return ParachainVerification.isCommitmentInHeaderDigest(commitment, header);
    }
}

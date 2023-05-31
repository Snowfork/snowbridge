// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import "../../src/IParachainClient.sol";
import "../../src/ParachainClient.sol";
import "../../src/BeefyClient.sol";

contract ParachainClientMock is ParachainClient {
    constructor(BeefyClient _client, uint32 _parachainID) ParachainClient(_client, _parachainID) {}

    function verifyCommitment(bytes32, bytes calldata parachainHeaderProof) external pure override returns (bool) {
        if (keccak256(parachainHeaderProof) == keccak256(bytes("validProof"))) {
            return true;
        } else {
            return false;
        }
    }

    function createParachainHeaderMerkleLeaf_public(ParachainHeader memory header) external view returns (bytes32) {
        return createParachainHeaderMerkleLeaf(header);
    }

    function isCommitmentInHeaderDigest_public(bytes32 commitment, ParachainHeader memory header)
        external
        pure
        returns (bool)
    {
        return isCommitmentInHeaderDigest(commitment, header);
    }
}

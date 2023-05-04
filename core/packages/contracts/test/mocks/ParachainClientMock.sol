// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import "../../src/IParachainClient.sol";

contract ParachainClientMock is IParachainClient {
    function verifyCommitment(bytes32, Proof calldata parachainHeaderProof) external pure override returns (bool) {
        IParachainClient.Proof memory mockProof = IParachainClient.Proof(
            new bytes(0),
            new bytes(0),
            IParachainClient.HeadProof(0, 0, new bytes32[](0)),
            IParachainClient.MMRLeafPartial(0, 0, bytes32(0), 0, 0, bytes32(0)),
            new bytes32[](0),
            0
        );

        if (keccak256(abi.encode(parachainHeaderProof)) == keccak256(abi.encode(mockProof))) {
            return true;
        } else {
            return false;
        }
    }
}

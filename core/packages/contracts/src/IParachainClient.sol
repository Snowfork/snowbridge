// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

interface IParachainClient {
    struct HeadProof {
        uint256 pos;
        uint256 width;
        bytes32[] proof;
    }

    struct MMRLeafPartial {
        uint8 version;
        uint32 parentNumber;
        bytes32 parentHash;
        uint64 nextAuthoritySetID;
        uint32 nextAuthoritySetLen;
        bytes32 nextAuthoritySetRoot;
    }

    struct Proof {
        bytes headPrefix;
        bytes headSuffix;
        HeadProof headProof;
        MMRLeafPartial leafPartial;
        bytes32[] leafProof;
        uint256 leafProofOrder;
    }

    function verifyCommitment(
        bytes32 commitment,
        Proof calldata proof
    ) external view returns (bool);
}

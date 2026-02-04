// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.33;

interface IBeefyClient {
    /* Types */

    struct PayloadItem {
        bytes2 payloadID;
        bytes data;
    }

    struct Commitment {
        uint32 blockNumber;
        uint64 validatorSetID;
        PayloadItem[] payload;
    }

    struct ValidatorProof {
        uint8 v;
        bytes32 r;
        bytes32 s;
        uint256 index;
        address account;
        bytes32[] proof;
    }

    struct MMRLeaf {
        uint8 version;
        uint32 parentNumber;
        bytes32 parentHash;
        uint64 nextAuthoritySetID;
        uint32 nextAuthoritySetLen;
        bytes32 nextAuthoritySetRoot;
        bytes32 parachainHeadsRoot;
    }

    function latestBeefyBlock() external view returns (uint64);

    function latestMMRRoot() external view returns (bytes32);

    function submitInitial(
        Commitment calldata commitment,
        uint256[] calldata bitfield,
        ValidatorProof calldata proof
    ) external;

    function commitPrevRandao(bytes32 commitmentHash) external;

    function submitFinal(
        Commitment calldata commitment,
        uint256[] calldata bitfield,
        ValidatorProof[] calldata proofs,
        MMRLeaf calldata leaf,
        bytes32[] calldata leafProof,
        uint256 leafProofOrder
    ) external;

    function createFinalBitfield(bytes32 commitmentHash, uint256[] calldata bitfield)
        external
        view
        returns (uint256[] memory);

    function createInitialBitfield(uint256[] calldata bitsToSet, uint256 length)
        external
        pure
        returns (uint256[] memory);

    function randaoCommitDelay() external view returns (uint256);

    function currentValidatorSet()
        external
        view
        returns (uint128 id, uint128 length, bytes32 root);

    function nextValidatorSet()
        external
        view
        returns (uint128 id, uint128 length, bytes32 root);
}

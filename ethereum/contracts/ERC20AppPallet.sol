// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "./ScaleCodec.sol";
import "./SubstrateTypes.sol";

library ERC20AppPallet {
    bytes1 constant PALLET_ID = 0x42;

    bytes1 constant MINT_CALL = 0x01;
    uint64 constant MINT_WEIGHT = 100_000_000;

    bytes1 constant CREATE_CALL = 0x02;
    uint64 constant CREATE_WEIGHT = 100_000_000;

    function mint(
        address token,
        address sender,
        bytes32 recipient,
        uint128 amount
    ) internal pure returns (bytes memory, uint64) {
        return (
            bytes.concat(
                PALLET_ID,
                MINT_CALL,
                SubstrateTypes.H160(token),
                SubstrateTypes.H160(sender),
                SubstrateTypes.MultiAddressWithID(recipient),
                ScaleCodec.encode128(amount),
                SubstrateTypes.None()
            ),
            MINT_WEIGHT
        );
    }

    function mintAndForward(
        address token,
        address sender,
        bytes32 recipient,
        uint128 amount,
        uint32 paraID,
        uint128 fee
    ) internal pure returns (bytes memory, uint64) {
        return (
            bytes.concat(
                PALLET_ID,
                MINT_CALL,
                SubstrateTypes.H160(token),
                SubstrateTypes.H160(sender),
                SubstrateTypes.MultiAddressWithID(recipient),
                ScaleCodec.encode128(amount),
                SubstrateTypes.SomeRemotePara(paraID, fee)
            ),
            MINT_WEIGHT
        );
    }

    function create(address token) internal pure returns (bytes memory, uint64) {
        return (bytes.concat(PALLET_ID, CREATE_CALL, SubstrateTypes.H160(token)), CREATE_WEIGHT);
    }
}

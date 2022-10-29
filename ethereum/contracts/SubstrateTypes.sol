// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "./ScaleCodec.sol";

library SubstrateTypes {
    function MultiAddressWithID(bytes32 account) internal pure returns (bytes memory) {
        return bytes.concat(bytes1(0x00), account);
    }

    function H160(address account) internal pure returns (bytes memory) {
        return abi.encodePacked(account);
    }

    function None() internal pure returns (bytes memory) {
        return hex"11";
    }

    function SomeRemotePara(uint32 paraID, uint128 fee) internal pure returns (bytes memory) {
        return bytes.concat(
            bytes1(0x01),
            ScaleCodec.encode32(paraID),
            ScaleCodec.encode128(fee)
        );
    }
}

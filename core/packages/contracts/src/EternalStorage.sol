// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.20;

abstract contract EternalStorage {
    struct Storage {
        mapping(bytes32 => bool) _bool;
        mapping(bytes32 => int256) _int;
        mapping(bytes32 => uint256) _uint;
        mapping(bytes32 => string) _string;
        mapping(bytes32 => address) _address;
        mapping(bytes32 => bytes) _bytes;
        mapping(bytes32 => bytes32) _bytes32;
    }

    Storage internal s;
}

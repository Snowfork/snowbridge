// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.23;

interface IShell {
    function upgrade(address impl, bytes32 implCodeHash, bytes calldata initializerParams) external;
}

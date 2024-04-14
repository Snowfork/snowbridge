// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.23;

interface IShell {
    // Upgrade gateway shell to a new implementation
    function upgrade(address impl, bytes32 implCodeHash, bytes calldata initializerParams) external;
}

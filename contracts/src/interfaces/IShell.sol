// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.28;

interface IShell {
    error Unauthorized();

    // Upgrade gateway shell to a new implementation
    function upgrade(address impl, bytes32 implCodeHash, bytes calldata initializerParams) external;

    // Retrieve address of trusted operator
    function operator() external returns (address);
}

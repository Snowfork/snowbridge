// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.28;

interface IUpgradable {
    // The new implementation address is a not a contract
    error InvalidContract();
    // The supplied codehash does not match the new implementation codehash
    error InvalidCodeHash();

    // The implementation contract was upgraded
    event Upgraded(address indexed implementation);
}

// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.23;

interface IUpgradable {
    error InvalidContract();
    error InvalidCodeHash();

    event Upgraded(address indexed implementation);
}

// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.20;

// @title Gateway implementations are initializable
interface IInitializable {
    error InitializationFailed();

    function initialize(bytes calldata data) external;
}

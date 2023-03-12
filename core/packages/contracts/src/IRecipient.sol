// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

interface IRecipient {
    function handle(bytes calldata origin, bytes calldata message) external;
}

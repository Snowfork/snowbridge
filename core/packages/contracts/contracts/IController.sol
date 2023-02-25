// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

interface IController {
    function handle(bytes calldata origin, bytes calldata message) external;
}

// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import {AccessControl} from "openzeppelin/access/AccessControl.sol";

import {UpgradeTask} from "../../src/UpgradeTask.sol";
import {Registry} from "../../src/Registry.sol";
import {OutboundQueue} from "../../src/OutboundQueue.sol";
import {ParaID} from "../../src/Types.sol";

contract UpgradeTaskMock is UpgradeTask {
    constructor(Registry registry) UpgradeTask(registry) {}
    function handle(ParaID origin, bytes calldata message) external override onlyRole(SENDER_ROLE) {}

    // In this simple upgrade we just update a fee parameter
    function run() external override {
        OutboundQueue(resolve(keccak256("OutboundQueue"))).updateFee(2 ether);
    }
}

contract FailingUpgradeTaskMock is UpgradeTask {
    constructor(Registry registry) UpgradeTask(registry) {}
    function handle(ParaID origin, bytes calldata message) external override onlyRole(SENDER_ROLE) {}

    function run() external pure override {
        revert("failed");
    }
}

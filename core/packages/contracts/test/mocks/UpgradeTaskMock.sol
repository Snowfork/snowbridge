// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import {AccessControl} from "openzeppelin/access/AccessControl.sol";

import {IUpgradeTask} from "../../src/IUpgradeTask.sol";
import {OutboundQueue} from "../../src/OutboundQueue.sol";

contract UpgradeTaskMock is IUpgradeTask, AccessControl {
    OutboundQueue public immutable outboundQueue;

    constructor(OutboundQueue _outboundQueue) {
        outboundQueue = _outboundQueue;
    }

    // In this simple upgrade we just update a fee parameter
    function run() external {
        outboundQueue.updateFee(2 ether);
    }
}

contract FailingUpgradeTaskMock is IUpgradeTask, AccessControl {
    function run() external pure {
        revert("failed");
    }
}

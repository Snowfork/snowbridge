// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity ^0.8.20;

import {AccessControl} from "openzeppelin/access/AccessControl.sol";

import {UpgradeTask} from "./UpgradeTask.sol";
import {Registry} from "./Registry.sol";
import {OutboundQueue} from "./OutboundQueue.sol";
import {ParaID} from "./Types.sol";

contract UpdateOutboundFee is UpgradeTask {
    uint256 public fee;

    constructor(Registry registry, uint256 _fee) UpgradeTask(registry) {
        fee = _fee;
    }

    function handle(ParaID origin, bytes calldata message) external override onlyRole(SENDER_ROLE) {}

    function run(bytes calldata params) external override {
        (uint256 _fee) = abi.decode(params, (uint256));
        OutboundQueue(resolve(keccak256("OutboundQueue"))).updateFee(_fee);
    }

    function createUpgradeParams() public view override returns (bytes memory) {
        return abi.encode(fee);
    }
}

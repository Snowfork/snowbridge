// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import {AccessControl} from "openzeppelin/access/AccessControl.sol";

import {Test} from "forge-std/Test.sol";

import {UpgradeProxy} from "../src/UpgradeProxy.sol";
import {IUpgradeTask} from "../src/IUpgradeTask.sol";
import {IVault, Vault} from "../src/Vault.sol";
import {OutboundQueue} from "../src/OutboundQueue.sol";
import {ParaID} from "../src/Types.sol";

import {UpgradeTaskMock, FailingUpgradeTaskMock} from "./mocks/UpgradeTaskMock.sol";

contract UpgradeProxyTest is Test {
    UpgradeProxy public upgradeProxy;
    IUpgradeTask public upgradeTask;
    IUpgradeTask public failedUpgradeTask;

    OutboundQueue public outboundQueue;

    ParaID origin = ParaID.wrap(1001);

    function setUp() public {
        upgradeProxy = new UpgradeProxy(origin);
        outboundQueue = new OutboundQueue(new Vault(), 1 ether);

        outboundQueue.grantRole(outboundQueue.ADMIN_ROLE(), address(upgradeProxy));
        outboundQueue.revokeRole(outboundQueue.ADMIN_ROLE(), address(this));

        upgradeProxy.grantRole(upgradeProxy.SENDER_ROLE(), address(this));

        // create upgrade tasks
        upgradeTask = new UpgradeTaskMock(outboundQueue);
        failedUpgradeTask = new FailingUpgradeTaskMock();
    }

    function createUpgradeMessage(IUpgradeTask task) internal pure returns (bytes memory) {
        return abi.encode(
            UpgradeProxy.Message(UpgradeProxy.Action.Upgrade, abi.encode(UpgradeProxy.UpgradePayload(address(task))))
        );
    }

    function testUpgrade() public {
        bytes memory message = createUpgradeMessage(upgradeTask);
        upgradeProxy.handle(origin, message);
        assertEq(outboundQueue.fee(), 2 ether);
    }

    function testUpgradeFailBadOrigin() public {
        vm.expectRevert(UpgradeProxy.Unauthorized.selector);
        upgradeProxy.handle(ParaID.wrap(3), hex"deadbeef");
    }

    function testUpgradeFail() public {
        bytes memory message = createUpgradeMessage(failedUpgradeTask);
        vm.expectRevert(UpgradeProxy.UpgradeFailed.selector);
        upgradeProxy.handle(origin, message);
    }
}

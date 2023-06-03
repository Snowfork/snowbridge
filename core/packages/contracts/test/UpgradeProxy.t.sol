// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import {AccessControl} from "openzeppelin/access/AccessControl.sol";

import {Test} from "forge-std/Test.sol";

import {UpgradeProxy} from "../src/UpgradeProxy.sol";
import {UpgradeTask} from "../src/UpgradeTask.sol";
import {Vault} from "../src/Vault.sol";
import {OutboundQueue} from "../src/OutboundQueue.sol";
import {ParaID} from "../src/Types.sol";
import {Registry} from "../src/Registry.sol";
import {Gateway} from "../src/Gateway.sol";

import {UpgradeTaskMock, FailingUpgradeTaskMock} from "./mocks/UpgradeTaskMock.sol";

contract UpgradeProxyTest is Test {
    UpgradeProxy public upgradeProxy;
    UpgradeTask public upgradeTask;
    UpgradeTask public failedUpgradeTask;

    OutboundQueue public outboundQueue;

    ParaID origin = ParaID.wrap(1001);

    function setUp() public {
        Registry registry = new Registry();
        registry.grantRole(registry.REGISTER_ROLE(), address(this));

        upgradeProxy = new UpgradeProxy(registry, origin);
        registry.registerContract(keccak256("UpgradeProxy"), address(upgradeProxy));

        outboundQueue = new OutboundQueue(registry, new Vault(), 1 ether);
        registry.registerContract(keccak256("OutboundQueue"), address(outboundQueue));

        outboundQueue.grantRole(outboundQueue.ADMIN_ROLE(), address(upgradeProxy));
        outboundQueue.revokeRole(outboundQueue.ADMIN_ROLE(), address(this));

        upgradeProxy.grantRole(upgradeProxy.SENDER_ROLE(), address(this));

        registry.grantRole(outboundQueue.ADMIN_ROLE(), address(upgradeProxy));

        // create upgrade tasks
        upgradeTask = new UpgradeTaskMock(registry);
        failedUpgradeTask = new FailingUpgradeTaskMock(registry);
    }

    function createUpgradeMessage(UpgradeTask task) internal pure returns (bytes memory) {
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
        vm.expectRevert(Gateway.Unauthorized.selector);
        upgradeProxy.handle(ParaID.wrap(3), hex"deadbeef");
    }

    function testUpgradeFail() public {
        bytes memory message = createUpgradeMessage(failedUpgradeTask);
        vm.expectRevert(UpgradeProxy.UpgradeFailed.selector);
        upgradeProxy.handle(origin, message);
    }
}

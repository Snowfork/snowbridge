// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.23;

import {Test} from "forge-std/Test.sol";
import {Strings} from "openzeppelin/utils/Strings.sol";
import {console} from "forge-std/console.sol";

import {IGateway} from "../src/interfaces/IGateway.sol";
import {IInitializable} from "../src/interfaces/IInitializable.sol";
import {IUpgradable} from "../src/interfaces/IUpgradable.sol";
import {IShell} from "../src/interfaces/IShell.sol";
import {Gateway} from "../src/Gateway.sol";
import {GatewayProxy} from "../src/GatewayProxy.sol";
import {Shell} from "../src/Shell.sol";
import {Upgrade} from "../src/Upgrade.sol";
import {GatewayMock, GatewayV2} from "./mocks/GatewayMock.sol";

contract ShellTest is Test {
    GatewayProxy public gateway;
    Shell public shell;

    function setUp() public {
        shell = new Shell(address(this));
        gateway = new GatewayProxy(address(shell), bytes(""));
    }

    function testUpgradeShell() public {
        // Upgrade to this new logic contract
        address newLogic = address(new GatewayV2());
        bytes memory initParams = abi.encode(42);

        // Expect the gateway to emit `Upgrade.Upgraded`
        vm.expectEmit(true, false, false, true);
        emit IUpgradable.Upgraded(newLogic);

        // Perform the upgrade
        IShell(address(gateway)).upgrade(newLogic, newLogic.codehash, initParams);

        // Verify that the upgrade occured

        // Execute code only available in the new impl
        assertEq(GatewayV2(address(gateway)).getValue(), 42);

        // Should no longer be able to upgrade via trusted operator
        vm.expectRevert();
        IShell(address(gateway)).upgrade(newLogic, newLogic.codehash, initParams);
    }
}

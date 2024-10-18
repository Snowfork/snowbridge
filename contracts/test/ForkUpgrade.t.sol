// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.25;

import {Test} from "forge-std/Test.sol";
import {console} from "forge-std/console.sol";

import {IUpgradable} from "../src/interfaces/IUpgradable.sol";
import {IGateway} from "../src/interfaces/IGateway.sol";
import {Gateway} from "../src/Gateway.sol";
import {Gateway202411} from "../src/upgrades/polkadot/Gateway202411.sol";
import {AgentExecutor} from "../src/AgentExecutor.sol";
import {
    UpgradeParams,
    SetOperatingModeParams,
    OperatingMode,
    RegisterForeignTokenParams
} from "../src/v1/Types.sol";
import {ChannelID, ParaID, OperatingMode, TokenInfo} from "../src/Types.sol";

// Old API
interface Gateway202404 {
    function upgrade(bytes memory data) external;
}

contract ForkUpgradeTest is Test {
    address private constant GATEWAY_PROXY = 0x27ca963C279c93801941e1eB8799c23f407d68e7;
    address private constant BEEFY_CLIENT = 0x6eD05bAa904df3DE117EcFa638d4CB84e1B8A00C;
    bytes32 private constant BRIDGE_HUB_AGENT =
        0x03170a2e7597b7b7e3d84c05391d139a62b157e78786d8c082f29dcf4c111314;

    function setUp() public {
        vm.createSelectFork(
            "https://rpc.tenderly.co/fork/b77e07b8-ad6d-4e83-b5be-30a2001964aa",
            20_645_700
        );
        vm.allowCheatcodes(GATEWAY_PROXY);
        vm.startPrank(GATEWAY_PROXY);
        forkUpgrade();
    }

    function forkUpgrade() public {
        AgentExecutor executor = new AgentExecutor();

        Gateway202411 newLogic = new Gateway202411(BEEFY_CLIENT, address(executor));

        UpgradeParams memory params = UpgradeParams({
            impl: address(newLogic),
            implCodeHash: address(newLogic).codehash,
            initParams: bytes("")
        });

        vm.expectEmit(true, false, false, false);
        emit IUpgradable.Upgraded(address(newLogic));

        Gateway202404(GATEWAY_PROXY).upgrade(abi.encode(params));
    }

    function checkLegacyToken() public {
        TokenInfo memory weth = Gateway202411(GATEWAY_PROXY).tokenInfo(
            0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2
        );
        assertEq(weth.isRegistered, true);
        assertEq(weth.foreignID, bytes32(""));
        TokenInfo memory myth = Gateway202411(GATEWAY_PROXY).tokenInfo(
            0xBA41Ddf06B7fFD89D1267b5A93BFeF2424eb2003
        );
        assertEq(myth.isRegistered, true);
        assertEq(myth.foreignID, bytes32(""));
    }

    function registerForeignToken() public {
        bytes32 dotId =
            0xa8f2ec5bdd7a07d844ee3bce83f9ba3881f495d96f07cacbeeb77d9e031db4f0;
        RegisterForeignTokenParams memory params = RegisterForeignTokenParams({
            foreignTokenID: dotId,
            name: "DOT",
            symbol: "DOT",
            decimals: 10
        });

        vm.expectEmit(true, true, false, false);
        emit IGateway.ForeignTokenRegistered(dotId, address(0x0));

        Gateway202411(GATEWAY_PROXY).v1_handleRegisterForeignToken(abi.encode(params));
        TokenInfo memory dot = Gateway202411(GATEWAY_PROXY).tokenInfo(
            0x70D9d338A6b17957B16836a90192BD8CDAe0b53d
        );
        assertEq(dot.isRegistered, true);
        assertEq(dot.foreignID, dotId);
    }

    function testSanityCheck() public {
        // Check AH channel nonces as expected
        (uint64 inbound, uint64 outbound) = IGateway(GATEWAY_PROXY).channelNoncesOf(
            ChannelID.wrap(
                0xc173fac324158e77fb5840738a1a541f633cbec8884c6a601c567d2b376a0539
            )
        );
        assertEq(inbound, 13);
        assertEq(outbound, 172);
        // Register PNA
        registerForeignToken();
        // Check legacy ethereum token not affected
        checkLegacyToken();
    }
}

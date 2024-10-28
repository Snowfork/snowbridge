// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.25;

import {WETH9} from "canonical-weth/WETH9.sol";
import {Script} from "forge-std/Script.sol";
import {stdJson} from "forge-std/StdJson.sol";
import {UD60x18, ud60x18} from "prb/math/src/UD60x18.sol";

import {BeefyClient} from "../../src/BeefyClient.sol";
import {IGateway} from "../../src/interfaces/IGateway.sol";
import {IShell} from "../../src/interfaces/IShell.sol";
import {GatewayProxy} from "../../src/GatewayProxy.sol";
import {Gateway} from "../../src/Gateway.sol";
import {MockGatewayV2} from "../../test/mocks/MockGatewayV2.sol";
import {Initializer} from "../../src/Initializer.sol";
import {Agent} from "../../src/Agent.sol";
import {AgentExecutor} from "../../src/AgentExecutor.sol";
import {ChannelID, ParaID, OperatingMode} from "../../src/Types.sol";
import {SafeNativeTransfer} from "../../src/utils/SafeTransfer.sol";

contract UpgradeShell is Script {
    using SafeNativeTransfer for address payable;
    using stdJson for string;

    struct Config {
        address gatewayProxy;
        address beefyClient;
        Initializer.Config initializerParams;
    }

    function readConfig() internal pure returns (Config memory config) {
        config = Config({
            gatewayProxy: 0x9Ed8b47Bc3417e3BD0507ADC06E56e2Fa360A4E9,
            beefyClient: 0x6DFaD3D73A28c48E4F4c616ECda80885b415283a,
            initializerParams: Initializer.Config({
                mode: OperatingMode.Normal,
                deliveryCost: 200_000_000_000, // 0.2 Wnd
                registerTokenFee: 0.002 ether,
                assetHubCreateAssetFee: 200_000_000_000, // 0.2 Wnd
                assetHubReserveTransferFee: 200_000_000_000, // 0.2 Wnd
                exchangeRate: ud60x18(2_400_000_000_000_000),
                multiplier: ud60x18(1_330_000_000_000_000_000),
                rescueOperator: 0x302F0B71B8aD3CF6dD90aDb668E49b2168d652fd,
                foreignTokenDecimals: 12,
                maxDestinationFee: 2_000_000_000_000
            })
        });
    }

    function run() public {
        vm.startBroadcast();

        Config memory config = readConfig();

        // AgentExecutor
        AgentExecutor executor = new AgentExecutor();

        // Gateway implementation
        Gateway gatewayLogic = new Gateway(config.beefyClient, address(executor));

        IShell shell = IShell(config.gatewayProxy);

        shell.upgrade(
            address(gatewayLogic),
            address(gatewayLogic).codehash,
            abi.encode(config.initializerParams)
        );

        vm.stopBroadcast();
    }
}

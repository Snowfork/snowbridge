// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.23;

import {WETH9} from "canonical-weth/WETH9.sol";
import {Script} from "forge-std/Script.sol";
import {BeefyClient} from "../src/BeefyClient.sol";

import {IGateway} from "../src/interfaces/IGateway.sol";
import {IShell} from "../src/interfaces/IShell.sol";
import {GatewayProxy} from "../src/GatewayProxy.sol";
import {Gateway} from "../src/Gateway.sol";
import {MockGatewayV2} from "../test/mocks/MockGatewayV2.sol";
import {Agent} from "../src/Agent.sol";
import {AgentExecutor} from "../src/AgentExecutor.sol";
import {ChannelID, ParaID, OperatingMode} from "../src/Types.sol";
import {SafeNativeTransfer} from "../src/utils/SafeTransfer.sol";
import {stdJson} from "forge-std/StdJson.sol";
import {UD60x18, ud60x18} from "prb/math/src/UD60x18.sol";


// 1 mDOT = 0.001 DOT
function mdot(uint32 value) pure returns (uint128) {
    return value * (10 ** 7);
}

contract Stage3 is Script {
    using SafeNativeTransfer for address payable;
    using stdJson for string;

    struct Config {
        address gatewayProxy;
        address beefyClient;
        ParaID bridgeHubParaID;
        bytes32 bridgeHubAgentID;
        uint8 foreignTokenDecimals;
        uint128 maxDestinationFee;
        Gateway.Config initializerParams;
    }

    function readConfig() internal pure returns (Config memory config) { 
        config = Config({
            gatewayProxy: address(0),
            beefyClient: address(0),
            bridgeHubParaID: ParaID.wrap(1002),
            bridgeHubAgentID: 0x03170a2e7597b7b7e3d84c05391d139a62b157e78786d8c082f29dcf4c111314,
            foreignTokenDecimals: 10,
            maxDestinationFee: 0,
            initializerParams: Gateway.Config({
                mode: OperatingMode.Normal,
                deliveryCost: mdot(100), // 0.1 DOT
                registerTokenFee: 0.002 ether,
                assetHubParaID: ParaID.wrap(1000),
                assetHubAgentID: 0x81c5ab2571199e3188135178f3c2c8e2d268be1313d029b30f534fa579b69b79,
                assetHubCreateAssetFee: mdot(100), // 0.1 DOT
                assetHubReserveTransferFee: mdot(100), // 0.1 DOT
                exchangeRate: ud60x18(0.0024e18),
                multiplier: ud60x18(1.33e18)
            })
        });
    }

    function run() public {
        vm.startBroadcast();

        Config memory config = readConfig();

        // AgentExecutor
        AgentExecutor executor = new AgentExecutor();

        // Gateway implementation
        Gateway gatewayLogic = new Gateway(
            config.beefyClient,
            address(executor),
            config.bridgeHubParaID,
            config.bridgeHubAgentID,
            config.foreignTokenDecimals,
            config.maxDestinationFee
        );

        IShell shell = IShell(config.gatewayProxy);

        shell.upgrade(address(gatewayLogic), address(gatewayLogic).codehash, abi.encode(config.initializerParams));

        vm.stopBroadcast();
    }
}

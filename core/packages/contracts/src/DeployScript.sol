// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.20;

import {WETH9} from "canonical-weth/WETH9.sol";
import {Script} from "forge-std/Script.sol";
import {BeefyClient} from "./BeefyClient.sol";

import {IGateway} from "./interfaces/IGateway.sol";
import {GatewayProxy} from "./GatewayProxy.sol";
import {Gateway} from "./Gateway.sol";
import {Agent} from "./Agent.sol";
import {AgentExecutor} from "./AgentExecutor.sol";
import {ParaID, Config} from "./Types.sol";
import {SafeNativeTransfer} from "./utils/SafeTransfer.sol";

contract DeployScript is Script {
    using SafeNativeTransfer for address payable;

    function setUp() public {}

    function run() public {
        uint256 privateKey = vm.envUint("PRIVATE_KEY");
        address deployer = vm.rememberKey(privateKey);
        vm.startBroadcast(deployer);

        // BeefyClient
        uint256 randaoCommitDelay = vm.envUint("RANDAO_COMMIT_DELAY");
        uint256 randaoCommitExpiration = vm.envUint("RANDAO_COMMIT_EXP");
        BeefyClient beefyClient = new BeefyClient(randaoCommitDelay, randaoCommitExpiration);

        ParaID bridgeHubParaID = ParaID.wrap(vm.envUint("BRIDGE_HUB_PARAID"));
        bytes32 bridgeHubAgentID = vm.envBytes32("BRIDGE_HUB_AGENT_ID");
        ParaID assetHubParaID = ParaID.wrap(vm.envUint("ASSET_HUB_PARAID"));
        bytes32 assetHubAgentID = vm.envBytes32("ASSET_HUB_AGENT_ID");

        AgentExecutor executor = new AgentExecutor();
        Gateway gatewayLogic = new Gateway(
            address(beefyClient),
            address(executor),
            vm.envUint("DISPATCH_GAS"),
            bridgeHubParaID,
            bridgeHubAgentID,
            assetHubParaID,
            assetHubAgentID,
            bytes2(vm.envBytes("CREATE_CALL_INDEX"))
        );

        bytes memory initParams = abi.encode(
            vm.envUint("DEFAULT_FEE"),
            vm.envUint("DEFAULT_REWARD"),
            vm.envUint("REGISTER_NATIVE_TOKEN_FEE"),
            vm.envUint("SEND_NATIVE_TOKEN_FEE")
        );

        GatewayProxy gateway = new GatewayProxy(address(gatewayLogic), initParams);

        // Deploy WETH for testing
        new WETH9();

        // Fund the sovereign account for the BridgeHub parachain. Used to reward relayers
        // of messages originating from BridgeHub
        uint256 initialDeposit = vm.envUint("BRIDGE_HUB_INITIAL_DEPOSIT");

        address bridgeHubAgent = IGateway(address(gateway)).agentOf(bridgeHubAgentID);
        address assetHubAgent = IGateway(address(gateway)).agentOf(assetHubAgentID);

        payable(bridgeHubAgent).safeNativeTransfer(initialDeposit);
        payable(assetHubAgent).safeNativeTransfer(initialDeposit);

        vm.stopBroadcast();
    }
}

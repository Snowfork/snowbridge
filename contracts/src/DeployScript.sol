// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.22;

import {WETH9} from "canonical-weth/WETH9.sol";
import {Script} from "forge-std/Script.sol";
import {BeefyClient} from "./BeefyClient.sol";

import {IGateway} from "./interfaces/IGateway.sol";
import {GatewayProxy} from "./GatewayProxy.sol";
import {Gateway} from "./Gateway.sol";
import {GatewayUpgradeMock} from "../test/mocks/GatewayUpgradeMock.sol";
import {Agent} from "./Agent.sol";
import {AgentExecutor} from "./AgentExecutor.sol";
import {ParaID, Config} from "./Types.sol";
import {SafeNativeTransfer} from "./utils/SafeTransfer.sol";
import {stdJson} from "forge-std/StdJson.sol";

contract DeployScript is Script {
    using SafeNativeTransfer for address payable;
    using stdJson for string;

    function setUp() public {}

    function run() public {
        uint256 privateKey = vm.envUint("PRIVATE_KEY");
        address deployer = vm.rememberKey(privateKey);
        vm.startBroadcast(deployer);

        // BeefyClient
        // Seems `fs_permissions` explicitly configured as absolute path does not work and only allowed from project root
        string memory root = vm.projectRoot();
        string memory beefyCheckpointFile = string.concat(root, "/beefy-state.json");
        string memory beefyCheckpointRaw = vm.readFile(beefyCheckpointFile);
        uint64 startBlock = uint64(beefyCheckpointRaw.readUint(".startBlock"));

        BeefyClient.ValidatorSet memory current = BeefyClient.ValidatorSet(
            uint128(beefyCheckpointRaw.readUint(".current.id")),
            uint128(beefyCheckpointRaw.readUint(".current.length")),
            beefyCheckpointRaw.readBytes32(".current.root")
        );
        BeefyClient.ValidatorSet memory next = BeefyClient.ValidatorSet(
            uint128(beefyCheckpointRaw.readUint(".next.id")),
            uint128(beefyCheckpointRaw.readUint(".next.length")),
            beefyCheckpointRaw.readBytes32(".next.root")
        );

        uint256 randaoCommitDelay = vm.envUint("RANDAO_COMMIT_DELAY");
        uint256 randaoCommitExpiration = vm.envUint("RANDAO_COMMIT_EXP");
        uint256 minimumSignatures = vm.envUint("MINIMUM_REQUIRED_SIGNATURES");
        BeefyClient beefyClient =
            new BeefyClient(randaoCommitDelay, randaoCommitExpiration, minimumSignatures, startBlock, current, next);

        ParaID bridgeHubParaID = ParaID.wrap(vm.envUint("BRIDGE_HUB_PARAID"));
        bytes32 bridgeHubAgentID = vm.envBytes32("BRIDGE_HUB_AGENT_ID");
        ParaID assetHubParaID = ParaID.wrap(vm.envUint("ASSET_HUB_PARAID"));
        bytes32 assetHubAgentID = vm.envBytes32("ASSET_HUB_AGENT_ID");

        AgentExecutor executor = new AgentExecutor();
        Gateway gatewayLogic = new Gateway(
            address(beefyClient),
            address(executor),
            bridgeHubParaID,
            bridgeHubAgentID,
            assetHubParaID,
            assetHubAgentID
        );

        bytes memory initParams = abi.encode(
            vm.envUint("DEFAULT_FEE"), vm.envUint("REGISTER_NATIVE_TOKEN_FEE"), vm.envUint("SEND_NATIVE_TOKEN_FEE")
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

        new GatewayUpgradeMock();

        vm.stopBroadcast();
    }
}

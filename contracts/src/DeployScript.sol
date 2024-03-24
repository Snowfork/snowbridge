// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.23;

import {WETH9} from "canonical-weth/WETH9.sol";
import {Script} from "forge-std/Script.sol";
import {BeefyClient} from "./BeefyClient.sol";

import {IGateway} from "./interfaces/IGateway.sol";
import {GatewayProxy} from "./GatewayProxy.sol";
import {Gateway} from "./Gateway.sol";
import {GatewayOutbound} from "./GatewayOutbound.sol";
import {GatewayUpgradeMock} from "../test/mocks/GatewayUpgradeMock.sol";
import {Agent} from "./Agent.sol";
import {AgentExecutor} from "./AgentExecutor.sol";
import {ChannelID, ParaID, OperatingMode} from "./Types.sol";
import {SafeNativeTransfer} from "./utils/SafeTransfer.sol";
import {stdJson} from "forge-std/StdJson.sol";
import {UD60x18, ud60x18} from "prb/math/src/UD60x18.sol";
import {DiamondStorage} from "./storage/DiamondStorage.sol";

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

        ParaID bridgeHubParaID = ParaID.wrap(uint32(vm.envUint("BRIDGE_HUB_PARAID")));
        bytes32 bridgeHubAgentID = vm.envBytes32("BRIDGE_HUB_AGENT_ID");
        ParaID assetHubParaID = ParaID.wrap(uint32(vm.envUint("ASSET_HUB_PARAID")));
        bytes32 assetHubAgentID = vm.envBytes32("ASSET_HUB_AGENT_ID");

        uint8 foreignTokenDecimals = uint8(vm.envUint("FOREIGN_TOKEN_DECIMALS"));

        AgentExecutor executor = new AgentExecutor();
        Gateway gatewayLogic = new Gateway();

        bool rejectOutboundMessages = vm.envBool("REJECT_OUTBOUND_MESSAGES");
        OperatingMode defaultOperatingMode;
        if (rejectOutboundMessages) {
            defaultOperatingMode = OperatingMode.RejectingOutboundMessages;
        } else {
            defaultOperatingMode = OperatingMode.Normal;
        }

        Gateway.Config memory config = Gateway.Config({
            beefyClient: address(beefyClient),
            agentExecutor: address(executor),
            bridgeHubParaID: bridgeHubParaID,
            bridgeHubAgentID: bridgeHubAgentID,
            foreignTokenDecimals: foreignTokenDecimals,
            mode: defaultOperatingMode,
            deliveryCost: uint128(vm.envUint("DELIVERY_COST")),
            registerTokenFee: uint128(vm.envUint("REGISTER_TOKEN_FEE")),
            assetHubParaID: assetHubParaID,
            assetHubAgentID: assetHubAgentID,
            assetHubCreateAssetFee: uint128(vm.envUint("CREATE_ASSET_FEE")),
            assetHubReserveTransferFee: uint128(vm.envUint("RESERVE_TRANSFER_FEE")),
            exchangeRate: ud60x18(vm.envUint("EXCHANGE_RATE")),
            multiplier: ud60x18(vm.envUint("FEE_MULTIPLIER"))
        });

        // Initialize facet of gatewayLogic
        bytes4[] memory gatewayLogicSelectors = new bytes4[](15);
        /// Functions from Gateway
        //submitV1
        gatewayLogicSelectors[0] = bytes4(0xdf4ed829);
        //operatingMode
        gatewayLogicSelectors[1] = bytes4(0x38004f69);
        //channelOperatingModeOf
        gatewayLogicSelectors[2] = bytes4(0x0705f465);
        //channelNoncesOf
        gatewayLogicSelectors[3] = bytes4(0x2a6c3229);
        //agentOf
        gatewayLogicSelectors[4] = bytes4(0x5e6dae26);
        //pricingParameters
        gatewayLogicSelectors[5] = bytes4(0x0b617646);
        //agentExecute
        gatewayLogicSelectors[6] = bytes4(0x35ede969);
        //createAgent
        gatewayLogicSelectors[7] = bytes4(0xc3b8ec8e);
        //createChannel
        gatewayLogicSelectors[8] = bytes4(0x17abcf60);
        //updateChannel
        gatewayLogicSelectors[9] = bytes4(0xafce33c4);
        //upgrade
        gatewayLogicSelectors[10] = bytes4(0x25394645);
        //setOperatingMode
        gatewayLogicSelectors[11] = bytes4(0x8257f3d5);
        //transferNativeFromAgent
        gatewayLogicSelectors[12] = bytes4(0x9a870c8b);
        //setTokenTransferFees
        gatewayLogicSelectors[13] = bytes4(0x5b2e9c4c);
        //setPricingParameters
        gatewayLogicSelectors[14] = bytes4(0x0c86ea46);

        // Initialize facet of gatewayOutboundLogic
        GatewayOutbound gatewayOutboundLogic = new GatewayOutbound();
        bytes4[] memory gatewayOutboundLogicSelectors = new bytes4[](7);
        //isTokenRegistered
        gatewayOutboundLogicSelectors[0] = bytes4(0x26aa101f);
        //quoteRegisterTokenFee
        gatewayOutboundLogicSelectors[1] = bytes4(0x805ce31d);
        //registerToken
        gatewayOutboundLogicSelectors[2] = bytes4(0x09824a80);
        //quoteSendTokenFee
        gatewayOutboundLogicSelectors[3] = bytes4(0x928bc49d);
        //sendToken
        gatewayOutboundLogicSelectors[4] = bytes4(0x52054834);
        //transferToken
        gatewayOutboundLogicSelectors[5] = bytes4(0x1382f5eb);
        //getTokenInfo
        gatewayOutboundLogicSelectors[6] = bytes4(0x2d8b70a1);

        // Initialize facetCut
        DiamondStorage.FacetCut memory gatewayLogicFacetCut = DiamondStorage.FacetCut({
            facetAddress: address(gatewayLogic),
            action: DiamondStorage.FacetCutAction.Add,
            functionSelectors: gatewayLogicSelectors
        });
        DiamondStorage.FacetCut memory gatewayOutboundLogicFacetCut = DiamondStorage.FacetCut({
            facetAddress: address(gatewayOutboundLogic),
            action: DiamondStorage.FacetCutAction.Add,
            functionSelectors: gatewayOutboundLogicSelectors
        });
        DiamondStorage.FacetCut[] memory facetCuts = new DiamondStorage.FacetCut[](2);
        facetCuts[0] = gatewayLogicFacetCut;
        facetCuts[1] = gatewayOutboundLogicFacetCut;

        GatewayProxy gateway = new GatewayProxy(facetCuts, address(gatewayLogic), abi.encode(config));

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

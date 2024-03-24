// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.23;

import {Test} from "forge-std/Test.sol";
import {Strings} from "openzeppelin/utils/Strings.sol";
import {console} from "forge-std/console.sol";

import {BeefyClient} from "../src/BeefyClient.sol";

import {IGateway} from "../src/interfaces/IGateway.sol";
import {IGatewayOutbound} from "../src/interfaces/IGatewayOutbound.sol";
import {IInitializable} from "../src/interfaces/IInitializable.sol";
import {Gateway} from "../src/Gateway.sol";
import {GatewayOutbound} from "../src/GatewayOutbound.sol";
import {GatewayMock, GatewayV2} from "./mocks/GatewayMock.sol";

import {GatewayProxy} from "../src/GatewayProxy.sol";

import {AgentExecutor} from "../src/AgentExecutor.sol";
import {Agent} from "../src/Agent.sol";
import {Verification} from "../src/Verification.sol";
import {Assets} from "../src/Assets.sol";
import {SubstrateTypes} from "./../src/SubstrateTypes.sol";

import {NativeTransferFailed} from "../src/utils/SafeTransfer.sol";
import {PricingStorage} from "../src/storage/PricingStorage.sol";
import {TokenInfo} from "../src/storage/AssetsStorage.sol";

import {
    UpgradeParams,
    CreateAgentParams,
    AgentExecuteParams,
    CreateChannelParams,
    UpdateChannelParams,
    SetOperatingModeParams,
    TransferNativeFromAgentParams,
    SetTokenTransferFeesParams,
    SetPricingParametersParams
} from "../src/Params.sol";

import {
    AgentExecuteCommand,
    InboundMessage,
    OperatingMode,
    ParaID,
    Command,
    multiAddressFromBytes32,
    multiAddressFromBytes20
} from "../src/Types.sol";

import {WETH9} from "canonical-weth/WETH9.sol";
import "./mocks/GatewayUpgradeMock.sol";
import {UD60x18, ud60x18, convert} from "prb/math/src/UD60x18.sol";
import {DiamondStorage} from "../src/storage/DiamondStorage.sol";

contract GatewayTest is Test {
    // Emitted when token minted
    event TokenMinted(bytes32 indexed tokenID, address token, address recipient, uint256 amount);

    ParaID public bridgeHubParaID = ParaID.wrap(1001);
    bytes32 public bridgeHubAgentID = keccak256("1001");
    address public bridgeHubAgent;

    ParaID public assetHubParaID = ParaID.wrap(1002);
    bytes32 public assetHubAgentID = keccak256("1002");
    address public assetHubAgent;

    address public relayer;

    bytes32[] public proof = [bytes32(0x2f9ee6cfdf244060dc28aa46347c5219e303fc95062dd672b4e406ca5c29764b)];
    bytes public parachainHeaderProof = bytes("validProof");

    GatewayMock public gatewayLogic;
    GatewayProxy public gateway;

    WETH9 public token;

    address public account1;
    address public account2;

    uint64 public maxDispatchGas = 500_000;
    uint256 public maxRefund = 1 ether;
    uint256 public reward = 1 ether;
    bytes32 public messageID = keccak256("cabbage");

    // remote fees in DOT
    uint128 public outboundFee = 1e10;
    uint128 public registerTokenFee = 0;
    uint128 public sendTokenFee = 1e10;
    uint128 public createTokenFee = 1e10;

    MultiAddress public recipientAddress32;
    MultiAddress public recipientAddress20;

    // For DOT
    uint8 public foreignTokenDecimals = 10;

    // ETH/DOT exchange rate
    UD60x18 public exchangeRate = ud60x18(0.0025e18);
    UD60x18 public multiplier = ud60x18(1e18);

    // tokenID for DOT
    bytes32 public dotTokenID;

    function setUp() public {
        AgentExecutor executor = new AgentExecutor();
        gatewayLogic = new GatewayMock();
        Gateway.Config memory config = Gateway.Config({
            beefyClient: address(0),
            agentExecutor: address(executor),
            bridgeHubParaID: bridgeHubParaID,
            bridgeHubAgentID: bridgeHubAgentID,
            foreignTokenDecimals: foreignTokenDecimals,
            mode: OperatingMode.Normal,
            deliveryCost: outboundFee,
            registerTokenFee: registerTokenFee,
            assetHubParaID: assetHubParaID,
            assetHubAgentID: assetHubAgentID,
            assetHubCreateAssetFee: createTokenFee,
            assetHubReserveTransferFee: sendTokenFee,
            exchangeRate: exchangeRate,
            multiplier: multiplier
        });

        // Initialize facet of gatewayLogic
        bytes4[] memory gatewayLogicSelectors = new bytes4[](32);

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

        /// Functions from GatewayOutbound
        //isTokenRegistered
        gatewayLogicSelectors[15] = bytes4(0x26aa101f);
        //quoteRegisterTokenFee
        gatewayLogicSelectors[16] = bytes4(0x805ce31d);
        //registerToken
        gatewayLogicSelectors[17] = bytes4(0x09824a80);
        //quoteSendTokenFee
        gatewayLogicSelectors[18] = bytes4(0x928bc49d);
        //sendToken
        gatewayLogicSelectors[19] = bytes4(0x52054834);
        //transferToken
        gatewayLogicSelectors[20] = bytes4(0x1382f5eb);
        //getTokenInfo
        gatewayLogicSelectors[21] = bytes4(0x2d8b70a1);

        /// Functions from GatewayMock
        //agentExecutePublic
        gatewayLogicSelectors[22] = bytes4(0x0b998355);
        //createAgentPublic
        gatewayLogicSelectors[23] = bytes4(0x98807a62);
        //upgradePublic
        gatewayLogicSelectors[24] = bytes4(0x50a7fb1f);
        //createChannelPublic
        gatewayLogicSelectors[25] = bytes4(0x6c948958);
        //updateChannelPublic
        gatewayLogicSelectors[26] = bytes4(0xefa4191c);
        //setOperatingModePublic
        gatewayLogicSelectors[27] = bytes4(0x9930e8ab);
        //transferNativeFromAgentPublic
        gatewayLogicSelectors[28] = bytes4(0x34d64cdd);
        //setCommitmentsAreVerified
        gatewayLogicSelectors[29] = bytes4(0x1500ab89);
        //setTokenTransferFeesPublic
        gatewayLogicSelectors[30] = bytes4(0x493cc51a);
        //setPricingParametersPublic
        gatewayLogicSelectors[31] = bytes4(0xddd419e0);

        // Initialize facetCut
        DiamondStorage.FacetCut memory gatewayLogicFacetCut = DiamondStorage.FacetCut({
            facetAddress: address(gatewayLogic),
            action: DiamondStorage.FacetCutAction.Add,
            functionSelectors: gatewayLogicSelectors
        });
        DiamondStorage.FacetCut[] memory facetCuts = new DiamondStorage.FacetCut[](1);
        facetCuts[0] = gatewayLogicFacetCut;

        gateway = new GatewayProxy(facetCuts, address(gatewayLogic), abi.encode(config));
        GatewayMock(address(gateway)).setCommitmentsAreVerified(true);

        SetOperatingModeParams memory params = SetOperatingModeParams({mode: OperatingMode.Normal});
        GatewayMock(address(gateway)).setOperatingModePublic(abi.encode(params));

        bridgeHubAgent = IGateway(address(gateway)).agentOf(bridgeHubAgentID);
        assetHubAgent = IGateway(address(gateway)).agentOf(assetHubAgentID);

        // fund the message relayer account
        relayer = makeAddr("relayer");

        // Features

        token = new WETH9();

        account1 = makeAddr("account1");
        account2 = makeAddr("account2");

        // create tokens for account 1
        hoax(account1);
        token.deposit{value: 500}();

        // create tokens for account 2
        token.deposit{value: 500}();

        recipientAddress32 = multiAddressFromBytes32(keccak256("recipient"));
        recipientAddress20 = multiAddressFromBytes20(bytes20(keccak256("recipient")));

        dotTokenID = bytes32(uint256(1));
    }

    function makeCreateAgentCommand() public pure returns (Command, bytes memory) {
        return (Command.CreateAgent, abi.encode((keccak256("6666"))));
    }

    function makeMockProof() public pure returns (Verification.Proof memory) {
        return Verification.Proof({
            header: Verification.ParachainHeader({
                parentHash: bytes32(0),
                number: 0,
                stateRoot: bytes32(0),
                extrinsicsRoot: bytes32(0),
                digestItems: new Verification.DigestItem[](0)
            }),
            headProof: Verification.HeadProof({pos: 0, width: 0, proof: new bytes32[](0)}),
            leafPartial: Verification.MMRLeafPartial({
                version: 0,
                parentNumber: 0,
                parentHash: bytes32(0),
                nextAuthoritySetID: 0,
                nextAuthoritySetLen: 0,
                nextAuthoritySetRoot: 0
            }),
            leafProof: new bytes32[](0),
            leafProofOrder: 0
        });
    }

    fallback() external payable {}
    receive() external payable {}

    /**
     * Message Verification
     */
    function testSubmitHappyPath() public {
        deal(assetHubAgent, 50 ether);

        (Command command, bytes memory params) = makeCreateAgentCommand();

        // Expect the gateway to emit `InboundMessageDispatched`
        vm.expectEmit(true, false, false, false);
        emit IGateway.InboundMessageDispatched(assetHubParaID.into(), 1, messageID, true);

        hoax(relayer, 1 ether);
        IGateway(address(gateway)).submitV1(
            InboundMessage(assetHubParaID.into(), 1, command, params, maxDispatchGas, maxRefund, reward, messageID),
            proof,
            makeMockProof()
        );
    }

    function testSubmitFailInvalidNonce() public {
        deal(assetHubAgent, 50 ether);

        (Command command, bytes memory params) = makeCreateAgentCommand();

        hoax(relayer, 1 ether);
        IGateway(address(gateway)).submitV1(
            InboundMessage(assetHubParaID.into(), 1, command, params, maxDispatchGas, maxRefund, reward, messageID),
            proof,
            makeMockProof()
        );

        // try to replay the message
        vm.expectRevert(Gateway.InvalidNonce.selector);
        hoax(relayer, 1 ether);
        IGateway(address(gateway)).submitV1(
            InboundMessage(assetHubParaID.into(), 1, command, params, maxDispatchGas, maxRefund, reward, messageID),
            proof,
            makeMockProof()
        );
    }

    function testSubmitFailInvalidChannel() public {
        (Command command,) = makeCreateAgentCommand();

        vm.expectRevert(Gateway.ChannelDoesNotExist.selector);
        hoax(relayer);
        IGateway(address(gateway)).submitV1(
            InboundMessage(ParaID.wrap(42).into(), 1, command, "", maxDispatchGas, maxRefund, reward, messageID),
            proof,
            makeMockProof()
        );
    }

    function testSubmitFailInvalidProof() public {
        deal(assetHubAgent, 50 ether);

        (Command command, bytes memory params) = makeCreateAgentCommand();

        GatewayMock(address(gateway)).setCommitmentsAreVerified(false);
        vm.expectRevert(Gateway.InvalidProof.selector);

        hoax(relayer, 1 ether);
        IGateway(address(gateway)).submitV1(
            InboundMessage(assetHubParaID.into(), 1, command, params, maxDispatchGas, maxRefund, reward, messageID),
            proof,
            makeMockProof()
        );
    }

    /**
     * Fees & Rewards
     */

    // Message relayer should be rewarded from the agent for a channel
    function testRelayerRewardedFromAgent() public {
        (Command command, bytes memory params) = makeCreateAgentCommand();

        vm.txGasPrice(10 gwei);
        hoax(relayer, 1 ether);
        deal(assetHubAgent, 50 ether);

        uint256 relayerBalanceBefore = address(relayer).balance;
        uint256 agentBalanceBefore = address(assetHubAgent).balance;

        uint256 startGas = gasleft();
        IGateway(address(gateway)).submitV1(
            InboundMessage(assetHubParaID.into(), 1, command, params, maxDispatchGas, maxRefund, reward, messageID),
            proof,
            makeMockProof()
        );
        uint256 endGas = gasleft();
        uint256 estimatedActualRefundAmount = (startGas - endGas) * tx.gasprice;
        assertLt(estimatedActualRefundAmount, maxRefund);

        // Check that agent balance decreased and relayer balance increases
        assertLt(address(assetHubAgent).balance, agentBalanceBefore);
        assertGt(relayer.balance, relayerBalanceBefore);

        // The total amount paid to the relayer
        uint256 totalPaid = agentBalanceBefore - address(assetHubAgent).balance;

        // Since we know that the actual refund amount is less than the max refund,
        // the total amount paid to the relayer is less.
        assertLt(totalPaid, maxRefund + reward);
    }

    // In this case, the agent has no funds to reward the relayer
    function testRelayerNotRewarded() public {
        (Command command, bytes memory params) = makeCreateAgentCommand();

        hoax(relayer, 1 ether);
        IGateway(address(gateway)).submitV1(
            InboundMessage(assetHubParaID.into(), 1, command, params, maxDispatchGas, maxRefund, reward, messageID),
            proof,
            makeMockProof()
        );

        assertEq(address(assetHubAgent).balance, 0 ether);
        assertEq(relayer.balance, 1 ether);
    }

    // Users should pay fees to send outbound messages
    function testUserPaysFees() public {
        // Create a mock user
        address user = makeAddr("user");
        deal(address(token), user, 1);

        // register token first
        uint256 fee = IGatewayOutbound(address(gateway)).quoteRegisterTokenFee();
        IGatewayOutbound(address(gateway)).registerToken{value: fee}(address(token));

        fee = IGatewayOutbound(address(gateway)).quoteSendTokenFee(address(token), ParaID.wrap(0), 1);

        // Let gateway lock up to 1 tokens
        hoax(user);
        token.approve(address(gateway), 1);

        hoax(user, fee);
        IGatewayOutbound(address(gateway)).sendToken{value: fee}(
            address(token), ParaID.wrap(0), recipientAddress32, 1, 1
        );

        assertEq(user.balance, 0);
    }

    // User doesn't have enough funds to send message
    function testUserDoesNotProvideEnoughFees() public {
        // register token first
        uint256 fee = IGatewayOutbound(address(gateway)).quoteRegisterTokenFee();
        IGatewayOutbound(address(gateway)).registerToken{value: fee}(address(token));

        // Create a mock user
        address user = makeAddr("user");
        deal(address(token), user, 1);

        // Let gateway lock up to 1 tokens
        hoax(user);
        token.approve(address(gateway), 1);

        vm.expectRevert(GatewayOutbound.FeePaymentToLow.selector);
        hoax(user, 2 ether);
        IGatewayOutbound(address(gateway)).sendToken{value: 0.002 ether}(
            address(token), ParaID.wrap(0), recipientAddress32, 1, 1
        );

        assertEq(user.balance, 2 ether);
    }

    /**
     * Handlers
     */
    function testAgentExecution() public {
        token.transfer(address(assetHubAgent), 200);

        AgentExecuteParams memory params = AgentExecuteParams({
            agentID: assetHubAgentID,
            payload: abi.encode(AgentExecuteCommand.TransferToken, abi.encode(address(token), address(account2), 10))
        });

        bytes memory encodedParams = abi.encode(params);
        GatewayMock(address(gateway)).agentExecutePublic(encodedParams);
    }

    function testAgentExecutionBadOrigin() public {
        AgentExecuteParams memory params = AgentExecuteParams({
            agentID: bytes32(0),
            payload: abi.encode(keccak256("transferNativeToken"), abi.encode(address(token), address(this), 1))
        });

        vm.expectRevert(Gateway.AgentDoesNotExist.selector);
        GatewayMock(address(gateway)).agentExecutePublic(abi.encode(params));
    }

    function testAgentExecutionBadPayload() public {
        AgentExecuteParams memory params = AgentExecuteParams({agentID: assetHubAgentID, payload: ""});

        vm.expectRevert(Gateway.InvalidAgentExecutionPayload.selector);
        GatewayMock(address(gateway)).agentExecutePublic(abi.encode(params));
    }

    function testCreateAgent() public {
        bytes32 agentID = keccak256("123");
        CreateAgentParams memory params = CreateAgentParams({agentID: agentID});

        vm.expectEmit(false, false, false, false, address(gateway));
        emit IGateway.AgentCreated(agentID, address(0));

        GatewayMock(address(gateway)).createAgentPublic(abi.encode(params));
    }

    function testCreateAgentAlreadyCreated() public {
        bytes32 agentID = keccak256("123");
        CreateAgentParams memory params = CreateAgentParams({agentID: agentID});

        GatewayMock(address(gateway)).createAgentPublic(abi.encode(params));

        vm.expectRevert(Gateway.AgentAlreadyCreated.selector);
        GatewayMock(address(gateway)).createAgentPublic(abi.encode(params));
    }

    function testCreateChannel() public {
        ParaID paraID = ParaID.wrap(3042);
        bytes32 agentID = keccak256("3042");

        GatewayMock(address(gateway)).createAgentPublic(abi.encode(CreateAgentParams({agentID: agentID})));

        CreateChannelParams memory params =
            CreateChannelParams({channelID: paraID.into(), agentID: agentID, mode: OperatingMode.Normal});

        vm.expectEmit(true, false, false, true);
        emit IGateway.ChannelCreated(paraID.into());
        GatewayMock(address(gateway)).createChannelPublic(abi.encode(params));
    }

    function testCreateChannelFailsAgentDoesNotExist() public {
        ParaID paraID = ParaID.wrap(3042);
        bytes32 agentID = keccak256("3042");

        CreateChannelParams memory params =
            CreateChannelParams({channelID: paraID.into(), mode: OperatingMode.Normal, agentID: agentID});

        vm.expectRevert(Gateway.AgentDoesNotExist.selector);
        GatewayMock(address(gateway)).createChannelPublic(abi.encode(params));
    }

    function testCreateChannelFailsChannelAlreadyExists() public {
        ParaID paraID = ParaID.wrap(3042);
        bytes32 agentID = keccak256("3042");

        GatewayMock(address(gateway)).createAgentPublic(abi.encode(CreateAgentParams({agentID: agentID})));

        CreateChannelParams memory params =
            CreateChannelParams({channelID: paraID.into(), agentID: agentID, mode: OperatingMode.Normal});

        GatewayMock(address(gateway)).createChannelPublic(abi.encode(params));

        vm.expectRevert(Gateway.ChannelAlreadyCreated.selector);
        GatewayMock(address(gateway)).createChannelPublic(abi.encode(params));
    }

    function testUpdateChannel() public {
        // get current fee (0.0025 ether)
        PricingStorage.Layout storage pricing = PricingStorage.layout();
        uint256 fee = pricing.deliveryCost;

        bytes memory params = abi.encode(
            UpdateChannelParams({channelID: assetHubParaID.into(), mode: OperatingMode.RejectingOutboundMessages})
        );

        vm.expectEmit(true, false, false, true);
        emit IGateway.ChannelUpdated(assetHubParaID.into());
        GatewayMock(address(gateway)).updateChannelPublic(params);

        // Due to the new exchange rate, new fee is halved
        uint256 newFee = pricing.deliveryCost;
        assertEq(fee / 2, newFee);
    }

    function testUpdateChannelFailDoesNotExist() public {
        bytes memory params = abi.encode(
            UpdateChannelParams({channelID: ParaID.wrap(5956).into(), mode: OperatingMode.RejectingOutboundMessages})
        );

        vm.expectRevert(Gateway.ChannelDoesNotExist.selector);
        GatewayMock(address(gateway)).updateChannelPublic(params);
    }

    function testUpdateChannelSanityChecksForPrimaryGovernanceChannel() public {
        bytes memory params = abi.encode(
            UpdateChannelParams({
                channelID: ChannelID.wrap(bytes32(uint256(1))),
                mode: OperatingMode.RejectingOutboundMessages
            })
        );

        vm.expectRevert(Gateway.InvalidChannelUpdate.selector);
        GatewayMock(address(gateway)).updateChannelPublic(params);
    }

    function testUpgrade() public {
        // Upgrade to this new logic contract
        GatewayV2 newLogic = new GatewayV2();

        bytes4[] memory gatewayLogicV2Selectors = new bytes4[](1);
        gatewayLogicV2Selectors[0] = bytes4(keccak256(bytes("getValue()")));
        // Initialize facetCut
        DiamondStorage.FacetCut memory gatewayLogicFacetCut = DiamondStorage.FacetCut({
            facetAddress: address(newLogic),
            action: DiamondStorage.FacetCutAction.Add,
            functionSelectors: gatewayLogicV2Selectors
        });
        DiamondStorage.FacetCut[] memory facetCuts = new DiamondStorage.FacetCut[](1);
        facetCuts[0] = gatewayLogicFacetCut;

        UpgradeParams memory params = UpgradeParams({
            facetCuts: facetCuts,
            impl: address(newLogic),
            implCodeHash: address(newLogic).codehash,
            initParams: abi.encode(42)
        });

        // Expect the gateway to emit `Upgraded`
        vm.expectEmit(true, false, false, false);
        emit IGateway.Upgraded(address(newLogic));

        GatewayMock(address(gateway)).upgradePublic(abi.encode(params));

        // Verify that the GatewayV2.setup was called
        assertEq(GatewayV2(address(gateway)).getValue(), 42);
    }

    function testUpgradeInitializerRunsOnlyOnce() public {
        // Upgrade to this current logic contract
        AgentExecutor executor = new AgentExecutor();
        GatewayMock currentLogic = new GatewayMock();

        Gateway.Config memory config = Gateway.Config({
            beefyClient: address(0),
            agentExecutor: address(executor),
            bridgeHubParaID: bridgeHubParaID,
            bridgeHubAgentID: bridgeHubAgentID,
            foreignTokenDecimals: foreignTokenDecimals,
            mode: OperatingMode.Normal,
            deliveryCost: outboundFee,
            registerTokenFee: registerTokenFee,
            assetHubParaID: assetHubParaID,
            assetHubAgentID: assetHubAgentID,
            assetHubCreateAssetFee: createTokenFee,
            assetHubReserveTransferFee: sendTokenFee,
            exchangeRate: exchangeRate,
            multiplier: multiplier
        });

        DiamondStorage.FacetCut[] memory facetCuts;

        UpgradeParams memory params = UpgradeParams({
            facetCuts: facetCuts,
            impl: address(currentLogic),
            implCodeHash: address(currentLogic).codehash,
            initParams: abi.encode(config)
        });

        vm.expectRevert(Gateway.AlreadyInitialized.selector);
        // Expect the gateway to emit `Upgraded`
        GatewayMock(address(gateway)).upgradePublic(abi.encode(params));
    }

    function testUpgradeSkipsInitializerIfNoneProvided() public {
        bytes32 agentID = keccak256("123");

        testSetPricingParameters();
        uint256 fee = IGatewayOutbound(address(gateway)).quoteRegisterTokenFee();
        assertEq(fee, 20000000000000001);

        testCreateAgent();
        assertNotEq(GatewayMock(address(gateway)).agentOf(agentID), address(0));

        // Upgrade to this current logic contract
        GatewayMock currentLogic = new GatewayMock();

        bytes memory initParams; // empty
        DiamondStorage.FacetCut[] memory facetCuts;
        UpgradeParams memory params = UpgradeParams({
            facetCuts: facetCuts,
            impl: address(currentLogic),
            implCodeHash: address(currentLogic).codehash,
            initParams: initParams
        });

        // Expect the gateway to emit `Upgraded`
        GatewayMock(address(gateway)).upgradePublic(abi.encode(params));

        // Verify that storage was not overwritten
        fee = IGatewayOutbound(address(gateway)).quoteRegisterTokenFee();
        assertEq(fee, 20000000000000001);
        assertNotEq(GatewayMock(address(gateway)).agentOf(agentID), address(0));
    }

    function testUpgradeGatewayMock() public {
        GatewayUpgradeMock newLogic = new GatewayUpgradeMock();
        uint256 d0 = 99;
        uint256 d1 = 66;
        bytes memory initParams = abi.encode(d0, d1);
        console.logBytes(initParams);

        DiamondStorage.FacetCut[] memory facetCuts;
        UpgradeParams memory params = UpgradeParams({
            facetCuts: facetCuts,
            impl: address(newLogic),
            implCodeHash: address(newLogic).codehash,
            initParams: initParams
        });

        // Expect the gateway to emit `Initialized`
        vm.expectEmit(true, false, false, true);
        emit GatewayUpgradeMock.Initialized(d0, d1);

        GatewayMock(address(gateway)).upgradePublic(abi.encode(params));
    }

    function testUpgradeFailOnInitializationFailure() public {
        GatewayV2 newLogic = new GatewayV2();

        DiamondStorage.FacetCut[] memory facetCuts;
        UpgradeParams memory params = UpgradeParams({
            facetCuts: facetCuts,
            impl: address(newLogic),
            implCodeHash: address(newLogic).codehash,
            initParams: abi.encode(666)
        });

        vm.expectRevert("initialize failed");
        GatewayMock(address(gateway)).upgradePublic(abi.encode(params));
    }

    function testUpgradeFailCodeHashMismatch() public {
        GatewayV2 newLogic = new GatewayV2();

        DiamondStorage.FacetCut[] memory facetCuts;
        UpgradeParams memory params = UpgradeParams({
            facetCuts: facetCuts,
            impl: address(newLogic),
            implCodeHash: bytes32(0),
            initParams: abi.encode(42)
        });

        vm.expectRevert(Gateway.InvalidCodeHash.selector);
        GatewayMock(address(gateway)).upgradePublic(abi.encode(params));
    }

    function testSetOperatingMode() public {
        SetOperatingModeParams memory params = SetOperatingModeParams({mode: OperatingMode.RejectingOutboundMessages});

        OperatingMode mode = IGateway(address(gateway)).operatingMode();
        assertEq(uint256(mode), 0);

        GatewayMock(address(gateway)).setOperatingModePublic(abi.encode(params));

        mode = IGateway(address(gateway)).operatingMode();
        assertEq(uint256(mode), 1);
    }

    function testWithdrawAgentFunds() public {
        deal(assetHubAgent, 50 ether);

        address recipient = makeAddr("recipient");

        bytes memory params =
            abi.encode(TransferNativeFromAgentParams({agentID: assetHubAgentID, recipient: recipient, amount: 3 ether}));

        GatewayMock(address(gateway)).transferNativeFromAgentPublic(params);

        assertEq(assetHubAgent.balance, 47 ether);
        assertEq(recipient.balance, 3 ether);
    }

    /**
     * Assets
     */
    function testRegisterToken() public {
        vm.expectEmit(false, false, false, true);
        emit IGatewayOutbound.TokenRegistrationSent(address(token));

        vm.expectEmit(true, false, false, false);
        emit IGatewayOutbound.OutboundMessageAccepted(assetHubParaID.into(), 1, messageID, bytes(""));

        IGatewayOutbound(address(gateway)).registerToken{value: 2 ether}(address(token));
    }

    function testRegisterTokenReimbursesExcessFees() public {
        vm.expectEmit(false, false, false, true);
        emit IGatewayOutbound.TokenRegistrationSent(address(token));

        vm.expectEmit(true, false, false, false);
        emit IGatewayOutbound.OutboundMessageAccepted(assetHubParaID.into(), 1, messageID, bytes(""));

        uint256 totalFee = IGatewayOutbound(address(gateway)).quoteRegisterTokenFee();

        uint256 balanceBefore = address(this).balance;
        IGatewayOutbound(address(gateway)).registerToken{value: totalFee + 1 ether}(address(token));
        uint256 balanceAfter = address(this).balance;

        // Check that the balance has decreased by the amount of gas used
        // channel.fee is baseFee & extraFee is registerNativeTokenFee
        uint256 etherUsed = balanceBefore - balanceAfter;
        assert(etherUsed == totalFee);
    }

    function testSendTokenAddress32() public {
        // Let gateway lock up to 1 tokens
        token.approve(address(gateway), 1);

        // Multilocation for recipient
        ParaID destPara = ParaID.wrap(2043);

        // register token first
        uint256 fee = IGatewayOutbound(address(gateway)).quoteRegisterTokenFee();
        IGatewayOutbound(address(gateway)).registerToken{value: fee}(address(token));

        fee = IGatewayOutbound(address(gateway)).quoteSendTokenFee(address(token), destPara, 1);

        vm.expectEmit(true, true, false, true);
        emit IGatewayOutbound.TokenSent(address(token), address(this), destPara, recipientAddress32, 1);

        // Expect the gateway to emit `OutboundMessageAccepted`
        vm.expectEmit(true, false, false, false);
        emit IGatewayOutbound.OutboundMessageAccepted(assetHubParaID.into(), 1, messageID, bytes(""));

        IGatewayOutbound(address(gateway)).sendToken{value: fee}(address(token), destPara, recipientAddress32, 1, 1);
    }

    function testSendTokenAddress32ToAssetHub() public {
        // Let gateway lock up to 1 tokens
        token.approve(address(gateway), 1);

        // Multilocation for recipient
        ParaID destPara = assetHubParaID;

        // register token first
        uint256 fee = IGatewayOutbound(address(gateway)).quoteRegisterTokenFee();
        IGatewayOutbound(address(gateway)).registerToken{value: fee}(address(token));

        fee = IGatewayOutbound(address(gateway)).quoteSendTokenFee(address(token), destPara, 1);

        vm.expectEmit(true, true, false, true);
        emit IGatewayOutbound.TokenSent(address(token), address(this), destPara, recipientAddress32, 1);

        // Expect the gateway to emit `OutboundMessageAccepted`
        vm.expectEmit(true, false, false, false);
        emit IGatewayOutbound.OutboundMessageAccepted(assetHubParaID.into(), 1, messageID, bytes(""));

        IGatewayOutbound(address(gateway)).sendToken{value: fee}(address(token), destPara, recipientAddress32, 1, 1);
    }

    function testSendTokenAddress20() public {
        // Let gateway lock up to 1 tokens
        token.approve(address(gateway), 1);

        // Multilocation for recipient
        ParaID destPara = ParaID.wrap(2043);

        // register token first
        uint256 fee = IGatewayOutbound(address(gateway)).quoteRegisterTokenFee();
        IGatewayOutbound(address(gateway)).registerToken{value: fee}(address(token));

        fee = IGatewayOutbound(address(gateway)).quoteSendTokenFee(address(token), destPara, 1);

        vm.expectEmit(true, true, false, true);
        emit IGatewayOutbound.TokenSent(address(token), address(this), destPara, recipientAddress20, 1);

        // Expect the gateway to emit `OutboundMessageAccepted`
        vm.expectEmit(true, false, false, false);
        emit IGatewayOutbound.OutboundMessageAccepted(assetHubParaID.into(), 1, messageID, bytes(""));

        IGatewayOutbound(address(gateway)).sendToken{value: fee}(address(token), destPara, recipientAddress20, 1, 1);
    }

    function testSendTokenAddress20FailsInvalidDestination() public {
        // Let gateway lock up to 1 tokens
        token.approve(address(gateway), 1);

        ParaID destPara = assetHubParaID;

        // register token first
        uint256 fee = IGatewayOutbound(address(gateway)).quoteRegisterTokenFee();
        IGatewayOutbound(address(gateway)).registerToken{value: fee}(address(token));

        // Should fail to send tokens to AssetHub
        vm.expectRevert(Assets.Unsupported.selector);
        IGatewayOutbound(address(gateway)).sendToken{value: 2 ether}(address(token), destPara, recipientAddress20, 1, 1);
    }

    /**
     * Operating Modes
     */
    function testDisableOutboundMessaging() public {
        // Let gateway lock up to 1 tokens
        token.approve(address(gateway), 1);

        GatewayMock(address(gateway)).setOperatingModePublic(
            abi.encode(SetOperatingModeParams({mode: OperatingMode.RejectingOutboundMessages}))
        );

        OperatingMode mode = IGateway(address(gateway)).operatingMode();
        assertEq(uint256(mode), 1);
    }

    function testDisableOutboundMessagingForChannel() public {
        // Let gateway lock up to 1 tokens
        token.approve(address(gateway), 1);

        GatewayMock(address(gateway)).setOperatingModePublic(
            abi.encode(SetOperatingModeParams({mode: OperatingMode.Normal}))
        );

        // register token first
        uint256 fee = IGatewayOutbound(address(gateway)).quoteRegisterTokenFee();
        IGatewayOutbound(address(gateway)).registerToken{value: fee}(address(token));

        bytes memory params = abi.encode(
            UpdateChannelParams({channelID: assetHubParaID.into(), mode: OperatingMode.RejectingOutboundMessages})
        );
        GatewayMock(address(gateway)).updateChannelPublic(params);

        OperatingMode mode = IGateway(address(gateway)).channelOperatingModeOf(assetHubParaID.into());
        assertEq(uint256(mode), 1);

        // Now all outbound messaging should be disabled

        vm.expectRevert(GatewayOutbound.Disabled.selector);
        IGatewayOutbound(address(gateway)).registerToken{value: 1 ether}(address(token));

        vm.expectRevert(GatewayOutbound.Disabled.selector);
        IGatewayOutbound(address(gateway)).sendToken{value: 1 ether}(
            address(token), ParaID.wrap(0), recipientAddress32, 1, 1
        );
    }

    /**
     * Misc checks
     */

    // Initialize function should not be externally callable on either proxy or implementation contract
    function testInitializeNotExternallyCallable() public {
        vm.expectRevert(Gateway.Unauthorized.selector);
        Gateway(address(gateway)).initialize("");

        vm.expectRevert(Gateway.Unauthorized.selector);
        GatewayMock(address(gatewayLogic)).initialize("");
    }

    // Handler functions should not be externally callable
    function testHandlersNotExternallyCallable() public {
        vm.expectRevert(Gateway.Unauthorized.selector);
        Gateway(address(gateway)).agentExecute("");

        vm.expectRevert(Gateway.Unauthorized.selector);
        Gateway(address(gateway)).createAgent("");

        vm.expectRevert(Gateway.Unauthorized.selector);
        Gateway(address(gateway)).createChannel("");

        vm.expectRevert(Gateway.Unauthorized.selector);
        Gateway(address(gateway)).updateChannel("");

        vm.expectRevert(Gateway.Unauthorized.selector);
        Gateway(address(gateway)).setOperatingMode("");

        vm.expectRevert(Gateway.Unauthorized.selector);
        Gateway(address(gateway)).upgrade("");

        vm.expectRevert(Gateway.Unauthorized.selector);
        Gateway(address(gateway)).transferNativeFromAgent("");
    }

    function testGetters() public {
        IGateway gw = IGateway(address(gateway));

        OperatingMode mode = gw.operatingMode();
        assertEq(uint256(mode), 0);

        OperatingMode channelMode = gw.channelOperatingModeOf(assetHubParaID.into());
        assertEq(uint256(channelMode), 0);

        (, uint128 fee) = gw.pricingParameters();
        assertEq(fee, 10000000000);

        (uint64 inbound, uint64 outbound) = gw.channelNoncesOf(assetHubParaID.into());
        assertEq(inbound, 0);
        assertEq(outbound, 0);

        address agent = gw.agentOf(assetHubAgentID);
        assertEq(agent, assetHubAgent);
    }

    function testCreateAgentWithNotEnoughGas() public {
        deal(assetHubAgent, 50 ether);

        (Command command, bytes memory params) = makeCreateAgentCommand();

        hoax(relayer, 1 ether);

        vm.expectEmit(true, false, false, true);
        // Expect dispatch result as false for `OutOfGas`
        emit IGateway.InboundMessageDispatched(assetHubParaID.into(), 1, messageID, false);
        // maxDispatchGas as 1 for `create_agent` is definitely not enough
        IGateway(address(gateway)).submitV1(
            InboundMessage(assetHubParaID.into(), 1, command, params, 1, maxRefund, reward, messageID),
            proof,
            makeMockProof()
        );
    }

    function testSetTokenFees() public {
        uint256 fee = IGatewayOutbound(address(gateway)).quoteRegisterTokenFee();
        assertEq(fee, 5000000000000000);
        // Double the assetHubCreateAssetFee
        GatewayMock(address(gateway)).setTokenTransferFeesPublic(
            abi.encode(
                SetTokenTransferFeesParams({
                    assetHubCreateAssetFee: createTokenFee * 2,
                    registerTokenFee: registerTokenFee,
                    assetHubReserveTransferFee: sendTokenFee
                })
            )
        );
        fee = IGatewayOutbound(address(gateway)).quoteRegisterTokenFee();
        // since deliveryCost not changed, so the total fee increased only by 50%
        assertEq(fee, 7500000000000000);
    }

    bytes32 public expectChannelIDBytes = bytes32(0xc173fac324158e77fb5840738a1a541f633cbec8884c6a601c567d2b376a0539);

    function testDeriveChannelID() public {
        ParaID para_id = ParaID.wrap(1000);
        ChannelID channel_id = para_id.into();
        assertEq(ChannelID.unwrap(channel_id), expectChannelIDBytes);
    }

    function testSetPricingParameters() public {
        uint256 fee = IGatewayOutbound(address(gateway)).quoteRegisterTokenFee();
        assertEq(fee, 5000000000000000);
        // Double both the exchangeRate and multiplier. Should lead to an 4x fee increase
        GatewayMock(address(gateway)).setPricingParametersPublic(
            abi.encode(
                SetPricingParametersParams({
                    exchangeRate: exchangeRate.mul(convert(2)),
                    multiplier: multiplier.mul(convert(2)),
                    deliveryCost: outboundFee
                })
            )
        );
        // Should expect 4x fee increase
        fee = IGatewayOutbound(address(gateway)).quoteRegisterTokenFee();
        assertEq(fee, 20000000000000001);
    }

    function testSendTokenToForeignDestWithInvalidFee() public {
        // Let gateway lock up to 1 tokens
        token.approve(address(gateway), 1);

        // Multilocation for recipient
        ParaID destPara = ParaID.wrap(2043);

        // register token first
        uint256 fee = IGatewayOutbound(address(gateway)).quoteRegisterTokenFee();
        IGatewayOutbound(address(gateway)).registerToken{value: fee}(address(token));

        fee = IGatewayOutbound(address(gateway)).quoteSendTokenFee(address(token), destPara, 0);

        vm.expectRevert(Assets.InvalidDestinationFee.selector);
        IGatewayOutbound(address(gateway)).sendToken{value: fee}(address(token), destPara, recipientAddress32, 0, 1);
    }

    function testAgentRegisterDot() public {
        AgentExecuteParams memory params = AgentExecuteParams({
            agentID: assetHubAgentID,
            payload: abi.encode(AgentExecuteCommand.RegisterToken, abi.encode(dotTokenID, "DOT", "DOT", 10))
        });

        vm.expectEmit(true, true, false, false);
        emit IGateway.ForeignTokenRegistered(bytes32(uint256(1)), assetHubAgentID, address(0));

        GatewayMock(address(gateway)).agentExecutePublic(abi.encode(params));
    }

    function testAgentMintDot() public {
        testAgentRegisterDot();

        AgentExecuteParams memory params = AgentExecuteParams({
            agentID: assetHubAgentID,
            payload: abi.encode(AgentExecuteCommand.MintToken, abi.encode(bytes32(uint256(1)), account1, 1000))
        });

        vm.expectEmit(true, true, false, false);
        emit TokenMinted(bytes32(uint256(1)), address(0), account1, 1000);

        GatewayMock(address(gateway)).agentExecutePublic(abi.encode(params));
    }

    function testTransferDotToAssetHub() public {
        // Register and then mint some DOT to account1
        testAgentMintDot();

        TokenInfo memory info = IGatewayOutbound(address(gateway)).getTokenInfo(dotTokenID);

        ParaID destPara = assetHubParaID;

        vm.prank(account1);

        vm.expectEmit(true, true, false, true);
        emit IGatewayOutbound.TokenTransfered(address(info.token), account1, destPara, recipientAddress32, 1);

        // Expect the gateway to emit `OutboundMessageAccepted`
        vm.expectEmit(true, false, false, false);
        emit IGatewayOutbound.OutboundMessageAccepted(assetHubParaID.into(), 1, messageID, bytes(""));

        IGatewayOutbound(address(gateway)).transferToken{value: 0.1 ether}(
            address(info.token), destPara, recipientAddress32, 1, 1
        );
    }
}

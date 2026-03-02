// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.33;

// Standalone unit tests for Agent -> SnowbridgeL1Adaptor flow (no GatewayV2Test inheritance
// to avoid stack-too-deep when compiling without via_ir).

import {Test} from "forge-std/Test.sol";
import {IGatewayV2} from "../src/v2/IGateway.sol";
import {GatewayProxy} from "../src/GatewayProxy.sol";
import {MockGateway} from "./mocks/MockGateway.sol";
import {AgentExecutor} from "../src/AgentExecutor.sol";
import {Initializer} from "../src/Initializer.sol";
import {Constants} from "../src/Constants.sol";
import {SetOperatingModeParams} from "../src/v2/Types.sol";
import {OperatingMode} from "../src/Types.sol";
import {Verification} from "../src/Verification.sol";
import {WETH9} from "canonical-weth/WETH9.sol";
import {UD60x18, ud60x18} from "prb/math/src/UD60x18.sol";
import {SnowbridgeL1Adaptor} from "../src/l2-integration/SnowbridgeL1Adaptor.sol";
import {SnowbridgeL2Adaptor} from "../src/l2-integration/SnowbridgeL2Adaptor.sol";
import {DepositParams, SendParams, SwapParams} from "../src/l2-integration/Types.sol";
import {MockSpokePool, MockSpokePoolReverting} from "./mocks/MockSpokePool.sol";
import {MockMessageHandler} from "./mocks/MockMessageHandler.sol";
import {SnowbridgeL2TestLib} from "./GatewayV2SnowbridgeL2TestLib.sol";

contract GatewayV2SnowbridgeL2Test is Test {
    address public assetHubAgent;
    address public relayer;
    bytes32 public relayerRewardAddress = keccak256("relayerRewardAddress");
    bytes32[] public proof =
        [bytes32(0x2f9ee6cfdf244060dc28aa46347c5219e303fc95062dd672b4e406ca5c29764b)];

    MockGateway public gatewayLogic;
    GatewayProxy public gateway;
    WETH9 public weth;
    address public user1;

    function setUp() public {
        weth = new WETH9();
        AgentExecutor executor = new AgentExecutor();
        gatewayLogic = new MockGateway(address(0), address(executor));
        Initializer.Config memory config = Initializer.Config({
            mode: OperatingMode.Normal,
            deliveryCost: 1e10,
            registerTokenFee: 0,
            assetHubCreateAssetFee: 1e10,
            assetHubReserveTransferFee: 1e10,
            exchangeRate: ud60x18(0.0025e18),
            multiplier: ud60x18(1e18),
            foreignTokenDecimals: 10,
            maxDestinationFee: 1e11
        });
        gateway = new GatewayProxy(address(gatewayLogic), abi.encode(config));
        MockGateway(address(gateway)).setCommitmentsAreVerified(true);

        SetOperatingModeParams memory params = SetOperatingModeParams({mode: OperatingMode.Normal});
        MockGateway(address(gateway)).v1_handleSetOperatingMode_public(abi.encode(params));

        assetHubAgent = IGatewayV2(address(gateway)).agentOf(Constants.ASSET_HUB_AGENT_ID);
        relayer = makeAddr("relayer");
        user1 = makeAddr("user1");

        hoax(user1);
        weth.deposit{value: 1 ether}();
    }

    function _makeMockProof() internal pure returns (Verification.Proof memory) {
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

    function _deployL1AdaptorWithMockSpokePool()
        internal
        returns (MockSpokePool mockSpokePool, SnowbridgeL1Adaptor adaptor)
    {
        mockSpokePool = new MockSpokePool();
        adaptor = new SnowbridgeL1Adaptor(address(mockSpokePool), address(weth), address(gateway));
    }

    function _deployL1AdaptorWithRevertingSpokePool()
        internal
        returns (MockSpokePoolReverting mockSpokePool, SnowbridgeL1Adaptor adaptor)
    {
        mockSpokePool = new MockSpokePoolReverting();
        adaptor = new SnowbridgeL1Adaptor(address(mockSpokePool), address(weth), address(gateway));
    }

    function _deployL2AdaptorWithMockSpokePool()
        internal
        returns (MockSpokePool mockSpokePool, SnowbridgeL2Adaptor adaptor)
    {
        mockSpokePool = new MockSpokePool();
        MockMessageHandler handler = new MockMessageHandler();
        adaptor = new SnowbridgeL2Adaptor(
            address(mockSpokePool),
            address(handler),
            address(gateway),
            address(weth),
            address(weth)
        );
    }

    function testAgentCallsSnowbridgeL1AdaptorDepositTokenSuccess() public {
        (MockSpokePool mockSpokePool, SnowbridgeL1Adaptor adaptor) =
            _deployL1AdaptorWithMockSpokePool();
        uint256 inputAmount = 1 ether;
        address recipient = makeAddr("recipient");

        DepositParams memory params =
            SnowbridgeL2TestLib.makeDepositParamsToken(address(weth), inputAmount, 0.9 ether);
        bytes32 topic = keccak256("snowbridge-topic");

        // Fund the agent so UnlockNativeToken can transfer to the adaptor
        hoax(user1);
        weth.transfer(assetHubAgent, inputAmount);

        hoax(relayer, 1 ether);
        IGatewayV2(address(gateway))
            .v2_submit(
                SnowbridgeL2TestLib.makeDepositTokenMessageWithPrefund(
                    address(adaptor), params, recipient, topic, address(weth), uint128(inputAmount)
                ),
                proof,
                _makeMockProof(),
                relayerRewardAddress
            );

        assertEq(mockSpokePool.numberOfDeposits(), 1);
        assertEq(weth.balanceOf(recipient), inputAmount);
    }

    function testAgentCallsSnowbridgeL1AdaptorDepositTokenRevertsWhenCallerNotAgent() public {
        (, SnowbridgeL1Adaptor adaptor) = _deployL1AdaptorWithMockSpokePool();
        uint256 inputAmount = 1 ether;
        hoax(user1);
        weth.transfer(address(adaptor), inputAmount);

        DepositParams memory params =
            SnowbridgeL2TestLib.makeDepositParamsToken(address(weth), inputAmount, 0.9 ether);
        bytes32 topic = keccak256("topic");
        address recipient = makeAddr("recipient");

        vm.expectRevert();
        hoax(user1);
        adaptor.depositToken(params, recipient, topic);
    }

    function testAgentCallsSnowbridgeL1AdaptorDepositTokenSpokePoolRevertsEmitsDepositCallFailed()
        public
    {
        (MockSpokePoolReverting mockSpokePool, SnowbridgeL1Adaptor adaptor) =
            _deployL1AdaptorWithRevertingSpokePool();
        uint256 inputAmount = 1 ether;
        address recipient = makeAddr("recipient");
        hoax(user1);
        weth.transfer(address(adaptor), inputAmount);

        DepositParams memory params =
            SnowbridgeL2TestLib.makeDepositParamsToken(address(weth), inputAmount, 0.9 ether);
        bytes32 topic = keccak256("topic");

        hoax(relayer, 1 ether);
        IGatewayV2(address(gateway))
            .v2_submit(
                SnowbridgeL2TestLib.makeDepositTokenMessage(
                    address(adaptor), params, recipient, topic
                ),
                proof,
                _makeMockProof(),
                relayerRewardAddress
            );

        assertEq(mockSpokePool.numberOfDeposits(), 0);
        assertEq(weth.balanceOf(recipient), inputAmount);
    }

    function testAgentCallsSnowbridgeL1AdaptorDepositNativeEtherSuccess() public {
        (MockSpokePool mockSpokePool, SnowbridgeL1Adaptor adaptor) =
            _deployL1AdaptorWithMockSpokePool();
        uint256 inputAmount = 0.5 ether;
        address recipient = makeAddr("recipient");

        DepositParams memory params =
            SnowbridgeL2TestLib.makeDepositParamsNativeEther(inputAmount, 0.4 ether);
        bytes32 topic = keccak256("snowbridge-native");

        vm.deal(assetHubAgent, inputAmount);

        hoax(relayer, 1 ether);
        IGatewayV2(address(gateway))
            .v2_submit(
                SnowbridgeL2TestLib.makeDepositNativeEtherMessageWithPrefund(
                    address(adaptor), params, recipient, topic, uint128(inputAmount)
                ),
                proof,
                _makeMockProof(),
                relayerRewardAddress
            );

        assertEq(mockSpokePool.numberOfDeposits(), 1);
        assertEq(address(mockSpokePool).balance, inputAmount);
    }

    function testL2AdaptorSendEtherAndCallSuccess() public {
        (MockSpokePool mockSpokePool, SnowbridgeL2Adaptor adaptor) =
            _deployL2AdaptorWithMockSpokePool();
        uint256 inputAmount = 1 ether;
        uint128 executionFee = 0.05 ether;
        uint128 relayerFee = 0.05 ether;
        uint256 outputAmount = 0.88 ether; // so totalOutputAmount = 0.88 + 0.05 + 0.05 = 0.98 < inputAmount
        address recipient = makeAddr("recipient");
        bytes32 topic = keccak256("l2-ether");

        DepositParams memory params = DepositParams({
            inputToken: address(0),
            outputToken: address(0x1234),
            inputAmount: inputAmount,
            outputAmount: outputAmount,
            destinationChainId: 8453,
            fillDeadlineBuffer: 600
        });
        SendParams memory sendParams = SendParams({
            xcm: "",
            assets: new bytes[](0),
            claimer: "",
            executionFee: executionFee,
            relayerFee: relayerFee
        });

        vm.expectEmit(true, true, false, true);
        emit SnowbridgeL2Adaptor.DepositCallInvoked(topic, 0);

        hoax(user1, inputAmount);
        adaptor.sendEtherAndCall{value: inputAmount}(params, sendParams, recipient, topic);

        assertEq(mockSpokePool.numberOfDeposits(), 1);
    }

    function testL2AdaptorSendTokenAndCallSuccess() public {
        (MockSpokePool mockSpokePool, SnowbridgeL2Adaptor adaptor) =
            _deployL2AdaptorWithMockSpokePool();
        uint256 inputAmount = 1 ether;
        uint256 outputAmount = 0.8 ether;
        uint256 swapInputAmount = 0.1 ether;
        address recipient = makeAddr("recipient");
        bytes32 topic = keccak256("l2-token");

        DepositParams memory params = DepositParams({
            inputToken: address(weth),
            outputToken: address(weth),
            inputAmount: inputAmount,
            outputAmount: outputAmount,
            destinationChainId: 8453,
            fillDeadlineBuffer: 600
        });
        SwapParams memory swapParams = SwapParams({
            inputAmount: swapInputAmount,
            router: address(0x1234),
            callData: hex"01"
        });
        SendParams memory sendParams = SendParams({
            xcm: "",
            assets: new bytes[](0),
            claimer: "",
            executionFee: 0.05 ether,
            relayerFee: 0.05 ether
        });

        hoax(user1);
        weth.approve(address(adaptor), inputAmount);

        vm.expectEmit(true, true, false, true);
        emit SnowbridgeL2Adaptor.DepositCallInvoked(topic, 0);

        hoax(user1);
        adaptor.sendTokenAndCall(params, swapParams, sendParams, recipient, topic);

        assertEq(mockSpokePool.numberOfDeposits(), 1);
    }

    function testL2AdaptorSendEtherAndCallRevertsWhenInputTokenNotZeroOrWeth() public {
        (, SnowbridgeL2Adaptor adaptor) = _deployL2AdaptorWithMockSpokePool();
        address recipient = makeAddr("recipient");
        address invalidToken = address(0x1234); // neither address(0) nor L2_WETH9

        DepositParams memory params = DepositParams({
            inputToken: invalidToken,
            outputToken: address(0x1234),
            inputAmount: 1 ether,
            outputAmount: 0.88 ether,
            destinationChainId: 8453,
            fillDeadlineBuffer: 600
        });
        SendParams memory sendParams = SendParams({
            xcm: "",
            assets: new bytes[](0),
            claimer: "",
            executionFee: 0.05 ether,
            relayerFee: 0.05 ether
        });

        vm.expectRevert(
            "Input token must be zero address or L2 WETH address for native ETH deposits"
        );
        adaptor.sendEtherAndCall(params, sendParams, recipient, keccak256("topic"));
    }

    function testL2AdaptorSendEtherAndCallRevertsWhenWethDepositWithValue() public {
        (, SnowbridgeL2Adaptor adaptor) = _deployL2AdaptorWithMockSpokePool();
        uint256 inputAmount = 1 ether;
        address recipient = makeAddr("recipient");

        DepositParams memory params = DepositParams({
            inputToken: address(weth), // WETH path
            outputToken: address(0x1234),
            inputAmount: inputAmount,
            outputAmount: 0.88 ether,
            destinationChainId: 8453,
            fillDeadlineBuffer: 600
        });
        SendParams memory sendParams = SendParams({
            xcm: "",
            assets: new bytes[](0),
            claimer: "",
            executionFee: 0.05 ether,
            relayerFee: 0.05 ether
        });

        hoax(user1);
        weth.approve(address(adaptor), inputAmount);

        vm.expectRevert("Sent value must be zero for WETH deposits");
        adaptor.sendEtherAndCall{value: 1 ether}(params, sendParams, recipient, keccak256("topic"));
    }
}

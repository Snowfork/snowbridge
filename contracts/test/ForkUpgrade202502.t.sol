// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {Test} from "forge-std/Test.sol";
import {console} from "forge-std/console.sol";

import {IUpgradable} from "../src/interfaces/IUpgradable.sol";
import {IGateway} from "../src/interfaces/IGateway.sol";
import {Verification} from "../src/Verification.sol";
import {Gateway} from "../src/Gateway.sol";
import {Gateway202502} from "../src/upgrades/Gateway202502.sol";
import {AgentExecutor} from "../src/AgentExecutor.sol";
import {UpgradeParams, SetOperatingModeParams, OperatingMode, RegisterForeignTokenParams} from "../src/Params.sol";
import {ChannelID, ParaID, OperatingMode, TokenInfo} from "../src/Types.sol";
import {MultiAddress, multiAddressFromBytes32} from "../src/MultiAddress.sol";

import {ForkTestUtils} from "./mocks/ForkTestUtils.sol";

contract ForkUpgradeTest is Test {

    address private constant GATEWAY_PROXY = 0x27ca963C279c93801941e1eB8799c23f407d68e7;
    address private constant BEEFY_CLIENT = 0x6eD05bAa904df3DE117EcFa638d4CB84e1B8A00C;
    address private constant VERIFICATION_ADDR = 0x515c0817005b2F3383B7D8837d6DCc15c0d71C56;
    bytes32 private constant BRIDGE_HUB_AGENT_ID = 0x03170a2e7597b7b7e3d84c05391d139a62b157e78786d8c082f29dcf4c111314;


    function setUp() public {
        vm.createSelectFork("https://rpc.tenderly.co/fork/b77e07b8-ad6d-4e83-b5be-30a2001964aa", 20645700);

        // Mock call to Verification.verifyCommitment to bypass BEEFY verification
        vm.mockCall(VERIFICATION_ADDR, abi.encodeWithSelector(Verification.verifyCommitment.selector), abi.encode(true));

        Gateway202502 newLogic = new Gateway202502(
            BEEFY_CLIENT,
            address(new AgentExecutor()),
            ParaID.wrap(1002),
            BRIDGE_HUB_AGENT_ID,
            10,
            20_000_000_000 // 2 DOT
        );

        vm.expectEmit();
        emit IUpgradable.Upgraded(address(newLogic));

        ForkTestUtils.upgrade(GATEWAY_PROXY, address(newLogic), bytes(""));

        // Now that the linked `Verification` library has been redeployed, reapply the mock
        vm.mockCall(address(Verification), abi.encodeWithSelector(Verification.verifyCommitment.selector), abi.encode(true));
    }

    function testRegisterForeignToken() public {
        bytes32 foreignTokenID = 0xa8f2ec5bdd7a07d844ee3bce83f9ba3881f495d96f07cacbeeb77d9e031db4f0;

        vm.expectEmit(true, false, false, true);
        emit IGateway.ForeignTokenRegistered(foreignTokenID, 0x196C20DA81Fbc324EcdF55501e95Ce9f0bD84d14);

        ForkTestUtils.registerForeignToken(GATEWAY_PROXY, foreignTokenID, "DOT", "DOT", 10);

        assert(IGateway(GATEWAY_PROXY).isTokenRegistered(0x196C20DA81Fbc324EcdF55501e95Ce9f0bD84d14));
        assertEq(IGateway(GATEWAY_PROXY).queryForeignTokenID(0x196C20DA81Fbc324EcdF55501e95Ce9f0bD84d14), foreignTokenID);
    }

    function testSanityCheck() public {
        // Check AH channel nonces as expected
        (uint64 inbound, uint64 outbound) = IGateway(GATEWAY_PROXY).channelNoncesOf(
            ChannelID.wrap(0xc173fac324158e77fb5840738a1a541f633cbec8884c6a601c567d2b376a0539)
        );
        assertEq(inbound, 13);
        assertEq(outbound, 172);
        // Check legacy ethereum token not affected
        checkLegacyToken();
        // Check sending of ether works
        checkSendingEthWithAmountAndFeeSucceeds();
    }

    function checkLegacyToken() public {
        assert(IGateway(GATEWAY_PROXY).isTokenRegistered(0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2));
        assertEq(IGateway(GATEWAY_PROXY).queryForeignTokenID(0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2), bytes32(""));
        assert(IGateway(GATEWAY_PROXY).isTokenRegistered(0xBA41Ddf06B7fFD89D1267b5A93BFeF2424eb2003));
        assertEq(IGateway(GATEWAY_PROXY).queryForeignTokenID(0xBA41Ddf06B7fFD89D1267b5A93BFeF2424eb2003), bytes32(""));
    }

    function checkSendingEthWithAmountAndFeeSucceeds() public {
        // Create a mock user
        address user = makeAddr("user");
        uint128 amount = 1;
        ParaID paraID = ParaID.wrap(1000);
        MultiAddress memory recipientAddress32 = multiAddressFromBytes32(keccak256("recipient"));

        uint128 fee = uint128(IGateway(GATEWAY_PROXY).quoteSendTokenFee(address(0), paraID, 1));

        vm.expectEmit();
        emit IGateway.TokenSent(address(0), user, paraID, recipientAddress32, amount);

        uint64 nonce = 173;
        bytes32 messageID = keccak256(abi.encodePacked(paraID.into(), nonce));

        vm.expectEmit();
        emit IGateway.OutboundMessageAccepted(
            paraID.into(),
            nonce,
            messageID,
            hex"00010000000000000001000000000000000000000000000000000000000000811085f5b5d1b29598e73ca51de3d712f5d3103ad50e22dc1f4d3ff1559d51150100000000000000000000000000000000ca9a3b000000000000000000000000"
        );

        deal(user, amount + fee);
        vm.startPrank(user);
        IGateway(GATEWAY_PROXY).sendToken{value: amount + fee}(address(0), paraID, recipientAddress32, 1, amount);

        assertEq(user.balance, 0);
    }
}

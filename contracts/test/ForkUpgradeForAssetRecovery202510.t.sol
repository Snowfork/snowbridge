// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {Vm} from "forge-std/Vm.sol";
import {Test} from "forge-std/Test.sol";
import {stdJson} from "forge-std/StdJson.sol";
import {console} from "forge-std/console.sol";

import {IUpgradable} from "../src/interfaces/IUpgradable.sol";
import {IGatewayV1} from "../src/v1/IGateway.sol";
import {Verification} from "../src/Verification.sol";
import {Gateway} from "../src/Gateway.sol";
import {GatewayAssetRecovery202510} from "../src/upgrade/GatewayAssetRecovery202510.sol";
import {AgentExecutor} from "../src/AgentExecutor.sol";
import {UpgradeParams, SetOperatingModeParams, OperatingMode, RegisterForeignTokenParams} from "../src/v1/Types.sol";
import {ChannelID, ParaID, OperatingMode, InboundMessage, Command, TokenInfo} from "../src/v1/Types.sol";
import {MultiAddress, multiAddressFromBytes32} from "../src/v1/MultiAddress.sol";
import {ForkTestFixtures, SubmitMessageFixture} from "./utils/ForkTestFixtures.sol";

contract ForkUpgradeForAssetRecovery202510 is Test {
    address private constant GATEWAY_PROXY = 0x27ca963C279c93801941e1eB8799c23f407d68e7;
    address private constant BEEFY_CLIENT = 0x1817874feAb3ce053d0F40AbC23870DB35C2AFfc;

    // NOTE: Can use tenderly transaction debugger to retrieve existing library address
    address private constant VERIFICATION_ADDR_V1 = 0x90c7F378e9ceD5dD268f0dF987c0838469846Da1;
    bytes4 private constant VERIFICATION_SELECTOR_V1 = 0xbc9535d4;

    ChannelID constant internal GOVERNANCE_CHANNEL = ChannelID.wrap(0x0000000000000000000000000000000000000000000000000000000000000001);
    ChannelID constant internal ASSETHUB_CHANNEL = ChannelID.wrap(0xc173fac324158e77fb5840738a1a541f633cbec8884c6a601c567d2b376a0539);
    bytes32 constant internal ASSETHUB_AGENT_ID =
        0x81c5ab2571199e3188135178f3c2c8e2d268be1313d029b30f534fa579b69b79;
    address constant internal assetHubAgent = 0xd803472c47a87D7B63E888DE53f03B4191B846a8;
    address constant internal hardcodedRecipient = 0xAd8D4c544a6ce24B89841354b2738E026a12BcA4;
    uint256 balanceInAgentBefore;
    uint256 balanceInRecipientBefore;

    function setUp() public {
        vm.createSelectFork("https://virtual.mainnet.eu.rpc.tenderly.co/dc7e68d7-5fac-4a88-a797-01ed4155f437", 23432697);

        // Mock call to Verification.verifyCommitment to bypass BEEFY verification.
        vm.mockCall(VERIFICATION_ADDR_V1, abi.encodeWithSelector(VERIFICATION_SELECTOR_V1), abi.encode(true));
        
        // Deploy new implementation contract
        GatewayAssetRecovery202510 newLogic = new GatewayAssetRecovery202510(
            BEEFY_CLIENT,
            address(new AgentExecutor())
        );

        // Prepare upgrade command
        UpgradeParams memory params = UpgradeParams({
            impl: address(newLogic),
            implCodeHash: address(newLogic).codehash,
            initParams: bytes("")
        });

        (bytes32[] memory proof1, Verification.Proof memory proof2) = ForkTestFixtures.makeMockProofs();
        (uint64 nonce,) = IGatewayV1(GATEWAY_PROXY).channelNoncesOf(GOVERNANCE_CHANNEL);

        balanceInAgentBefore = assetHubAgent.balance;
        balanceInRecipientBefore = hardcodedRecipient.balance;

        vm.expectEmit();
        emit IUpgradable.Upgraded(address(newLogic));

        // Issue the upgrade
        IGatewayV1(GATEWAY_PROXY).submitV1(
            InboundMessage(
                GOVERNANCE_CHANNEL,
                nonce + 1,
                Command.Upgrade,
                abi.encode(params),
                100_000,
                block.basefee,
                0,
                keccak256("message-id")
            ),
            proof1,
            proof2
        );

        // Governance channel inbound nonce should have incremented by 1
        (uint64 nonceAfter,) = IGatewayV1(GATEWAY_PROXY).channelNoncesOf(GOVERNANCE_CHANNEL);
        assertEq(nonceAfter, nonce + 1);      
    }

    function testUpgradeForAssetRecovery202510Success() public {
       // Asset hub agent balance should have decreased by 0.35 ETH
        uint256 balanceInAgentAfter = assetHubAgent.balance;
        assertEq(balanceInAgentAfter, balanceInAgentBefore - 350000000000000000);

        // Hardcoded recipient balance should have increased by 0.35 ETH
        uint256 balanceInRecipientAfter = hardcodedRecipient.balance;
        assertEq(balanceInRecipientAfter, balanceInRecipientBefore + 350000000000000000);
    }
}

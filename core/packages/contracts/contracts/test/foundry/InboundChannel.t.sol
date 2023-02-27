// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "forge-std/Test.sol";
import "forge-std/console.sol";

import { InboundChannel } from "../../InboundChannel.sol";
import { EtherVault  } from "../../EtherVault.sol";
import { IParachainClient } from "../../IParachainClient.sol";
import { SovereignTreasury  } from "../../SovereignTreasury.sol";
import { ParachainClientMock } from "../ParachainClientMock.sol";
import { RecipientMock } from "../RecipientMock.sol";

contract InboundChannelTest is Test {

    InboundChannel public channel;
    RecipientMock public recipient;

    bytes32[] proof = [bytes32(0x2f9ee6cfdf244060dc28aa46347c5219e303fc95062dd672b4e406ca5c29764b)];
    bool[] hashSides = [true];

    function setUp() public {
        IParachainClient parachainClient = new ParachainClientMock();
        recipient = new RecipientMock();

        // SovereignTreasury
        EtherVault etherVault = new EtherVault();
        SovereignTreasury treasury = new SovereignTreasury(etherVault);
        etherVault.transferOwnership(address(treasury));

        deal(address(this), 100 ether);
        treasury.deposit{value: 50 ether}(bytes("statemint"));

        channel = new InboundChannel(parachainClient, treasury, 100 wei);

        treasury.grantRole(treasury.WITHDRAW_ROLE(), address(channel));
    }

    function testSubmit() public {
        address relayer = makeAddr("alice");
        hoax(relayer, 100 ether);

        channel.submit(
            InboundChannel.Message(
                bytes("statemint"),
                1,
                address(recipient),
                hex"deadbeef"
            ),
            proof,
            hashSides,
            hex"deadbeef"
        );
    }

}

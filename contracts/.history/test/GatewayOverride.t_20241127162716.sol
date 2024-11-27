// SPDX-License-Identifier: SEE LICENSE IN LICENSE
pragma solidity 0.8.25;

import {Test} from "forge-std/Test.sol";

import {IGateway} from "../src/interfaces/IGateway.sol";
import {MockGateway} from "./mocks/MockGateway.sol";

import {Channel, InboundMessage, OperatingMode, ParaID, Command, ChannelID} from "../src/Types.sol";
import {CreateChannelParams, CreateAgentParams} from "../src/Params.sol";
import {Validators} from "../src/Validators.sol";

import {GatewayTest} from "./Gateway.t.sol";

contract GatewayOverride is GatewayTest {
    //     Whole payload: "7015003800000cd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d90b5ab205c6974c9ea841be688864633dc9ca8a357843eeacf2314649965fe228eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48"
    // Breaking down payload into components below:
    // Magic bytes: "70150038"
    // Message: "00000cd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d90b5ab205c6974c9ea841be688864633dc9ca8a357843eeacf2314649965fe228eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48"
    // Breaking down message below
    // Validators: "0cd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d90b5ab205c6974c9ea841be688864633dc9ca8a357843eeacf2314649965fe228eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48"
    // Breaking down validators array below:
    // Size of validator vector compact encoded: "0c"
    // Array without the scale encoded size in front: "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d90b5ab205c6974c9ea841be688864633dc9ca8a357843eeacf2314649965fe228eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48"

    bytes private constant FINAL_VALIDATORS_PAYLOAD =
        hex"7015003800000cd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d90b5ab205c6974c9ea841be688864633dc9ca8a357843eeacf2314649965fe228eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48";
    bytes private constant VALIDATORS_DATA =
        "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d90b5ab205c6974c9ea841be688864633dc9ca8a357843eeacf2314649965fe228eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48";

    bytes private constant WRONG_LENGTH_VALIDATORS_DATA =
        "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d90b5ab205c6974c9ea841be688864633dc9ca8a357843eeacf2314649965fe228eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a4";

    function testSendValidatorsData() public {
        // Create mock agent and paraID
        ParaID paraID = ParaID.wrap(3);
        bytes32 agentID = keccak256("3");

        MockGateway(address(gateway)).createAgentPublic(abi.encode(CreateAgentParams({agentID: agentID})));

        CreateChannelParams memory params =
            CreateChannelParams({channelID: paraID.into(), agentID: agentID, mode: OperatingMode.Normal});

        vm.expectEmit(true, false, false, true);
        emit IGateway.ChannelCreated(paraID.into());
        MockGateway(address(gateway)).createChannelPublic(abi.encode(params));

        vm.expectEmit(true, true, false, false);
        emit IGateway.ValidatorsDataCreated(3, FINAL_VALIDATORS_PAYLOAD);

        vm.expectEmit(true, false, false, false);
        emit IGateway.OutboundMessageAccepted(paraID.into(), 1, messageID, FINAL_VALIDATORS_PAYLOAD);

        IGateway(address(gateway)).sendValidatorsData{value: 1 ether}(VALIDATORS_DATA, paraID);
    }

    function testShouldNotSendValidatorsDataBecauseValidatorsNotMultipleOf32() public {
        // Create mock agent and paraID
        ParaID paraID = ParaID.wrap(3);
        bytes32 agentID = keccak256("3");

        MockGateway(address(gateway)).createAgentPublic(abi.encode(CreateAgentParams({agentID: agentID})));

        CreateChannelParams memory params =
            CreateChannelParams({channelID: paraID.into(), agentID: agentID, mode: OperatingMode.Normal});

        vm.expectEmit(true, false, false, true);
        emit IGateway.ChannelCreated(paraID.into());
        MockGateway(address(gateway)).createChannelPublic(abi.encode(params));

        // error Validators__UnsupportedValidatorsLength();
        //     error Validators__ValidatorsLengthTooLong();
        vm.expectRevert(Validators.Validators__UnsupportedValidatorsLength.selector);

        IGateway(address(gateway)).sendValidatorsData{value: 1 ether}(VALIDATORS_DATA, paraID);
    }
}

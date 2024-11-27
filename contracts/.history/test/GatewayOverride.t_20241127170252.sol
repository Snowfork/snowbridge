// SPDX-License-Identifier: SEE LICENSE IN LICENSE
pragma solidity 0.8.25;

import {console2, Test} from "forge-std/Test.sol";

import {IGateway} from "../src/interfaces/IGateway.sol";
import {MockGateway} from "./mocks/MockGateway.sol";

import {Channel, InboundMessage, OperatingMode, ParaID, Command, ChannelID} from "../src/Types.sol";
import {CreateChannelParams, CreateAgentParams} from "../src/Params.sol";
import {Validators} from "../src/Validators.sol";

import {GatewayTest} from "./Gateway.t.sol";

contract GatewayOverride is GatewayTest {}

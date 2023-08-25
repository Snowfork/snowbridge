// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity ^0.8.20;

import {Gateway} from "../../src/Gateway.sol";
import {GatewayMock} from "./GatewayMock.sol";
import {OperatingMode} from "../../src/Types.sol";
import {console} from "forge-std/console.sol";

contract HelloWorld {
    event SaidHello(string indexed message);

    function sayHello(string memory _text) public {
        string memory fullMessage = string(abi.encodePacked("Hello there, ", _text));
        emit SaidHello(fullMessage);
    }

    function attack(address gateway) public {
        console.log("gateway proxy address is: %s", gateway);
        Gateway.SetOperatingModeParams memory params =
            Gateway.SetOperatingModeParams({mode: OperatingMode.RejectingOutboundMessages});

        bytes memory call = abi.encodeCall(GatewayMock.setOperatingModePublic, abi.encode(params));

        (bool success,) = gateway.call(call);

        require(success, "Failed to reentrancy");

        emit SaidHello("hacked");
    }
}

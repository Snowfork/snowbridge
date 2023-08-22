// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity ^0.8.0;

contract HelloWorld {
    event SaidHello(string indexed message);

    function sayHello(string memory _text) public {
        string memory fullMessage = string(abi.encodePacked("Hello there, ", _text));
        emit SaidHello(fullMessage);
    }
}

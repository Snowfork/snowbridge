// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.23;

contract HelloWorld {
    event SaidHello(string indexed message);

    function sayHello(string memory _text) public {
        string memory fullMessage = string(abi.encodePacked("Hello there, ", _text));
        emit SaidHello(fullMessage);
    }
}

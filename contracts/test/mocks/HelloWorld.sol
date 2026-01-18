// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {IERC20} from "../../src/interfaces/IERC20.sol";

contract HelloWorld {
    event SaidHello(string indexed message);
    event TokenConsumed(address indexed token, address indexed from, uint256 amount);

    error Unauthorized();

    function sayHello(string memory _text) public payable {
        string memory fullMessage = string(abi.encodePacked("Hello there, ", _text));
        emit SaidHello(fullMessage);
    }

    function revertUnauthorized() public pure {
        revert Unauthorized();
    }

    function retBomb() public pure returns (bytes memory) {
        assembly {
            return(1, 3000000)
        }
    }

    /// @dev Consume an approved ERC20 token from the caller
    /// @param token The ERC20 token address
    /// @param amount The amount to transfer from msg.sender to this contract
    function consumeToken(address token, uint256 amount) public {
        require(
            IERC20(token).transferFrom(msg.sender, address(this), amount), "transferFrom failed"
        );
        emit TokenConsumed(token, msg.sender, amount);
    }
}

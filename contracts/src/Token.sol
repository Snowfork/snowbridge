// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2025 Snowfork <hello@snowfork.com>

pragma solidity 0.8.28;

import {IERC20} from "./interfaces/IERC20.sol";
import {IERC20Metadata} from "./interfaces/IERC20Metadata.sol";
import {IERC20Permit} from "./interfaces/IERC20Permit.sol";
import {TokenLib} from "./TokenLib.sol";

/**
 * @dev Implementation of the {IERC20} interface.
 */
contract Token is IERC20, IERC20Metadata, IERC20Permit {
    using TokenLib for TokenLib.Token;

    address public immutable gateway;
    uint8 public immutable decimals;

    string public name;
    string public symbol;

    TokenLib.Token internal token;

    error Unauthorized();

    /**
     * @dev Sets the values for {name}, {symbol}, and {decimals}.
     */
    constructor(string memory _name, string memory _symbol, uint8 _decimals) {
        name = _name;
        symbol = _symbol;
        decimals = _decimals;
        gateway = msg.sender;
    }

    modifier onlyGateway() {
        if (msg.sender != gateway) {
            revert Unauthorized();
        }
        _;
    }

    function mint(address account, uint256 amount) external onlyGateway {
        token.mint(account, amount);
    }

    function burn(address account, uint256 amount) external onlyGateway {
        token.burn(account, amount);
    }

    function transfer(address recipient, uint256 amount) external returns (bool) {
        return token.transfer(recipient, amount);
    }

    function approve(address spender, uint256 amount) external returns (bool) {
        return token.approve(spender, amount);
    }

    function transferFrom(address sender, address recipient, uint256 amount) external returns (bool) {
        return token.transferFrom(sender, recipient, amount);
    }

    function balanceOf(address account) external view returns (uint256) {
        return token.balance[account];
    }

    function totalSupply() external view returns (uint256) {
        return token.totalSupply;
    }

    function allowance(address _owner, address spender) external view returns (uint256) {
        return token.allowance[_owner][spender];
    }

    // IERC20Permit

    function DOMAIN_SEPARATOR() external view returns (bytes32) {
        return TokenLib.domainSeparator(name);
    }

    function permit(address issuer, address spender, uint256 value, uint256 deadline, uint8 v, bytes32 r, bytes32 s)
        external
    {
        token.permit(name, issuer, spender, value, deadline, v, r, s);
    }

    function nonces(address account) external view returns (uint256) {
        return token.nonces[account];
    }
}

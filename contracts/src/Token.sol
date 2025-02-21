// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: 2023 Axelar Network
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>

pragma solidity 0.8.28;

import {IERC20} from "./interfaces/IERC20.sol";
import {IERC20Permit} from "./interfaces/IERC20Permit.sol";
import {TokenLib} from "./TokenLib.sol";

/**
 * @dev Implementation of the {IERC20} interface.
 *
 * This implementation is agnostic to the way tokens are created. This means
 * that a supply mechanism has to be added in a derived contract using {_mint}.
 * This supply mechanism has been added in {ERC20Permit-mint}.
 *
 * We have followed general OpenZeppelin guidelines: functions revert instead
 * of returning `false` on failure. This behavior is conventional and does
 * not conflict with the expectations of ERC20 applications.
 *
 * Additionally, an {Approval} event is emitted on calls to {transferFrom}.
 * This allows applications to reconstruct the allowance for all accounts just
 * by listening to these events. Other implementations of the EIP may not emit
 * these events, as it isn't required by the specification.
 *
 * Finally, the non-standard {decreaseAllowance} and {increaseAllowance}
 * functions have been added to mitigate the well-known issues around setting
 * allowances. See {IERC20-approve}.
 */
contract Token is IERC20, IERC20Permit {
    using TokenLib for TokenLib.Token;

    bytes32 public immutable DOMAIN_SEPARATOR;
    uint8 public immutable decimals;

    address internal owner;
    string public name;
    string public symbol;
    TokenLib.Token token;

    error Unauthorized();

    /**
     * @dev Sets the values for {name}, {symbol}, and {decimals}.
     */
    constructor(string memory _name, string memory _symbol, uint8 _decimals) {
        name = _name;
        symbol = _symbol;
        decimals = _decimals;
        owner = msg.sender;
        DOMAIN_SEPARATOR = keccak256(
            abi.encode(
                keccak256("EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)"),
                keccak256(bytes(_name)),
                keccak256(bytes("1")),
                block.chainid,
                address(this)
            )
        );
    }

    modifier onlyOwner() {
        if (msg.sender != owner) {
            revert Unauthorized();
        }
        _;
    }

    function setOwner(address newOwner) external onlyOwner {
        owner = newOwner;
    }

    function mint(address account, uint256 amount) external onlyOwner {
        token.mint(account, amount);
    }

    function burn(address account, uint256 amount) external onlyOwner {
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

    function permit(address issuer, address spender, uint256 value, uint256 deadline, uint8 v, bytes32 r, bytes32 s)
        external
    {
        token.permit(DOMAIN_SEPARATOR, issuer, spender, value, deadline, v, r, s);
    }

    function balanceOf(address account) external view returns (uint256) {
        return token.balance[account];
    }

    function nonces(address account) external view returns (uint256) {
        return token.nonces[account];
    }

    function totalSupply() external view returns (uint256) {
        return token.totalSupply;
    }

    function allowance(address owner, address spender) external view returns (uint256) {
        return token.allowance[owner][spender];
    }
}

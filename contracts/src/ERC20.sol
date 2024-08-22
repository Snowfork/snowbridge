// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: 2023 Axelar Network
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>

pragma solidity 0.8.25;

import {IERC20} from "./interfaces/IERC20.sol";
import {IERC20Permit} from "./interfaces/IERC20Permit.sol";
import {ERC20Lib} from "./ERC20Lib.sol";

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
contract ERC20 is IERC20, IERC20Permit {
    using ERC20Lib for ERC20Lib.TokenStorage;

    error Unauthorized();

    ERC20Lib.TokenStorage token;

    address public immutable OWNER;

    uint8 public immutable decimals;

    string public name;
    string public symbol;

    /**
     * @dev Sets the values for {name}, {symbol}, and {decimals}.
     */
    constructor(address _owner, string memory name_, string memory symbol_, uint8 decimals_) {
        OWNER = _owner;
        name = name_;
        symbol = symbol_;
        decimals = decimals_;
        token.init(name_);
    }

    modifier onlyOwner() {
        if (msg.sender != OWNER) {
            revert Unauthorized();
        }
        _;
    }

    /**
     * @dev Creates `amount` tokens and assigns them to `account`, increasing
     * the total supply. Can only be called by the owner.
     *
     * Emits a {Transfer} event with `from` set to the zero address.
     *
     * Requirements:
     *
     * - `account` cannot be the zero address.
     */
    function mint(address account, uint256 amount) external virtual onlyOwner {
        token.mint(account, amount);
    }

    /**
     * @dev Destroys `amount` tokens from the account.
     */
    function burn(address account, uint256 amount) external virtual onlyOwner {
        token.burn(account, amount);
    }

    /**
     * @dev See {IERC20-transfer}.
     *
     * Requirements:
     *
     * - `recipient` cannot be the zero address.
     * - the caller must have a balance of at least `amount`.
     */
    function transfer(address recipient, uint256 amount) external virtual override returns (bool) {
        return token.transfer(msg.sender, recipient, amount);
    }

    /**
     * @dev See {IERC20-approve}.
     *
     * NOTE: Prefer the {increaseAllowance} and {decreaseAllowance} methods, as
     * they aren't vulnerable to the frontrunning attack described here:
     * https://github.com/ethereum/EIPs/issues/20#issuecomment-263524729
     * See {IERC20-approve}.
     *
     * NOTE: If `amount` is the maximum `uint256`, the allowance is not updated on
     * `transferFrom`. This is semantically equivalent to an infinite approval.
     *
     * Requirements:
     *
     * - `spender` cannot be the zero address.
     */
    function approve(address spender, uint256 amount) external virtual override returns (bool) {
        return token.approve(msg.sender, spender, amount);
    }

    /**
     * @dev See {IERC20-transferFrom}.
     *
     * Emits an {Approval} event indicating the updated allowance. This is not
     * required by the EIP. See the note at the beginning of {ERC20}.
     *
     * Requirements:
     *
     * - `sender` and `recipient` cannot be the zero address.
     * - `sender` must have a balance of at least `amount`.
     * - the caller must have allowance for ``sender``'s tokens of at least
     * `amount`.
     */
    function transferFrom(address sender, address recipient, uint256 amount) external virtual override returns (bool) {
        return token.transferFrom(sender, recipient, amount);
    }

    /**
     * @dev Atomically increases the allowance granted to `spender` by the caller.
     *
     * This is an alternative to {approve} that can be used as a mitigation for
     * problems described in {IERC20-approve}.
     *
     * Emits an {Approval} event indicating the updated allowance.
     *
     * Requirements:
     *
     * - `spender` cannot be the zero address.
     */
    function increaseAllowance(address spender, uint256 addedValue) external virtual returns (bool) {
        return token.increaseAllowance(spender, addedValue);
    }

    /**
     * @dev Atomically decreases the allowance granted to `spender` by the caller.
     *
     * This is an alternative to {approve} that can be used as a mitigation for
     * problems described in {IERC20-approve}.
     *
     * Emits an {Approval} event indicating the updated allowance.
     *
     * Requirements:
     *
     * - `spender` cannot be the zero address.
     * - `spender` must have allowance for the caller of at least
     * `subtractedValue`.
     */
    function decreaseAllowance(address spender, uint256 subtractedValue) external virtual returns (bool) {
        return token.decreaseAllowance(spender, subtractedValue);
    }

    function permit(address issuer, address spender, uint256 value, uint256 deadline, uint8 v, bytes32 r, bytes32 s)
        external
    {
        token.permit(issuer, spender, value, deadline, v, r, s);
    }

    function balanceOf(address account) external view returns (uint256) {
        return token.balancesOf(account);
    }

    function nonces(address account) external view returns (uint256) {
        return token.noncesOf(account);
    }

    function totalSupply() external view returns (uint256) {
        return token.totalSupplyOf();
    }

    function allowance(address owner, address spender) external view returns (uint256) {
        return token.allowanceOf(owner, spender);
    }

    function DOMAIN_SEPARATOR() external view returns (bytes32) {
        return token.domainSeparatorOf();
    }
}

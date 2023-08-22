// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: 2023 Axelar Network
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>

pragma solidity 0.8.20;

import {IERC20} from "./interfaces/IERC20.sol";
import {IERC20Permit} from "./interfaces/IERC20Permit.sol";

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
    error PermitExpired();
    error InvalidS();
    error InvalidV();
    error InvalidSignature();
    error Unauthorized();

    mapping(address => uint256) public override balanceOf;

    mapping(address => mapping(address => uint256)) public override allowance;

    mapping(address => uint256) public nonces;

    bytes32 public immutable DOMAIN_SEPARATOR;
    uint256 public override totalSupply;

    // keccak256('EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)')
    bytes32 private constant DOMAIN_TYPE_SIGNATURE_HASH =
        bytes32(0x8b73c3c69bb8fe3d512ecc4cf759cc79239f7b179b0ffacaa9a75d522b39400f);

    // keccak256('Permit(address owner,address spender,uint256 value,uint256 nonce,uint256 deadline)')
    bytes32 private constant PERMIT_SIGNATURE_HASH =
        bytes32(0x6e71edae12b1b97f4d1f60370fef10105fa2faae0126114a169c64845d6126c9);

    address public immutable OWNER;
    string private constant EIP191_PREFIX_FOR_EIP712_STRUCTURED_DATA = "\x19\x01";
    uint8 public immutable decimals;

    string public name;
    string public symbol;

    /**
     * @dev Sets the values for {name}, {symbol}, and {decimals}.
     */
    constructor(string memory name_, string memory symbol_, uint8 decimals_) {
        OWNER = msg.sender;
        DOMAIN_SEPARATOR = keccak256(
            abi.encode(
                DOMAIN_TYPE_SIGNATURE_HASH, keccak256(bytes(name_)), keccak256(bytes("1")), block.chainid, address(this)
            )
        );

        name = name_;
        symbol = symbol_;
        decimals = decimals_;
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
     * - `to` cannot be the zero address.
     */
    function mint(address account, uint256 amount) external virtual onlyOwner {
        _mint(account, amount);
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
        _transfer(msg.sender, recipient, amount);
        return true;
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
        _approve(msg.sender, spender, amount);
        return true;
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
        uint256 _allowance = allowance[sender][msg.sender];

        if (_allowance != type(uint256).max) {
            _approve(sender, msg.sender, _allowance - amount);
        }

        _transfer(sender, recipient, amount);

        return true;
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
        _approve(msg.sender, spender, allowance[msg.sender][spender] + addedValue);
        return true;
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
        _approve(msg.sender, spender, allowance[msg.sender][spender] - subtractedValue);
        return true;
    }

    function permit(address issuer, address spender, uint256 value, uint256 deadline, uint8 v, bytes32 r, bytes32 s)
        external
    {
        if (block.timestamp > deadline) revert PermitExpired();

        if (uint256(s) > 0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF5D576E7357A4501DDFE92F46681B20A0) revert InvalidS();

        if (v != 27 && v != 28) revert InvalidV();

        bytes32 digest = keccak256(
            abi.encodePacked(
                EIP191_PREFIX_FOR_EIP712_STRUCTURED_DATA,
                DOMAIN_SEPARATOR,
                keccak256(abi.encode(PERMIT_SIGNATURE_HASH, issuer, spender, value, nonces[issuer]++, deadline))
            )
        );

        address recoveredAddress = ecrecover(digest, v, r, s);

        if (recoveredAddress != issuer) revert InvalidSignature();

        // _approve will revert if issuer is address(0x0)
        _approve(issuer, spender, value);
    }

    /**
     * @dev Moves tokens `amount` from `sender` to `recipient`.
     *
     * This is internal function is equivalent to {transfer}, and can be used to
     * e.g. implement automatic token fees, slashing mechanisms, etc.
     *
     * Emits a {Transfer} event.
     *
     * Requirements:
     *
     * - `sender` cannot be the zero address.
     * - `recipient` cannot be the zero address.
     * - `sender` must have a balance of at least `amount`.
     */
    function _transfer(address sender, address recipient, uint256 amount) internal virtual {
        if (sender == address(0) || recipient == address(0)) revert InvalidAccount();

        balanceOf[sender] -= amount;
        balanceOf[recipient] += amount;
        emit Transfer(sender, recipient, amount);
    }

    /**
     * @dev Creates `amount` tokens and assigns them to `account`, increasing
     * the total supply.
     *
     * Emits a {Transfer} event with `from` set to the zero address.
     *
     * Requirements:
     *
     * - `to` cannot be the zero address.
     */
    function _mint(address account, uint256 amount) internal virtual {
        if (account == address(0)) revert InvalidAccount();

        totalSupply += amount;
        balanceOf[account] += amount;
        emit Transfer(address(0), account, amount);
    }

    /**
     * @dev Destroys `amount` tokens from `account`, reducing the
     * total supply.
     *
     * Emits a {Transfer} event with `to` set to the zero address.
     *
     * Requirements:
     *
     * - `account` cannot be the zero address.
     * - `account` must have at least `amount` tokens.
     */
    function _burn(address account, uint256 amount) internal virtual {
        if (account == address(0)) revert InvalidAccount();

        balanceOf[account] -= amount;
        totalSupply -= amount;
        emit Transfer(account, address(0), amount);
    }

    /**
     * @dev Sets `amount` as the allowance of `spender` over the `owner` s tokens.
     *
     * This internal function is equivalent to `approve`, and can be used to
     * e.g. set automatic allowances for certain subsystems, etc.
     *
     * Emits an {Approval} event.
     *
     * Requirements:
     *
     * - `owner` cannot be the zero address.
     * - `spender` cannot be the zero address.
     */
    function _approve(address owner, address spender, uint256 amount) internal virtual {
        if (owner == address(0) || spender == address(0)) revert InvalidAccount();

        allowance[owner][spender] = amount;
        emit Approval(owner, spender, amount);
    }
}

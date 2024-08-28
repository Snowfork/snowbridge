// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: 2023 Axelar Network
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>

pragma solidity 0.8.25;

import {IERC20} from "./interfaces/IERC20.sol";
import {IERC20Permit} from "./interfaces/IERC20Permit.sol";

library TokenLib {
    // keccak256('EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)')
    bytes32 internal constant DOMAIN_TYPE_SIGNATURE_HASH =
        bytes32(0x8b73c3c69bb8fe3d512ecc4cf759cc79239f7b179b0ffacaa9a75d522b39400f);

    // keccak256('Permit(address owner,address spender,uint256 value,uint256 nonce,uint256 deadline)')
    bytes32 internal constant PERMIT_SIGNATURE_HASH =
        bytes32(0x6e71edae12b1b97f4d1f60370fef10105fa2faae0126114a169c64845d6126c9);

    string internal constant EIP191_PREFIX_FOR_EIP712_STRUCTURED_DATA = "\x19\x01";

    struct Token {
        mapping(address account => uint256) balance;
        mapping(address account => mapping(address spender => uint256)) allowance;
        mapping(address token => uint256) nonces;
        uint256 totalSupply;
    }

    /**
     * @dev See {IERC20-transfer}.
     *
     * Requirements:
     *
     * - `recipient` cannot be the zero address.
     * - the caller must have a balance of at least `amount`.
     */
    function transfer(Token storage token, address sender, address recipient, uint256 amount) external returns (bool) {
        _transfer(token, sender, recipient, amount);
        return true;
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
    function mint(Token storage token, address account, uint256 amount) external {
        if (account == address(0)) {
            revert IERC20.InvalidAccount();
        }

        _update(token, address(0), account, amount);
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
    function burn(Token storage token, address account, uint256 amount) external {
        if (account == address(0)) {
            revert IERC20.InvalidAccount();
        }

        _update(token, account, address(0), amount);
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
    function approve(Token storage token, address owner, address spender, uint256 amount) external returns (bool) {
        _approve(token, owner, spender, amount);
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
    function transferFrom(Token storage token, address sender, address recipient, uint256 amount)
        external
        returns (bool)
    {
        uint256 _allowance = token.allowance[sender][msg.sender];

        if (_allowance != type(uint256).max) {
            if (_allowance < amount) {
                revert IERC20.InsufficientAllowance(msg.sender, _allowance, amount);
            }
            unchecked {
                _approve(token, sender, msg.sender, _allowance - amount);
            }
        }

        _transfer(token, sender, recipient, amount);

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
    function increaseAllowance(Token storage token, address spender, uint256 addedValue) external returns (bool) {
        uint256 _allowance = token.allowance[msg.sender][spender];
        if (_allowance != type(uint256).max) {
            _approve(token, msg.sender, spender, _allowance + addedValue);
        }
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
    function decreaseAllowance(Token storage token, address spender, uint256 subtractedValue) external returns (bool) {
        uint256 _allowance = token.allowance[msg.sender][spender];
        if (_allowance != type(uint256).max) {
            if (_allowance < subtractedValue) {
                revert IERC20.InsufficientAllowance(msg.sender, _allowance, subtractedValue);
            }
            unchecked {
                _approve(token, msg.sender, spender, _allowance - subtractedValue);
            }
        }
        return true;
    }

    function permit(
        Token storage token,
        bytes32 domainSeparator,
        address issuer,
        address spender,
        uint256 value,
        uint256 deadline,
        uint8 v,
        bytes32 r,
        bytes32 s
    ) external {
        if (block.timestamp > deadline) revert IERC20Permit.PermitExpired();

        if (uint256(s) > 0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF5D576E7357A4501DDFE92F46681B20A0) {
            revert IERC20Permit.InvalidS();
        }

        if (v != 27 && v != 28) revert IERC20Permit.InvalidV();

        bytes32 digest = keccak256(
            abi.encodePacked(
                EIP191_PREFIX_FOR_EIP712_STRUCTURED_DATA,
                domainSeparator,
                keccak256(abi.encode(PERMIT_SIGNATURE_HASH, issuer, spender, value, token.nonces[issuer]++, deadline))
            )
        );

        address recoveredAddress = ecrecover(digest, v, r, s);

        if (recoveredAddress != issuer) revert IERC20Permit.InvalidSignature();

        // _approve will revert if issuer is address(0x0)
        _approve(token, issuer, spender, value);
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
    function _transfer(Token storage token, address sender, address recipient, uint256 amount) internal {
        if (sender == address(0) || recipient == address(0)) {
            revert IERC20.InvalidAccount();
        }

        _update(token, sender, recipient, amount);
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
    function _approve(Token storage token, address owner, address spender, uint256 amount) internal {
        if (owner == address(0) || spender == address(0)) {
            revert IERC20.InvalidAccount();
        }

        token.allowance[owner][spender] = amount;
        emit IERC20.Approval(owner, spender, amount);
    }

    /**
     * @dev Transfers a `value` amount of tokens from `from` to `to`, or alternatively mints (or burns) if `from`
     * (or `to`) is the zero address. All customizations to transfers, mints, and burns should be done by overriding
     * this function.
     *
     * Emits a {Transfer} event.
     */
    function _update(Token storage token, address from, address to, uint256 value) internal {
        if (from == address(0)) {
            // Overflow check required: The rest of the code assumes that totalSupply never overflows
            token.totalSupply += value;
        } else {
            uint256 fromBalance = token.balance[from];
            if (fromBalance < value) {
                revert IERC20.InsufficientBalance(from, fromBalance, value);
            }
            unchecked {
                // Overflow not possible: value <= fromBalance <= totalSupply.
                token.balance[from] = fromBalance - value;
            }
        }

        if (to == address(0)) {
            unchecked {
                // Overflow not possible: value <= totalSupply or value <= fromBalance <= totalSupply.
                token.totalSupply -= value;
            }
        } else {
            unchecked {
                // Overflow not possible: balance + value is at most totalSupply, which we know fits into a uint256.
                token.balance[to] += value;
            }
        }

        emit IERC20.Transfer(from, to, value);
    }
}

// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: 2023 Axelar Network
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>

pragma solidity 0.8.23;

import {IERC20} from "./interfaces/IERC20.sol";
import {IERC20Permit} from "./interfaces/IERC20Permit.sol";

library ERC20Lib {
    // keccak256('EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)')
    bytes32 internal constant DOMAIN_TYPE_SIGNATURE_HASH =
        bytes32(0x8b73c3c69bb8fe3d512ecc4cf759cc79239f7b179b0ffacaa9a75d522b39400f);

    // keccak256('Permit(address owner,address spender,uint256 value,uint256 nonce,uint256 deadline)')
    bytes32 internal constant PERMIT_SIGNATURE_HASH =
        bytes32(0x6e71edae12b1b97f4d1f60370fef10105fa2faae0126114a169c64845d6126c9);

    string internal constant EIP191_PREFIX_FOR_EIP712_STRUCTURED_DATA = "\x19\x01";

    error InvalidAccount();
    error PermitExpired();
    error InvalidS();
    error InvalidV();
    error InvalidSignature();
    error ERC20InsufficientBalance(address sender, uint256 balance, uint256 needed);
    error ERC20InsufficientAllowance(address spender, uint256 allowance, uint256 needed);
    error OwnableInvalidOwner(address owner);

    struct TokenStorage {
        mapping(address => uint256) balanceOf;
        mapping(address => mapping(address => uint256)) allowance;
        mapping(address => uint256) nonces;
        uint256 totalSupply;
        bytes32 domainSeparator;
    }

    function init(TokenStorage storage self, string memory name_) internal {
        self.domainSeparator = keccak256(
            abi.encode(
                DOMAIN_TYPE_SIGNATURE_HASH, keccak256(bytes(name_)), keccak256(bytes("1")), block.chainid, address(this)
            )
        );
    }

    /**
     * @dev See {IERC20-transfer}.
     *
     * Requirements:
     *
     * - `recipient` cannot be the zero address.
     * - the caller must have a balance of at least `amount`.
     */
    function transfer(TokenStorage storage self, address sender, address recipient, uint256 amount)
        external
        returns (bool)
    {
        _transfer(self, sender, recipient, amount);
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
    function mint(TokenStorage storage self, address account, uint256 amount) external {
        if (account == address(0)) revert InvalidAccount();

        _update(self, address(0), account, amount);
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
    function burn(TokenStorage storage self, address account, uint256 amount) external {
        if (account == address(0)) revert InvalidAccount();

        _update(self, account, address(0), amount);
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
    function approve(TokenStorage storage self, address owner, address spender, uint256 amount)
        external
        returns (bool)
    {
        _approve(self, owner, spender, amount);
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
    function transferFrom(TokenStorage storage self, address sender, address recipient, uint256 amount)
        external
        returns (bool)
    {
        uint256 _allowance = self.allowance[sender][msg.sender];

        if (_allowance != type(uint256).max) {
            if (_allowance < amount) {
                revert ERC20InsufficientAllowance(msg.sender, _allowance, amount);
            }
            unchecked {
                _approve(self, sender, msg.sender, _allowance - amount);
            }
        }

        _transfer(self, sender, recipient, amount);

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
    function increaseAllowance(TokenStorage storage self, address spender, uint256 addedValue)
        external
        returns (bool)
    {
        uint256 _allowance = self.allowance[msg.sender][spender];
        if (_allowance != type(uint256).max) {
            _approve(self, msg.sender, spender, _allowance + addedValue);
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
    function decreaseAllowance(TokenStorage storage self, address spender, uint256 subtractedValue)
        external
        returns (bool)
    {
        uint256 _allowance = self.allowance[msg.sender][spender];
        if (_allowance != type(uint256).max) {
            if (_allowance < subtractedValue) {
                revert ERC20InsufficientAllowance(msg.sender, _allowance, subtractedValue);
            }
            unchecked {
                _approve(self, msg.sender, spender, _allowance - subtractedValue);
            }
        }
        return true;
    }

    function permit(
        TokenStorage storage self,
        address issuer,
        address spender,
        uint256 value,
        uint256 deadline,
        uint8 v,
        bytes32 r,
        bytes32 s
    ) external {
        if (block.timestamp > deadline) revert PermitExpired();

        if (uint256(s) > 0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF5D576E7357A4501DDFE92F46681B20A0) revert InvalidS();

        if (v != 27 && v != 28) revert InvalidV();

        bytes32 digest = keccak256(
            abi.encodePacked(
                EIP191_PREFIX_FOR_EIP712_STRUCTURED_DATA,
                self.domainSeparator,
                keccak256(abi.encode(PERMIT_SIGNATURE_HASH, issuer, spender, value, self.nonces[issuer]++, deadline))
            )
        );

        address recoveredAddress = ecrecover(digest, v, r, s);

        if (recoveredAddress != issuer) revert InvalidSignature();

        // _approve will revert if issuer is address(0x0)
        _approve(self, issuer, spender, value);
    }

    function balancesOf(TokenStorage storage self, address account) internal view returns (uint256) {
        return self.balanceOf[account];
    }

    function noncesOf(TokenStorage storage self, address account) external view returns (uint256) {
        return self.nonces[account];
    }

    function totalSupplyOf(TokenStorage storage self) external view returns (uint256) {
        return self.totalSupply;
    }

    function allowanceOf(TokenStorage storage self, address owner, address spender) external view returns (uint256) {
        return self.allowance[owner][spender];
    }

    function domainSeparatorOf(TokenStorage storage self) external view returns (bytes32) {
        return self.domainSeparator;
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
    function _transfer(TokenStorage storage self, address sender, address recipient, uint256 amount) internal {
        if (sender == address(0) || recipient == address(0)) revert InvalidAccount();

        _update(self, sender, recipient, amount);
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
    function _approve(TokenStorage storage self, address owner, address spender, uint256 amount) internal {
        if (owner == address(0) || spender == address(0)) revert InvalidAccount();

        self.allowance[owner][spender] = amount;
        emit IERC20.Approval(owner, spender, amount);
    }

    /**
     * @dev Transfers a `value` amount of tokens from `from` to `to`, or alternatively mints (or burns) if `from`
     * (or `to`) is the zero address. All customizations to transfers, mints, and burns should be done by overriding
     * this function.
     *
     * Emits a {Transfer} event.
     */
    function _update(TokenStorage storage self, address from, address to, uint256 value) internal {
        if (from == address(0)) {
            // Overflow check required: The rest of the code assumes that totalSupply never overflows
            self.totalSupply += value;
        } else {
            uint256 fromBalance = self.balanceOf[from];
            if (fromBalance < value) {
                revert ERC20InsufficientBalance(from, fromBalance, value);
            }
            unchecked {
                // Overflow not possible: value <= fromBalance <= totalSupply.
                self.balanceOf[from] = fromBalance - value;
            }
        }

        if (to == address(0)) {
            unchecked {
                // Overflow not possible: value <= totalSupply or value <= fromBalance <= totalSupply.
                self.totalSupply -= value;
            }
        } else {
            unchecked {
                // Overflow not possible: balance + value is at most totalSupply, which we know fits into a uint256.
                self.balanceOf[to] += value;
            }
        }

        emit IERC20.Transfer(from, to, value);
    }
}

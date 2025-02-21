// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: 2023 Axelar Network
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>

pragma solidity 0.8.28;

import {IERC20} from "./interfaces/IERC20.sol";
import {IERC20Permit} from "./interfaces/IERC20Permit.sol";

library TokenLib {
    // keccak256('EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)')
    bytes32 internal constant DOMAIN_TYPE_SIGNATURE_HASH =
        bytes32(0x8b73c3c69bb8fe3d512ecc4cf759cc79239f7b179b0ffacaa9a75d522b39400f);

    // keccak256('Permit(address owner,address spender,uint256 value,uint256 nonce,uint256 deadline)')
    bytes32 internal constant PERMIT_SIGNATURE_HASH =
        bytes32(0x6e71edae12b1b97f4d1f60370fef10105fa2faae0126114a169c64845d6126c9);

    struct Token {
        mapping(address account => uint256) balance;
        mapping(address account => mapping(address spender => uint256)) allowance;
        mapping(address token => uint256) nonces;
        uint256 totalSupply;
    }

    function mint(Token storage token, address account, uint256 amount) external {
        require(account != address(0), IERC20.InvalidReceiver(account));
        _update(token, address(0), account, amount);
    }

    function burn(Token storage token, address account, uint256 amount) external {
        require(account != address(0), IERC20.InvalidSender(address(0)));
        _update(token, account, address(0), amount);
    }

    function approve(Token storage token, address spender, uint256 amount) external returns (bool) {
        _approve(token, msg.sender, spender, amount, true);
        return true;
    }

    function transfer(Token storage token, address recipient, uint256 amount) external returns (bool) {
        _transfer(token, msg.sender, recipient, amount);
        return true;
    }

    function transferFrom(Token storage token, address owner, address recipient, uint256 amount)
        external
        returns (bool)
    {
        _spendAllowance(token, owner, msg.sender, amount);
        _transfer(token, owner, recipient, amount);
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
        require(block.timestamp <= deadline, IERC20Permit.PermitExpired());
        require(
            uint256(s) <= 0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF5D576E7357A4501DDFE92F46681B20A0, IERC20Permit.InvalidS()
        );
        require(v == 27 || v == 28, IERC20Permit.InvalidV());

        bytes32 digest = keccak256(
            abi.encodePacked(
                "\x19\x01",
                domainSeparator,
                keccak256(
                    abi.encode(
                        keccak256("Permit(address owner,address spender,uint256 value,uint256 nonce,uint256 deadline)"),
                        issuer,
                        spender,
                        value,
                        token.nonces[issuer]++,
                        deadline
                    )
                )
            )
        );

        address signatory = ecrecover(digest, v, r, s);
        require(signatory != address(0), IERC20Permit.Unauthorized());
        require(signatory == issuer, IERC20Permit.InvalidSignature());

        _approve(token, issuer, spender, value, true);
    }

    function _transfer(Token storage token, address sender, address recipient, uint256 amount) internal {
        require(sender != address(0), IERC20.InvalidSender(address(0)));
        require(recipient != address(0), IERC20.InvalidReceiver(address(0)));
        _update(token, sender, recipient, amount);
    }

    function _spendAllowance(Token storage token, address owner, address spender, uint256 value)
        internal
        returns (bool)
    {
        uint256 allowance = token.allowance[owner][spender];
        if (allowance != type(uint256).max) {
            require(allowance >= value, IERC20.InsufficientAllowance(spender, allowance, value));
            unchecked {
                _approve(token, owner, spender, allowance - value, false);
            }
        }
        return true;
    }

    function _approve(Token storage token, address owner, address spender, uint256 amount, bool emitEvent) internal {
        require(owner != address(0), IERC20.InvalidApprover(address(0)));
        require(spender != address(0), IERC20.InvalidSpender(address(0)));

        token.allowance[owner][spender] = amount;

        if (emitEvent) {
            emit IERC20.Approval(owner, spender, amount);
        }
    }

    function _update(Token storage token, address from, address to, uint256 value) internal {
        if (from == address(0)) {
            // Overflow check required: The rest of the code assumes that totalSupply never overflows
            token.totalSupply += value;
        } else {
            uint256 fromBalance = token.balance[from];
            require(fromBalance >= value, IERC20.InsufficientBalance(from, fromBalance, value));
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

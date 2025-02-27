// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2025 Snowfork <hello@snowfork.com>

pragma solidity 0.8.28;

import {IERC20} from "./interfaces/IERC20.sol";
import {IERC20Permit} from "./interfaces/IERC20Permit.sol";
import {ECDSA} from "openzeppelin/utils/cryptography/ECDSA.sol";

library TokenLib {

    /// The EIP-712 typehash for the contract's domain
    bytes32 public constant DOMAIN_TYPEHASH = keccak256("EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)");

    /// The EIP-712 typehash for the permit struct used by the contract
    bytes32 public constant PERMIT_TYPEHASH = keccak256("Permit(address owner,address spender,uint256 value,uint256 nonce,uint256 deadline)");

    struct Token {
        mapping(address account => uint256) balance;
        mapping(address account => mapping(address spender => uint256)) allowance;
        mapping(address token => uint256) nonces;
        uint256 totalSupply;
    }

    function mint(Token storage token, address account, uint256 amount) external {
        require(account != address(0), IERC20.InvalidReceiver(address(0)));
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
        string storage tokenName,
        address issuer,
        address spender,
        uint256 value,
        uint256 deadline,
        uint8 v,
        bytes32 r,
        bytes32 s
    ) external {
        require(block.timestamp <= deadline, IERC20Permit.PermitExpired());

        bytes32 digest = keccak256(
            abi.encodePacked(
                hex"1901",
                _domainSeparator(tokenName),
                keccak256(
                    abi.encode(
                        PERMIT_TYPEHASH,
                        issuer,
                        spender,
                        value,
                        token.nonces[issuer]++,
                        deadline
                    )
                )
            )
        );

        address signatory = ECDSA.recover(digest, v, r, s);
        require(signatory == issuer, IERC20Permit.InvalidSignature());

        _approve(token, issuer, spender, value, true);
    }

    function domainSeparator(string storage name) external view returns (bytes32) {
        return _domainSeparator(name);
    }

    function _domainSeparator(string storage name) internal view returns (bytes32) {
        return keccak256(
            abi.encode(
                DOMAIN_TYPEHASH,
                keccak256(bytes(name)),
                keccak256(bytes("1")),
                block.chainid,
                address(this)
            )
        );
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
                // Overflow not possible: value <= fromBalance <= totalSupply
                token.balance[from] = fromBalance - value;
            }
        }

        if (to == address(0)) {
            unchecked {
                // Overflow not possible: value <= totalSupply or value <= fromBalance <= totalSupply
                token.totalSupply -= value;
            }
        } else {
            unchecked {
                // Overflow not possible: balance + value is at most totalSupply, which we know fits into a uint256
                token.balance[to] += value;
            }
        }

        emit IERC20.Transfer(from, to, value);
    }
}

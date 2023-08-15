// SPDX-License-Identifier: MIT

pragma solidity 0.8.20;

import {IERC20} from "./interfaces/IERC20.sol";
import {IERC20Permit} from "./interfaces/IERC20Permit.sol";

import {ERC20} from "./ERC20.sol";
import {Ownable} from "openzeppelin/access/Ownable.sol";

contract ERC20Permit is IERC20, IERC20Permit, ERC20, Ownable {
    error PermitExpired();
    error InvalidS();
    error InvalidV();
    error InvalidSignature();

    bytes32 public immutable DOMAIN_SEPARATOR;

    string private constant EIP191_PREFIX_FOR_EIP712_STRUCTURED_DATA = "\x19\x01";

    // keccak256('EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)')
    bytes32 private constant DOMAIN_TYPE_SIGNATURE_HASH =
        bytes32(0x8b73c3c69bb8fe3d512ecc4cf759cc79239f7b179b0ffacaa9a75d522b39400f);

    // keccak256('Permit(address owner,address spender,uint256 value,uint256 nonce,uint256 deadline)')
    bytes32 private constant PERMIT_SIGNATURE_HASH =
        bytes32(0x6e71edae12b1b97f4d1f60370fef10105fa2faae0126114a169c64845d6126c9);

    mapping(address => uint256) public nonces;

    constructor(string memory name) {
        DOMAIN_SEPARATOR = keccak256(
            abi.encode(
                DOMAIN_TYPE_SIGNATURE_HASH, keccak256(bytes(name)), keccak256(bytes("1")), block.chainid, address(this)
            )
        );
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
}

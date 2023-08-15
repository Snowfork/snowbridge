// SPDX-License-Identifier: MIT

pragma solidity ^0.8.9;

interface IERC20Permit {
    function DOMAIN_SEPARATOR() external view returns (bytes32);

    function nonces(address account) external view returns (uint256);

    function permit(address issuer, address spender, uint256 value, uint256 deadline, uint8 v, bytes32 r, bytes32 s)
        external;
}

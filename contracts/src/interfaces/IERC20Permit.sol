// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: 2023 Axelar Network
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>

pragma solidity 0.8.25;

interface IERC20Permit {
    error PermitExpired();
    error InvalidS();
    error InvalidV();
    error InvalidSignature();

    function DOMAIN_SEPARATOR() external view returns (bytes32);

    function nonces(address account) external view returns (uint256);

    function permit(address issuer, address spender, uint256 value, uint256 deadline, uint8 v, bytes32 r, bytes32 s)
        external;
}

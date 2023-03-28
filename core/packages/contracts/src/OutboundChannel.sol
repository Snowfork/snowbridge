// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import "openzeppelin/access/AccessControl.sol";
import "./IOutboundChannel.sol";
import "./IVault.sol";

contract OutboundChannel is IOutboundChannel, AccessControl {
    bytes32 public constant ADMIN_ROLE = keccak256("ADMIN_ROLE");
    bytes32 public constant SUBMIT_ROLE = keccak256("SUBMIT_ROLE");

    mapping(bytes => uint64) public nonce;

    IVault public vault;
    uint256 public fee;

    event Message(bytes indexed dest, uint64 nonce, bytes payload);

    event FeeUpdated(uint256 fee);

    error FeePaymentToLow();

    constructor(IVault _vault, uint256 _fee) {
        _grantRole(ADMIN_ROLE, msg.sender);
        _setRoleAdmin(SUBMIT_ROLE, ADMIN_ROLE);
        vault = _vault;
        fee = _fee;
    }

    function submit(
        bytes calldata dest,
        bytes calldata payload
    ) external payable onlyRole(SUBMIT_ROLE) {
        if (msg.value < fee) {
            revert FeePaymentToLow();
        }
        nonce[dest] = nonce[dest] + 1;
        vault.deposit{ value: msg.value }(dest);
        emit Message(dest, nonce[dest], payload);
    }

    function updateFee(uint256 newFee) external onlyRole(ADMIN_ROLE) {
        fee = newFee;
        emit FeeUpdated(fee);
    }
}

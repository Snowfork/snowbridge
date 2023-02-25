// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "@openzeppelin/contracts/access/AccessControl.sol";
import "./IOutboundChannel.sol";
import "./ISovereignTreasury.sol";

contract OutboundChannel is IOutboundChannel, AccessControl {
    bytes32 public constant ADMIN_ROLE = keccak256("ADMIN_ROLE");
    bytes32 public constant SUBMIT_ROLE = keccak256("SUBMIT_ROLE");

    mapping(bytes => uint64) public nonce;

    ISovereignTreasury public sovereignTreasury;
    uint256 public fee;

    event Message(bytes dest, uint64 nonce, bytes payload);

    event FeeUpdated(uint256 fee);
    error FeePaymentToLow();

    constructor(ISovereignTreasury _sovereignTreasury, uint256 _fee) {
        _grantRole(ADMIN_ROLE, msg.sender);
        _setRoleAdmin(SUBMIT_ROLE, ADMIN_ROLE);
        sovereignTreasury = _sovereignTreasury;
        fee = _fee;
    }

    function submit(bytes calldata dest, bytes calldata payload) external payable override onlyRole(SUBMIT_ROLE) {
        if (msg.value < fee) {
            revert FeePaymentToLow();
        }
        sovereignTreasury.deposit{value: msg.value}(dest);
        nonce[dest] = nonce[dest] + 1;
        emit Message(dest, nonce[dest], payload);
    }

    function updateFee(uint256 newFee) external onlyRole(ADMIN_ROLE) {
        fee = newFee;
        emit FeeUpdated(fee);
    }
}

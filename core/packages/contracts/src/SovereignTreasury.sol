// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import "openzeppelin/access/AccessControl.sol";
import "./IVault.sol";

contract SovereignTreasury is AccessControl {
    bytes32 public constant ADMIN_ROLE = keccak256("ADMIN_ROLE");
    bytes32 public constant SENDER_ROLE = keccak256("SENDER_ROLE");

    IVault public vault;

    struct Message {
        Action action;
        bytes payload;
    }

    enum Action {
        // Withdraw from sovereign account and transfer to recipient.
        // Parachain teams will occasionally send this message to retrieve collected fees.
        Withdraw
    }

    struct WithdrawPayload {
        address payable recipient;
        uint256 amount;
    }

    constructor(IVault _vault) {
        _grantRole(ADMIN_ROLE, msg.sender);
        _setRoleAdmin(SENDER_ROLE, ADMIN_ROLE);
        vault = _vault;
    }

    // Handle a message from the bridge.
    function handle(bytes calldata origin, bytes calldata message) external onlyRole(SENDER_ROLE) {
        Message memory decoded = abi.decode(message, (Message));
        if (decoded.action == Action.Withdraw) {
            WithdrawPayload memory payload = abi.decode(decoded.payload, (WithdrawPayload));
            vault.withdraw(origin, payload.recipient, payload.amount);
        }
    }
}

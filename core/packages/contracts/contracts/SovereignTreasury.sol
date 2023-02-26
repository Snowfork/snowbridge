// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "@openzeppelin/contracts/access/AccessControl.sol";
import "./ISovereignTreasury.sol";
import "./EtherVault.sol";

contract SovereignTreasury is ISovereignTreasury, AccessControl {
    bytes32 public constant ADMIN_ROLE = keccak256("ADMIN_ROLE");
    bytes32 public constant WITHDRAW_ROLE = keccak256("WITHDRAW_ROLE");
    bytes32 public constant SENDER_ROLE = keccak256("SENDER_ROLE");

    EtherVault public vault;

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

    constructor(EtherVault _vault) {
        _grantRole(ADMIN_ROLE, msg.sender);
        _setRoleAdmin(WITHDRAW_ROLE, ADMIN_ROLE);
        _setRoleAdmin(SENDER_ROLE, ADMIN_ROLE);
        vault = _vault;
    }

    function deposit(bytes calldata sovereign) external payable {
        vault.deposit{value: msg.value}(sovereign);
    }

    function withdraw(bytes calldata sovereign, address payable recipient, uint256 amount) external onlyRole(WITHDRAW_ROLE) {
        vault.withdraw(sovereign, recipient, amount);
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

// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.20;

import {Auth} from "./Auth.sol";
import {ParaID} from "./Types.sol";

contract Vault is Auth {
    event Deposited(ParaID indexed sovereign, uint256 amount);
    event Withdrawn(ParaID indexed sovereign, address recipient, uint256 amount);

    error InsufficientBalance();
    error ZeroAmount();
    error CannotSendFunds();

    bytes32 public constant WITHDRAW_ROLE = keccak256("WITHDRAW_ROLE");

    // Mapping of sovereign to balance
    mapping(ParaID sovereign => uint256) public balances;

    constructor() {
        _setRoleAdmin(WITHDRAW_ROLE, ADMIN_ROLE);
    }

    receive() external payable {
        revert("Must use deposit function");
    }

    function deposit(ParaID sovereign) external payable {
        balances[sovereign] += msg.value;
        emit Deposited(sovereign, msg.value);
    }

    function withdraw(ParaID sovereign, address payable recipient, uint256 amount) external onlyRole(WITHDRAW_ROLE) {
        if (amount == 0) {
            revert ZeroAmount();
        }

        if (balances[sovereign] < amount) {
            revert InsufficientBalance();
        }

        balances[sovereign] -= amount;

        (bool success,) = recipient.call{value: amount}("");
        if (!success) {
            revert CannotSendFunds();
        }

        emit Withdrawn(sovereign, recipient, amount);
    }
}

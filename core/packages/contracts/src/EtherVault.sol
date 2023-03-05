// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "openzeppelin/access/Ownable.sol";

contract EtherVault is Ownable {
    event Deposited(bytes indexed sovereign, uint256 amount);
    event Withdrawn(bytes indexed sovereign, address recipient, uint256 amount);

    error InsufficientBalance();
    error ZeroAmount();
    error CannotSendFunds();

    // Mapping of sovereign to balance
    mapping(bytes => uint256) private balances;

    receive() external payable {
        revert("Must use deposit function");
    }

    function deposit(bytes calldata sovereign) external payable onlyOwner {
        balances[sovereign] += msg.value;
        emit Deposited(sovereign, msg.value);
    }

    function withdraw(
        bytes calldata sovereign,
        address recipient,
        uint256 amount
    ) external onlyOwner {
        if (amount == 0) {
            revert ZeroAmount();
        }

        if (balances[sovereign] < amount) {
            revert InsufficientBalance();
        }

        balances[sovereign] -= amount;

        // NB: Keep this transfer after reducing the balance to avoid reentrancy attacks.
        // https://consensys.github.io/smart-contract-best-practices/attacks/reentrancy/
        // https://docs.soliditylang.org/en/v0.8.18/security-considerations.html#re-entrancy
        (bool success, ) = recipient.call{ value: amount }("");
        if (!success) {
            revert CannotSendFunds();
        }

        emit Withdrawn(sovereign, recipient, amount);
    }
}

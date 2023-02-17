// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

// This contract actually holds Ether balances for each sovereignID.
contract Vault {
    // Mapping of sovereignID to balance
    mapping(bytes32 => uint256) private balances;

    // TODO: restrict access to SovereignTreasury
    function deposit(bytes32 sovereignID) external payable {
        balances[sovereignID] += msg.value;
    }

    // TODO: restrict access to SovereignTreasury
    function withdraw(bytes32 sovereignID, uint256 amount) external {
        require(balances[sovereignID] >= amount, "Insufficient funds for withdrawal");

        balances[sovereignID] -= amount;
    }
}

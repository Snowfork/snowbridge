// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "@openzeppelin/contracts/utils/Address.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

// TODO: transfer ownership from deployer to SovereignTreasury
// This contract actually holds Ether balances for each sovereignID.
contract EtherVault is Ownable {
    using Address for address payable;

    event Deposited(bytes32 indexed sovereignID, uint256 amount);
    event Withdrawn(bytes32 indexed sovereignID, WithdrawPayload payload);

    // Mapping of sovereignID to balance
    mapping(bytes32 => uint256) private balances;

    receive() external payable {
        revert("Must use deposit function");
    }

    function deposit(bytes32 sovereignID) external payable onlyOwner {
        balances[sovereignID] += msg.value;

        emit Deposited(sovereignID, msg.value);
    }

    function withdraw(bytes32 sovereignID, WithdrawPayload memory payload) external onlyOwner {
        require(payload.amount > 0, "EtherVault: must withdraw a positive amount");
        require(balances[sovereignID] >= payload.amount, "EtherVault: insufficient balance");

        balances[sovereignID] -= payload.amount;

        // NB: Keep this transfer after reducing the balance to avoid reentrancy attacks.
        // https://consensys.github.io/smart-contract-best-practices/attacks/reentrancy/
        // https://docs.soliditylang.org/en/v0.8.18/security-considerations.html#re-entrancy
        payload.recipient.sendValue(payload.amount);

        emit Withdrawn(sovereignID, payload);
    }
}

struct WithdrawPayload {
    address payable recipient;
    uint256 amount;
}

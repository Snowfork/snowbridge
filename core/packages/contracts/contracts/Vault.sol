// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "@openzeppelin/contracts/utils/Address.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

// This contract actually holds Ether balances for each sovereignID.
// TODO: transfer ownership from deployer to SovereignTreasury
contract Vault is Ownable {
    using Address for address payable;

    event Deposited(bytes32 indexed sovereignID, uint256 amount);
    event Withdrawn(bytes32 indexed sovereignID, address recipient, uint256 amount);

    // Mapping of sovereignID to balance
    mapping(bytes32 => uint256) private _balances;

    receive() external payable {
        revert("Must use deposit function");
    }

    function deposit(bytes32 sovereignID) external payable onlyOwner {
        _balances[sovereignID] += msg.value;

        emit Deposited(sovereignID, msg.value);
    }

    function withdraw(
        bytes32 sovereignID,
        address payable recipient,
        uint256 amount
    ) external onlyOwner {
        require(amount > 0, "Vault: must withdraw a positive amount");
        require(_balances[sovereignID] >= amount, "Vault: insufficient balance");

        _balances[sovereignID] -= amount;

        // NB: Keep this transfer after reducing the balance to avoid reentrancy attacks.
        // https://consensys.github.io/smart-contract-best-practices/attacks/reentrancy/
        // https://docs.soliditylang.org/en/v0.8.18/security-considerations.html#re-entrancy
        recipient.sendValue(amount);

        emit Withdrawn(sovereignID, recipient, amount);
    }
}

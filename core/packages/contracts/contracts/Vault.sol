// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "@openzeppelin/contracts/access/Ownable.sol";

// This contract actually holds Ether balances for each sovereignID.
// TODO: transfer ownership from deployer to SovereignTreasury
contract Vault is Ownable {
    // Mapping of sovereignID to balance
    mapping(bytes32 => uint256) private _balances;

    function deposit(bytes32 sovereignID) external payable onlyOwner {
        _balances[sovereignID] += msg.value;
    }

    function withdraw(
        bytes32 sovereignID,
        address payable recipient,
        uint256 amount
    ) external onlyOwner {
        require(_balances[sovereignID] >= amount, "Insufficient funds for withdrawal");

        _balances[sovereignID] -= amount;

        // NB: Keep this transfer as the last statement to avoid reentrancy attacks.
        // https://consensys.github.io/smart-contract-best-practices/attacks/reentrancy/
        // NOTE: Use call instead of transfer or send so that we don't assume a limit on the gas passed to the fallback
        // function.
        // https://consensys.net/diligence/blog/2019/09/stop-using-soliditys-transfer-now
        (bool success, ) = recipient.call{ value: amount }("");
        require(success, "Transfer failed");
    }
}

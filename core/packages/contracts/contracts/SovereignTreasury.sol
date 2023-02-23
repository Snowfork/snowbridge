// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "./Vault.sol";

// TODO: transfer ownership from deployer to inbound channel
contract SovereignTreasury is Ownable {
    Vault public vault;

    constructor(Vault vault) {
        vault = vault;
    }

    // Handle a message from the bridge.
    function handle(bytes32 sovereignID, bytes calldata message) external onlyOwner {
        Message memory message = abi.decode(message, (Message));

        if (message.action == Action.Withdraw) {
            WithdrawPayload memory payload = abi.decode(message.payload, (WithdrawPayload));

            transfer(sovereignID, payload.recipient, payload.amount);
        }
    }

    // Deposit ETH into a sovereign account. Permissionless.
    function deposit(bytes32 sovereignID) external payable {
        vault.deposit{ value: msg.value }(sovereignID);
    }

    function transfer(bytes32 sovereignID, address payable recipient, uint256 amount) private {
        vault.withdraw(sovereignID, recipient, amount);
    }
}

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

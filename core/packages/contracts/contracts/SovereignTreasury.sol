// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "./EtherVault.sol";

// TODO: transfer ownership from deployer to inbound channel
contract SovereignTreasury is Ownable {
    EtherVault public vault;

    constructor(EtherVault vault) {
        vault = vault;
    }

    // Handle a message from the bridge.
    function handle(bytes32 origin, bytes calldata message) external onlyOwner {
        Message memory message = abi.decode(message, (Message));

        if (message.action == Action.Withdraw) {
            WithdrawPayload memory payload = abi.decode(message.payload, (WithdrawPayload));

            transfer(origin, payload);
        }
    }

    // Deposit ETH into a sovereign account. Permissionless.
    function deposit(bytes32 sovereignID) external payable {
        vault.deposit{ value: msg.value }(sovereignID);
    }

    function transfer(bytes32 sovereignID, WithdrawPayload memory payload) private {
        vault.withdraw(sovereignID, payload);
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

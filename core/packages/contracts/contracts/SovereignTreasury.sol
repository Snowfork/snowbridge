// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "./Vault.sol";

contract SovereignTreasury {
    Vault public vault;

    // TODO: emit events?

    constructor(Vault _vault) {
        vault = _vault;
    }

    // Handle a message from the bridge.
    function handle(bytes32 sovereignID, bytes calldata _message) external {
        Message memory message = abi.decode(_message, (Message));

        if (message.action == Action.Withdraw) {
            // TODO: how do we know that the recipient address is payable?
            WithdrawPayload memory payload = abi.decode(message.payload, (WithdrawPayload));

            transfer(sovereignID, payload.recipient, payload.amount);
        }

        // TODO: refund relayer
        // TODO: reward relayer
        /* uint256 rewardAmount = 1; */
        /* transfer(sovereignID, msg.sender, rewardAmount); */
    }

    // Deposit ETH into a sovereign account. Permissionless.
    function deposit(bytes32 sovereignID) external payable {
        vault.deposit(sovereignID, msg.value);
    }

    // Reward a relayer
    function transfer(bytes32 sovereignID, address payable recipient, uint256 amount) private {
        require(msg.value >= amount, "Insufficient funds for transfer");

        vault.withdraw(sovereignID, amount);

        // NB: Keep this transfer as the last statement to avoid reentrancy attacks.
        // https://consensys.github.io/smart-contract-best-practices/attacks/reentrancy/
        (bool success, ) = recipient.call{ value: amount }("");
        require(success, "Transfer failed");
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

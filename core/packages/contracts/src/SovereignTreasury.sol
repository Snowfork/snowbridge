// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.20;

import {AccessControl} from "openzeppelin/access/AccessControl.sol";
import {Vault} from "./Vault.sol";
import {Auth} from "./Auth.sol";
import {Gateway} from "./Gateway.sol";
import {Registry} from "./Registry.sol";

import {IRecipient} from "./IRecipient.sol";

import {ParaID} from "./Types.sol";

contract SovereignTreasury is Gateway {
    Vault public immutable vault;

    struct Message {
        Action action;
        bytes payload;
    }

    enum Action
    // Withdraw from sovereign account and transfer to recipient.
    // Parachain teams will occasionally send this message to retrieve collected fees.
    {Withdraw}

    struct WithdrawPayload {
        address payable recipient;
        uint256 amount;
    }

    constructor(Registry registry, Vault _vault) Gateway(registry) {
        _setRoleAdmin(SENDER_ROLE, ADMIN_ROLE);
        vault = _vault;
    }

    // Handle a message from the bridge.
    function handle(ParaID origin, bytes calldata message) external override onlyRole(SENDER_ROLE) {
        Message memory decoded = abi.decode(message, (Message));
        if (decoded.action == Action.Withdraw) {
            WithdrawPayload memory payload = abi.decode(decoded.payload, (WithdrawPayload));
            vault.withdraw(origin, payload.recipient, payload.amount);
        }
    }
}

// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.20;

import {AccessControl} from "openzeppelin/access/AccessControl.sol";
import {UpgradeTask} from "./UpgradeTask.sol";
import {ParaID} from "./Types.sol";
import {Gateway} from "./Gateway.sol";
import {IRecipient} from "./IRecipient.sol";
import {Registry} from "./Registry.sol";

contract UpgradeProxy is Gateway {
    struct Message {
        Action action;
        bytes payload;
    }

    enum Action {Upgrade}

    struct UpgradePayload {
        address task;
    }

    error InvalidMessage();
    error UpgradeFailed();

    // Parachain ID of BridgeHub
    ParaID public immutable bridgeHubParaID;

    constructor(Registry registry, ParaID _bridgeHubParaID) Gateway(registry) {
        bridgeHubParaID = _bridgeHubParaID;
    }

    function handle(ParaID origin, bytes calldata message) external override onlyRole(SENDER_ROLE) {
        ensureOrigin(origin, bridgeHubParaID);

        Message memory decoded = abi.decode(message, (Message));
        if (decoded.action != Action.Upgrade) {
            revert InvalidMessage();
        }

        UpgradePayload memory payload = abi.decode(decoded.payload, (UpgradePayload));

        (bool success,) = payload.task.delegatecall(abi.encodeCall(UpgradeTask.run, ()));
        if (!success) {
            revert UpgradeFailed();
        }
    }
}

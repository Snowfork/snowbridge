// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import {AccessControl} from "openzeppelin/access/AccessControl.sol";
import {IVault} from "./IVault.sol";
import {IUpgradeTask} from "./IUpgradeTask.sol";
import {ParaID} from "./Types.sol";

contract UpgradeProxy is AccessControl {
    bytes32 public constant ADMIN_ROLE = keccak256("ADMIN_ROLE");
    bytes32 public constant SENDER_ROLE = keccak256("SENDER_ROLE");

    struct Message {
        Action action;
        bytes payload;
    }

    enum Action
    // Withdraw from sovereign account and transfer to recipient.
    // Parachain teams will occasionally send this message to retrieve collected fees.
    {Upgrade}

    struct UpgradePayload {
        address upgrader;
    }

    error InvalidMessage();
    error Unauthorized();
    error UpgradeFailed();

    // Parachain ID of BridgeHub
    ParaID public immutable bridgeHubParaID;

    constructor(ParaID _bridgeHubParaID) {
        _grantRole(ADMIN_ROLE, msg.sender);
        _setRoleAdmin(SENDER_ROLE, ADMIN_ROLE);
        bridgeHubParaID = _bridgeHubParaID;
    }

    function handle(ParaID origin, bytes calldata message) external onlyRole(SENDER_ROLE) {
        if (origin != bridgeHubParaID) {
            revert Unauthorized();
        }

        Message memory decoded = abi.decode(message, (Message));
        if (decoded.action != Action.Upgrade) {
            revert InvalidMessage();
        }

        UpgradePayload memory payload = abi.decode(decoded.payload, (UpgradePayload));

        (bool success,) = payload.upgrader.delegatecall(abi.encodeCall(IUpgradeTask.run, ()));
        if (!success) {
            revert UpgradeFailed();
        }
    }
}

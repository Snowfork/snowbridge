// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.20;

import {AccessControl} from "openzeppelin/access/AccessControl.sol";
import {Registry} from "./Registry.sol";
import {IOutboundQueue} from "./IOutboundQueue.sol";
import {IRecipient} from "./IRecipient.sol";
import {ParaID} from "./Types.sol";
import {Auth} from "./Auth.sol";
import {RegistryLookup} from "./RegistryLookup.sol";

abstract contract Gateway is Auth, RegistryLookup, IRecipient {
    bytes32 public constant SENDER_ROLE = keccak256("SENDER_ROLE");
    bytes32 public constant OUTBOUND_QUEUE = keccak256("OutboundQueue");

    /* Errors */

    error Unauthorized();

    constructor(Registry registry) RegistryLookup(registry) {
        _setRoleAdmin(SENDER_ROLE, ADMIN_ROLE);
    }

    function handle(ParaID origin, bytes calldata message) external virtual;

    function outboundQueue() internal view returns (IOutboundQueue) {
        return IOutboundQueue(resolve(OUTBOUND_QUEUE));
    }

    function ensureOrigin(ParaID a, ParaID b) internal pure {
        if (a != b) {
            revert Unauthorized();
        }
    }
}

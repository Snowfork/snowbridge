// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import {ParaID} from "./Types.sol";
import {IOutboundQueue} from "./IOutboundQueue.sol";
import {IOutboundQueueDelegate} from "./IOutboundQueueDelegate.sol";
import {AccessControl} from "openzeppelin/access/AccessControl.sol";

/// OutboundQueue accepts messages for delivery to the BridgeHub parachain
contract OutboundQueue is IOutboundQueue, AccessControl {

    bytes32 public constant ADMIN_ROLE = keccak256("ADMIN_ROLE");
    bytes32 public constant SUBMIT_ROLE = keccak256("SUBMIT_ROLE");

    /// The actual implementation contract
    IOutboundQueueDelegate public delegate;

    event DelegateUpdated(IOutboundQueueDelegate delegate);

    constructor() {
        _grantRole(ADMIN_ROLE, msg.sender);
        _setRoleAdmin(ADMIN_ROLE, ADMIN_ROLE);
        _setRoleAdmin(SUBMIT_ROLE, ADMIN_ROLE);
    }

    /// Submit a message `payload` for delivery to parachain `dest`.
    function submit(ParaID dest, bytes calldata params, bytes calldata payload) external payable onlyRole(SUBMIT_ROLE) {
        uint64 nonce = delegate.submit{value: msg.value}(msg.sender, dest, params);
        emit MessageAccepted(dest, nonce, payload);
    }

    function updateDelegate(IOutboundQueueDelegate _delegate) external onlyRole(ADMIN_ROLE) {
        delegate = _delegate;
        emit DelegateUpdated(_delegate);
    }
}

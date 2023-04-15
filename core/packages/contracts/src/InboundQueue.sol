// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import {ParaID} from "./Types.sol";
import {IInboundQueueDelegate} from "./IInboundQueueDelegate.sol";
import {AccessControl} from "openzeppelin/access/AccessControl.sol";

/// InboundQueue receives messages from the BridgeHub parachain on Polkadot,
/// and then verifies and dispatches them.
contract InboundQueue is AccessControl {

    /// This role is allowed to administer the contract
    bytes32 public constant ADMIN_ROLE = keccak256("ADMIN_ROLE");

    /// Actual implementation contract
    IInboundQueueDelegate public delegate;

    event DelegateUpdated(IInboundQueueDelegate delegate);

    constructor() {
        // Give admin rights to caller
        _grantRole(ADMIN_ROLE, msg.sender);
        _setRoleAdmin(ADMIN_ROLE, ADMIN_ROLE);
    }

    /// Submit a single message
    function submit(bytes calldata message) external {
        delegate.submit(payable(msg.sender), message);
    }

    /// Submit a batch of messages
    function submitBatch(bytes calldata message) external {
        delegate.submitBatch(payable(msg.sender), message);
    }

    /// Update the implementation contract
    function updateDelegate(IInboundQueueDelegate _delegate) external onlyRole(ADMIN_ROLE) {
        delegate = _delegate;
        emit DelegateUpdated(_delegate);
    }
}

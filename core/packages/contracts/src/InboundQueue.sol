// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import {ParaID} from "./Types.sol";
import {IInboundQueueDelegate} from "./IInboundQueueDelegate.sol";
import {AccessControl} from "openzeppelin/access/AccessControl.sol";

contract InboundQueue is AccessControl {

    bytes32 public constant ADMIN_ROLE = keccak256("ADMIN_ROLE");

    IInboundQueueDelegate public delegate;

    event DelegateUpdated(IInboundQueueDelegate delegate);

    function submit(bytes calldata message) external {
        delegate.submit(message);
    }

    function updateDelegate(IInboundQueueDelegate _delegate) external onlyRole(ADMIN_ROLE) {
        delegate = _delegate;
        emit DelegateUpdated(_delegate);
    }
}

// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import {ParaID} from "./Types.sol";

contract InboundQueue {

    address public delegate;

    // Inbound message from BridgeHub parachain
    struct Message {
        ParaID origin;
        uint64 nonce;
        uint16 handler;
        bytes payload;
    }

    enum DispatchResult {
        Success,
        Failure
    }

    event MessageDispatched(ParaID indexed origin, uint64 indexed nonce, DispatchResult result);

    function submit(bytes calldata message) external {

    }
}

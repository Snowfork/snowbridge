// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import {IOutboundQueue} from "../../src/IOutboundQueue.sol";
import {ParaID} from "../../src/Types.sol";

contract OutboundQueueMock is IOutboundQueue {
    uint64 public nonce;

    function submit(ParaID dest, bytes calldata params, bytes calldata payload) external payable {
        emit MessageAccepted(dest, ++nonce, payload);
    }
}

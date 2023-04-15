// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import {ParaID} from "./Types.sol";

interface IInboundQueueDelegate {
    function submit(address payable relayer, bytes calldata opaqueMessage) external;
    function submitBatch(address payable relayer, bytes calldata opaqueMessage) external;
}

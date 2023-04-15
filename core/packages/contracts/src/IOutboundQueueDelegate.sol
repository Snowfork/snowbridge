// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import {ParaID} from "./Types.sol";

interface IOutboundQueueDelegate {
    function submit(address origin, ParaID dest, bytes calldata params) external payable returns (uint64);
}

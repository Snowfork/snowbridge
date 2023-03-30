// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import {ParaID} from "./Types.sol";

interface IOutboundQueue {
    function submit(ParaID dest, bytes calldata payload) external payable;
}

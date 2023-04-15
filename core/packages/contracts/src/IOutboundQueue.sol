// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import {ParaID} from "./Types.sol";

interface IOutboundQueue {
    event MessageAccepted(ParaID indexed dest, uint64 indexed nonce, bytes payload);

    function submit(ParaID dest, bytes calldata params, bytes calldata payload) external payable;
}

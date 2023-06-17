// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.20;

import {ParaID} from "./Types.sol";

interface IOutboundQueue {
    event Message(ParaID indexed dest, uint64 indexed nonce, bytes payload);

    function submit(ParaID dest, bytes calldata payload) external payable;
}

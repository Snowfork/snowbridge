// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.20;

import {ParaID} from "./Types.sol";

interface IRecipient {
    function handle(ParaID origin, bytes calldata message) external;
}

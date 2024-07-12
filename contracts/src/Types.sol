// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.25;

import "./types/Common.sol";
import {InboundMessage as InboundMessageV1, Command as CommandV1, AgentExecuteCommand} from "./types/V1.sol";
import {InboundMessage as InboundMessageV2, Command as CommandV2} from "./types/V2.sol";

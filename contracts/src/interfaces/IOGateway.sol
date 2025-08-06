//SPDX-License-Identifier: GPL-3.0-or-later

// Copyright (C) Moondance Labs Ltd.
// This file is part of Tanssi.
// Tanssi is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
// Tanssi is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
// You should have received a copy of the GNU General Public License
// along with Tanssi.  If not, see <http://www.gnu.org/licenses/>
pragma solidity ^0.8.0;

import {ParaID, Command} from "../Types.sol";
import {IGateway} from "./IGateway.sol";

interface IOGateway is IGateway {
    // Emitted when operators data has been created
    event OperatorsDataCreated(uint256 indexed validatorsCount, bytes payload);

    // Emitted when owner of the gateway is changed.
    event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);

    // Emitted when the middleware contract address is changed by the owner.
    event MiddlewareChanged(address indexed previousMiddleware, address indexed newMiddleware);

    // Emitted when the middleware fails to apply an individual slash
    event UnableToProcessIndividualSlashB(
        bytes32 indexed operatorKey, uint256 slashFranction, uint256 indexed epoch, bytes error
    );

    // Emitted when the middleware fails to apply an individual slash
    event UnableToProcessIndividualSlashS(
        bytes32 indexed operatorKey, uint256 slashFranction, uint256 indexed epoch, string error
    );

    // Emitted when the middleware fails to apply the slash message
    event UnableToProcessSlashMessageB(bytes error);

    // Emitted when the middleware fails to apply the slash message
    event UnableToProcessSlashMessageS(string error);

    // Emitted when the middleware fails to apply the slash message
    event UnableToProcessRewardsMessageB(bytes error);

    // Emitted when the middleware fails to apply the slash message
    event UnableToProcessRewardsMessageS(string error);

    // Emitted when a non accepted command is received
    event NotImplementedCommand(Command command);

    // Slash struct, used to decode slashes, which are identified by
    // operatorKey to be slashed
    // slashFraction to be applied as parts per billion
    // epoch identifying when the slash happened
    struct Slash {
        bytes32 operatorKey;
        uint256 slashFraction;
        uint256 epoch;
    }

    struct SlashParams {
        uint256 eraIndex;
        Slash[] slashes;
    }

    function s_middleware() external view returns (address);

    function reportSlashes(bytes calldata data) external;

    function sendRewards(bytes calldata data) external;

    function sendOperatorsData(bytes32[] calldata data, uint48 epoch) external;

    function setMiddleware(address middleware) external;
}

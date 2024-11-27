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
pragma solidity 0.8.25;

import {BeefyClient} from "./BeefyClient.sol";
import {ScaleCodec} from "./utils/ScaleCodec.sol";
import {SubstrateTypes} from "./SubstrateTypes.sol";
import {MultiAddress, Ticket, Costs, ParaID} from "./Types.sol";
import {IGateway} from "./interfaces/IGateway.sol";

library Validators {
    error Validators__UnsupportedValidatorsLength();

    function encodeValidatorsData(bytes calldata validatorsKeys) internal returns (Ticket memory ticket) {
        uint256 validatorsKeysLength = validatorsKeys.length;
        if (validatorsKeysLength / 32 > 1000) {
            revert Validators__UnsupportedValidatorsLength();
        }

        ticket.dest = ParaID.wrap(3);
        ticket.costs = Costs(0, 0);

        ticket.payload = SubstrateTypes.EncodedValidatorsData(validatorsKeys, uint32(validatorsKeysLength / 32));
        emit IGateway.ValidatorsDataCreated(ticket.dest, ticket.payload);
    }
}

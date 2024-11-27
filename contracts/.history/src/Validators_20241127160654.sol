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
    error Validators__ValidatorsLengthTooLong();

    uint8 private constant VALIDATOR_KEY_HEX_LENGTH = 32 * 2;

    function encodeValidatorsData(bytes calldata validatorsKeys, ParaID dest) internal returns (Ticket memory ticket) {
        if (validatorsKeys.length % VALIDATOR_KEY_HEX_LENGTH != 0) {
            revert Validators__UnsupportedValidatorsLength();
        }
        uint256 validatorsKeysLength = validatorsKeys.length / VALIDATOR_KEY_HEX_LENGTH;

        if (validatorsKeysLength > 1000) {
            revert Validators__ValidatorsLengthTooLong();
        }

        ticket.dest = dest;
        // For now mock it to 0
        ticket.costs = Costs(0, 0);

        ticket.payload = SubstrateTypes.EncodedValidatorsData(validatorsKeys, uint32(validatorsKeysLength));
        emit IGateway.ValidatorsDataCreated(validatorsKeysLength, ticket.payload);
    }
}

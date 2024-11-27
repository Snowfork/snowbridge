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
import {MultiAddress, Ticket, Costs} from "./Types.sol";

library Validators {
    function encodeValidatorsData(bytes calldata data) internal pure returns (Ticket memory ticket) {
        ticket.dest = 
        ticket.costs = _sendTokenCosts(destinationChain, destinationChainFee, maxDestinationChainFee);

        // Construct a message payload
        if (destinationChain == $.assetHubParaID && destinationAddress.isAddress32()) {
            // The funds will be minted into the receiver's account on AssetHub
            // The receiver has a 32-byte account ID
            ticket.payload = SubstrateTypes.SendForeignTokenToAssetHubAddress32(
                foreignID, destinationAddress.asAddress32(), $.assetHubReserveTransferFee, amount
            );
        emit IGateway.TokenSent(token, sender, destinationChain, destinationAddress, amount);
        return ScaleCodec.decodeValidatorSet(data);


    }
}

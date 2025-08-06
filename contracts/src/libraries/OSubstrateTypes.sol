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

import {ScaleCodec} from "../utils/ScaleCodec.sol";
import {ParaID} from "../Types.sol";

library OSubstrateTypes {
    enum Message {
        V0
    }

    enum OutboundCommandV1 {
        ReceiveValidators
    }

    function EncodedOperatorsData(bytes memory encodedOperatorsKeys, uint32 operatorsCount, uint48 epoch)
        internal
        pure
        returns (bytes memory)
    {
        return bytes.concat(
            bytes4(0x70150038),
            bytes1(uint8(Message.V0)),
            bytes1(uint8(OutboundCommandV1.ReceiveValidators)),
            ScaleCodec.encodeCompactU32(operatorsCount),
            encodedOperatorsKeys,
            ScaleCodec.encodeU64(uint64(epoch))
        );
    }
}

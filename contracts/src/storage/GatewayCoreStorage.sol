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

library GatewayCoreStorage {
    struct Layout {
        // Owner of the gateway for configuration purposes.
        address owner;
        // Address of the Symbiotic middleware to properly execute messages.
        address middleware;
    }

    bytes32 internal constant SLOT = keccak256("tanssi-bridge-relayer.gateway.core");

    function layout() internal pure returns (Layout storage ptr) {
        bytes32 slot = SLOT;
        assembly {
            ptr.slot := slot
        }
    }
}

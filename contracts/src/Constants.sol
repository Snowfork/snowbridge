// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.25;

import {ParaID, ChannelID} from "./v1/Types.sol";

library Constants {
    ParaID constant ASSET_HUB_PARA_ID = ParaID.wrap(1000);
    bytes32 constant ASSET_HUB_AGENT_ID =
        0x81c5ab2571199e3188135178f3c2c8e2d268be1313d029b30f534fa579b69b79;

    ParaID constant BRIDGE_HUB_PARA_ID = ParaID.wrap(1002);
    bytes32 constant BRIDGE_HUB_AGENT_ID =
        0x03170a2e7597b7b7e3d84c05391d139a62b157e78786d8c082f29dcf4c111314;

    // ChannelIDs
    ChannelID internal constant PRIMARY_GOVERNANCE_CHANNEL_ID =
        ChannelID.wrap(bytes32(uint256(1)));
    ChannelID internal constant SECONDARY_GOVERNANCE_CHANNEL_ID =
        ChannelID.wrap(bytes32(uint256(2)));
}

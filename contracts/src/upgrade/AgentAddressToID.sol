// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.23;

import "../Gateway.sol";

contract GatewayWithAgentAddressToID is Gateway {
    event Upgraded();

    constructor(
        address beefyClient,
        address agentExecutor,
        ParaID bridgeHubParaID,
        bytes32 bridgeHubAgentID,
        uint8 foreignTokenDecimals
    ) Gateway(beefyClient, agentExecutor, bridgeHubParaID, bridgeHubAgentID, foreignTokenDecimals) {}

    function initialize(bytes memory params) external override {
        CoreStorage.Layout storage core = CoreStorage.layout();
        address bridgeHubAgent = core.agents[BRIDGE_HUB_AGENT_ID];
        core.agentAddresses[bridgeHubAgent] = BRIDGE_HUB_AGENT_ID;

        (ASSET_HUB_PARA_ID, ASSET_HUB_AGENT_ID) = abi.decode(params, (ParaID, bytes32));
        address assetHubAgent = core.agents[ASSET_HUB_AGENT_ID];
        core.agentAddresses[assetHubAgent] = ASSET_HUB_AGENT_ID;
        emit Upgraded();
    }
}

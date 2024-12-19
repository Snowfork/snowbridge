// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.25;

import "../Gateway.sol";

// New `Gateway` logic contract for the `GatewayProxy` deployed on mainnet
contract Gateway202410 is Gateway {
    constructor(
        address beefyClient,
        address agentExecutor,
        ParaID bridgeHubParaID,
        bytes32 bridgeHubAgentID,
        uint8 foreignTokenDecimals,
        uint128 destinationMaxTransferFee
    )
        Gateway(
            beefyClient,
            agentExecutor,
            bridgeHubParaID,
            bridgeHubAgentID,
            foreignTokenDecimals,
            destinationMaxTransferFee
        )
    {}

    // Override parent initializer to prevent re-initialization of storage.
    function initialize(bytes memory) external override {
        // Ensure that arbitrary users cannot initialize storage in this logic contract.
        if (ERC1967.load() == address(0)) {
            revert Unauthorized();
        }

        // We expect version 0, deploying version 1.
        CoreStorage.Layout storage $ = CoreStorage.layout();
        if ($.version != 0) {
            revert Unauthorized();
        }
        $.version = 1;

        // migrate asset hub agent
        address agent = _ensureAgent(hex"81c5ab2571199e3188135178f3c2c8e2d268be1313d029b30f534fa579b69b79");
        bytes memory call =
            abi.encodeCall(AgentExecutor.transferNativeToGateway, (payable(address(this)), agent.balance));
        _invokeOnAgent(agent, call);
    }
}

// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.25;

import "../../Gateway.sol";

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

    function initialize(bytes memory) external override {
        if (ERC1967.load() == address(0)) {
            revert Unauthorized();
        }
    }
}

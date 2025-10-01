// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.28;

import "../Gateway.sol";
import {UnlockNativeTokenParams} from "../v2/Types.sol";

// New Gateway logic contract with an asset recovery initializer
contract GatewayAssetRecovery202510 is Gateway {
    constructor(address beefyClient, address agentExecutor) Gateway(beefyClient, agentExecutor) {}

    function initialize(bytes calldata) external override {
        if (ERC1967.load() == address(0)) {
            revert Unauthorized();
        }

        UnlockNativeTokenParams memory params = UnlockNativeTokenParams({
            token: address(0),
            recipient: 0xAd8D4c544a6ce24B89841354b2738E026a12BcA4,
            amount: 350000000000000000
        });

        HandlersV2.unlockNativeToken(AGENT_EXECUTOR, abi.encode(params));

        // Todo: add any other recovery actions here
    }
}

// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.34;

import {Gateway} from "../Gateway.sol";
import {ERC1967} from "../utils/ERC1967.sol";

/// @notice Gateway implementation prepared for the October 2026 mainnet upgrade.
contract Gateway202610 is Gateway {
    constructor(address beefyClient, address agentExecutor) Gateway(beefyClient, agentExecutor) {}

    /// @dev This upgrade does not migrate storage. Prevent initialization of the standalone
    ///      implementation while allowing the proxy-mediated upgrade call.
    function initialize(bytes calldata) external view override {
        if (ERC1967.load() == address(0)) {
            revert Unauthorized();
        }
    }
}

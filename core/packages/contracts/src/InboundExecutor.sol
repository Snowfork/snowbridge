// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.20;

import {AccessControl} from "openzeppelin/access/AccessControl.sol";
import {ParaID} from "./Types.sol";
import {SubstrateTypes} from "./SubstrateTypes.sol";

import {IERC20Metadata} from "openzeppelin/token/ERC20/extensions/IERC20Metadata.sol";
import {IERC20} from "openzeppelin/token/ERC20/IERC20.sol";
import {SafeERC20} from "openzeppelin/token/ERC20/utils/SafeERC20.sol";

contract Executor {
    using SafeERC20 for IERC20;

    bytes32 public constant ACTION_UNLOCK_TOKENS = keccak256("unlockTokens");

    function execute(bytes calldata data) external {
        (bytes32 action, bytes memory actionData) = abi.decode(data, (bytes32, bytes));
        if (action == ACTION_UNLOCK_TOKENS) {
            unlockNativeTokens(actionData);
        }
    }

    function unlockNativeTokens(bytes memory data) internal {
        (address token, address recipient, uint128 amount) = abi.decode(data, (address, address, uint128));
        IERC20(token).safeTransfer(recipient, amount);
    }
}

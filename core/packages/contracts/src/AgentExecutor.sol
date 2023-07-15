// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.20;

import {AccessControl} from "openzeppelin/access/AccessControl.sol";
import {ParaID} from "./Types.sol";
import {SubstrateTypes} from "./SubstrateTypes.sol";

import {IGateway} from "./IGateway.sol";

import {IERC20Metadata} from "openzeppelin/token/ERC20/extensions/IERC20Metadata.sol";
import {IERC20} from "openzeppelin/token/ERC20/IERC20.sol";
import {SafeERC20} from "openzeppelin/token/ERC20/utils/SafeERC20.sol";

import {console} from "forge-std/console.sol";

contract AgentExecutor {
    using SafeERC20 for IERC20;

    bytes32 public constant ACTION_UNLOCK_TOKENS = keccak256("unlockTokens");

    event NativeTokensUnlocked(address token, address recipient, uint256 amount);

    error InsufficientBalance();
    error TransferEthFailed();
    error Unauthorized();

    function execute(address, bytes memory data) external {
        (bytes32 action, bytes memory actionData) = abi.decode(data, (bytes32, bytes));
        if (action == ACTION_UNLOCK_TOKENS) {
            unlockNativeTokens(actionData);
        }
    }

    function unlockNativeTokens(bytes memory data) internal {
        (address token, address recipient, uint256 amount) = abi.decode(data, (address, address, uint256));
        IERC20(token).safeTransfer(recipient, amount);
        emit NativeTokensUnlocked(token, recipient, amount);
    }

    function withdrawTo(address payable recipient, uint256 amount) external {
        if (address(this).balance < amount) {
            revert InsufficientBalance();
        }
        (bool success,) = recipient.call{value: amount}("");
        if (!success) {
            revert TransferEthFailed();
        }
    }
}

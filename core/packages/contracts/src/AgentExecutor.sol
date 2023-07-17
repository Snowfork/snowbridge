// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.20;

import {ParaID} from "./Types.sol";
import {SubstrateTypes} from "./SubstrateTypes.sol";

import {IGateway} from "./IGateway.sol";

import {IERC20} from "./interfaces/IERC20.sol";
import {SafeTokenTransfer, SafeNativeTransfer} from "./utils/SafeTransfer.sol";

import {console} from "forge-std/console.sol";

contract AgentExecutor {
    using SafeTokenTransfer for IERC20;
    using SafeNativeTransfer for address payable;

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
        recipient.safeNativeTransfer(amount);
    }
}

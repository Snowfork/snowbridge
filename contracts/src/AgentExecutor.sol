// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.28;

import {IERC20} from "./interfaces/IERC20.sol";
import {SafeTokenTransfer, SafeNativeTransfer} from "./utils/SafeTransfer.sol";
import {Call} from "./utils/Call.sol";
import {CallContractParams} from "./v2/Types.sol";

/// @title Code which will run within an `Agent` using `delegatecall`.
/// @dev This is a singleton contract, meaning that all agents will execute the same code.
contract AgentExecutor {
    using SafeTokenTransfer for IERC20;
    using SafeNativeTransfer for address payable;

    // Transfer ether to `recipient`.
    function transferEther(address payable recipient, uint256 amount) external {
        recipient.safeNativeTransfer(amount);
    }

    // Transfer ERC20 to `recipient`.
    function transferToken(address token, address recipient, uint128 amount) external {
        IERC20(token).safeTransfer(recipient, amount);
    }

    // Call contract with Ether value
    function callContract(CallContractParams calldata params) external {
        bool success = Call.safeCall(params.target, params.data, params.value);
        if (!success) {
            revert();
        }
    }

    // Call multiple contracts with Ether values; reverts on the first failure
    function callContracts(CallContractParams[] calldata params) external {
        uint256 len = params.length;
        for (uint256 i; i < len; ++i) {
            bool success = Call.safeCall(params[i].target, params[i].data, params[i].value);
            if (!success) {
                revert();
            }
        }
    }
}

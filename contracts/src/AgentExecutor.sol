// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.20;

import {AgentExecuteCommand, ParaID} from "./Types.sol";
import {SubstrateTypes} from "./SubstrateTypes.sol";

import {IERC20} from "./interfaces/IERC20.sol";
import {SafeTokenTransfer, SafeNativeTransfer} from "./utils/SafeTransfer.sol";

/// @title Code which will run within an `Agent` using `delegatecall`.
/// @dev This is a singleton contract, meaning that all agents will execute the same code.
contract AgentExecutor {
    using SafeTokenTransfer for IERC20;
    using SafeNativeTransfer for address payable;

    error NotEnoughGas();

    event TransactFailed(address target, bytes result, bytes payload);
    event TransactSucceeded(bytes result);

    /// @dev Execute a message which originated from the Polkadot side of the bridge. In other terms,
    /// the `data` parameter is constructed by the BridgeHub parachain.
    ///
    /// NOTE: There are no authorization checks here. Since this contract is only used for its code.
    function execute(address, bytes memory data) external {
        (AgentExecuteCommand command, bytes memory params) = abi.decode(data, (AgentExecuteCommand, bytes));

        if (command == AgentExecuteCommand.TransferToken) {
            (address token, address recipient, uint128 amount) = abi.decode(params, (address, address, uint128));
            _transferToken(token, recipient, amount);
        }
        if (command == AgentExecuteCommand.Transact) {
            (address target, bytes memory payload, uint256 dynamicGas) = abi.decode(params, (address, bytes, uint256));
            _executeCall(target, payload, dynamicGas);
        }
    }

    /// @dev Transfer ether to `recipient`. Unlike `_transferToken` This logic is not nested within `execute`,
    /// as the gateway needs to control an agent's ether balance directly.
    ///
    /// NOTE: There are no authorization checks here. Since this contract is only used for its code.
    function transferNative(address payable recipient, uint256 amount) external {
        recipient.safeNativeTransfer(amount);
    }

    /// @dev Transfer ERC20 to `recipient`. Only callable via `execute`.
    function _transferToken(address token, address recipient, uint128 amount) internal {
        IERC20(token).safeTransfer(recipient, amount);
    }

    /// @dev Call a contract at the given address, with provided bytes as payload.
    function _executeCall(address target, bytes memory payload, uint256 dynamicGas) internal {
        if (gasleft() < dynamicGas) {
            revert NotEnoughGas();
        }
        (bool success, bytes memory result) = target.call{gas: 200000}(payload);
        if (success) {
            emit TransactSucceeded(result);
        } else {
            emit TransactFailed(target, result, payload);
        }
    }
}

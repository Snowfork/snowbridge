// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.20;

import {AgentExecuteCommand, ParaID} from "./Types.sol";
import {SubstrateTypes} from "./SubstrateTypes.sol";

import {Agent} from "./Agent.sol";
import {ERC20} from "./ERC20.sol";
import {Gateway} from "./Gateway.sol";
import {IERC20} from "./interfaces/IERC20.sol";
import {SafeTokenTransfer, SafeNativeTransfer} from "./utils/SafeTransfer.sol";

/// @title Code which will run within an `Agent` using `delegatecall`.
/// @dev This is a singleton contract, meaning that all agents will execute the same code.
contract AgentExecutor {
    using SafeTokenTransfer for IERC20;
    using SafeNativeTransfer for address payable;

    event TokenRegistered(address ownerAgent, address token);

    // NOTE: These variables must match those in Agent.sol, in type, name & order of declaration.
    // These storage variables allow access to the values on the agent when the methods below are invoked with
    // delegatecall.
    bytes32 public AGENT_ID;
    address public GATEWAY;

    /// @dev Execute a message which originated from the Polkadot side of the bridge. In other terms,
    /// the `data` parameter is constructed by the BridgeHub parachain.
    ///
    /// NOTE: There are no authorization checks here. Since this contract is only used for its code.
    function execute(address, bytes memory data) external {
        (AgentExecuteCommand command, bytes memory params) = abi.decode(data, (AgentExecuteCommand, bytes));

        if (command == AgentExecuteCommand.TransferToken) {
            (address token, address recipient, uint128 amount) = abi.decode(params, (address, address, uint128));
            _transferToken(token, recipient, amount);
        } else if (command == AgentExecuteCommand.RegisterToken) {
            (string memory name, string memory symbol, uint8 decimals) = abi.decode(params, (string, string, uint8));
            _registerToken(name, symbol, decimals);
        } else if (command == AgentExecuteCommand.MintToken) {
            (address token, address recipient, uint256 amount) = abi.decode(params, (address, address, uint256));
            _mintToken(token, recipient, amount);
        }
    }

    /// @dev Transfer ether to `recipient`. Unlike `_transferToken` This logic is not nested within `execute`,
    /// as the gateway needs to control an agent's ether balance directly.
    ///
    /// NOTE: There are no authorization checks here. Since this contract is only used for its code.
    function transferNative(address payable recipient, uint256 amount) external {
        recipient.safeNativeTransfer(amount);
    }

    function burnToken(address token, address sender, uint256 amount) external {
        ERC20(token).burn(sender, amount);
    }

    /// @dev Create a new ERC20 token with this agent as the owner.
    function _registerToken(string memory name, string memory symbol, uint8 decimals) internal {
        IERC20 token = new ERC20(name, symbol, decimals);
        Gateway(GATEWAY).setTokenOwnerAgentID(address(token), AGENT_ID);
        emit TokenRegistered(address(this), address(token));
    }

    /// @dev Mint ERC20 `token` and transfer to `recipient`. Only callable via `execute`.
    function _mintToken(address token, address recipient, uint256 amount) internal {
        ERC20(token).mint(recipient, amount);
    }

    /// @dev Transfer ERC20 to `recipient`. Only callable via `execute`.
    function _transferToken(address token, address recipient, uint128 amount) internal {
        IERC20(token).safeTransfer(recipient, amount);
    }
}

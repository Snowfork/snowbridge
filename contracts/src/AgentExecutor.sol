// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.23;

import {AgentExecuteCommand, ParaID} from "./Types.sol";
import {SubstrateTypes} from "./SubstrateTypes.sol";

import {IERC20} from "./interfaces/IERC20.sol";
import {SafeTokenTransfer, SafeNativeTransfer} from "./utils/SafeTransfer.sol";
import {ERC20} from "./ERC20.sol";
import {Gateway} from "./Gateway.sol";

/// @title Code which will run within an `Agent` using `delegatecall`.
/// @dev This is a singleton contract, meaning that all agents will execute the same code.
contract AgentExecutor {
    using SafeTokenTransfer for IERC20;
    using SafeNativeTransfer for address payable;

    // Emitted when token minted
    event TokenMinted(bytes32 indexed tokenID, address token, address recipient, uint256 amount);

    /// @dev Execute a message which originated from the Polkadot side of the bridge. In other terms,
    /// the `data` parameter is constructed by the BridgeHub parachain.
    ///
    function execute(bytes32 agentID, bytes memory data) external {
        (AgentExecuteCommand command, bytes memory params) = abi.decode(data, (AgentExecuteCommand, bytes));
        if (command == AgentExecuteCommand.TransferToken) {
            (address token, address recipient, uint128 amount) = abi.decode(params, (address, address, uint128));
            _transferToken(token, recipient, amount);
        } else if (command == AgentExecuteCommand.RegisterToken) {
            (bytes32 tokenID, string memory name, string memory symbol, uint8 decimals) =
                abi.decode(params, (bytes32, string, string, uint8));
            _registerToken(agentID, tokenID, name, symbol, decimals);
        } else if (command == AgentExecuteCommand.MintToken) {
            (bytes32 tokenID, address recipient, uint256 amount) = abi.decode(params, (bytes32, address, uint256));
            _mintToken(tokenID, recipient, amount);
        }
    }

    /// @dev Transfer ether to `recipient`. Unlike `_transferToken` This logic is not nested within `execute`,
    /// as the gateway needs to control an agent's ether balance directly.
    ///
    function transferNative(address payable recipient, uint256 amount) external {
        recipient.safeNativeTransfer(amount);
    }

    /// @dev Transfer ERC20 to `recipient`. Only callable via `execute`.
    function _transferToken(address token, address recipient, uint128 amount) internal {
        IERC20(token).safeTransfer(recipient, amount);
    }

    /// @dev Register native asset from polkadto as ERC20 `token`.
    function _registerToken(bytes32 agentID, bytes32 tokenID, string memory name, string memory symbol, uint8 decimals)
        internal
    {
        IERC20 token = new ERC20(name, symbol, decimals);
        Gateway(msg.sender).registerTokenByID(tokenID, address(token), agentID);
    }

    /// @dev Mint ERC20 token to `recipient`.
    function _mintToken(bytes32 tokenID, address recipient, uint256 amount) internal {
        address token = Gateway(msg.sender).getTokenAddress(tokenID);
        ERC20(token).mint(recipient, amount);
        emit TokenMinted(tokenID, token, recipient, amount);
    }
}

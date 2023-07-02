// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.20;

import {AccessControl} from "openzeppelin/access/AccessControl.sol";
import {Registry} from "./Registry.sol";
import {IOutboundQueue} from "./IOutboundQueue.sol";
import {IRecipient} from "./IRecipient.sol";
import {ParaID} from "./Types.sol";
import {Auth} from "./Auth.sol";
import {RegistryLookup} from "./RegistryLookup.sol";
import {IExecutor} from "./IExecutor.sol";
import {IERC20} from "openzeppelin/token/ERC20/IERC20.sol";
import {SafeERC20} from "openzeppelin/token/ERC20/utils/SafeERC20.sol";
import {SubstrateTypes} from "./SubstrateTypes.sol";
import {NativeTokensTypes} from "./NativeTokensTypes.sol";

contract Executor is IExecutor {
    using SafeERC20 for IERC20;

    bytes32 public constant ACTION_UNLOCK_TOKENS = keccak256("unlockTokens");

    address public immutable gateway;

    constructor(address gateway) {}

    function execute(bytes calldata data) external {
        (bytes32 action, bytes memory actionData) = abi.decode(data, (bytes32, bytes));

        if (action == ACTION_UNLOCK_TOKENS) {
            (address token, address recipient, uint128 amount) = abi.decode(actionData, (address, address, uint128));
            doUnlockTokens(token, recipient, amount);
        }
    }

    /// @dev Locks an amount of ERC20 Tokens in the vault and enqueues a mint message.
    /// Requires the allowance to be set on the ERC20 token where the spender is the vault.
    /// @param token The token to lock.
    /// @param recipient The recipient on the substrate side.
    /// @param amount The amount to lock.
    function sendTokens(address token, ParaID dest, bytes calldata recipient, uint128 amount) external payable {
        if (amount == 0) {
            revert InvalidAmount();
        }

        vault.deposit(msg.sender, token, amount);

        bytes memory payload = NativeTokensTypes.Mint(address(registry), token, dest, recipient, amount);
        outboundQueue().submit{value: msg.value}(assetHubParaID, payload);

        emit Locked(recipient, token, amount);
    }

    /// @dev Enqueues a create native token message to substrate.
    /// @param token The ERC20 token address.
    function createTokens(address token) external payable {
        // to avoid spam, charge a fee for creating a new token
        if (msg.value < createTokenFee) {
            revert NoFundsforCreateToken();
        }

        IERC20Metadata metadata = IERC20Metadata(token);
        bytes memory name = bytes(metadata.name());
        if (name.length > 32) {
            name = hex"";
        }
        bytes memory symbol = bytes(metadata.symbol());
        if (symbol.length > 32) {
            symbol = hex"";
        }
        uint8 decimals = metadata.decimals();

        bytes memory payload =
            NativeTokensTypes.Create(address(registry), token, name, symbol, decimals, createCallId, setMetadataCallId);
        outboundQueue().submit{value: msg.value}(assetHubParaID, payload);

        emit Created(token);
    }

    function doUnlockTokens(address token, address recipient, uint128 amount) internal {
        IERC20(token).safeTransfer(recipient, amount);
    }
}

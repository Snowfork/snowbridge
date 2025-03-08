// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.28;

import {IERC20} from "./interfaces/IERC20.sol";
import {SafeNativeTransfer, SafeTokenTransferFrom} from "./utils/SafeTransfer.sol";
import {Agent} from "./Agent.sol";
import {Call} from "./utils/Call.sol";
import {Address} from "./utils/Address.sol";
import {AgentExecutor} from "./AgentExecutor.sol";
import {CoreStorage} from "./storage/CoreStorage.sol";
import {AssetsStorage} from "./storage/AssetsStorage.sol";
import {Token} from "./Token.sol";
import {TokenInfo} from "./types/Common.sol";
import {ChannelID, Channel} from "./v1/Types.sol";
import {IGatewayBase} from "./interfaces/IGatewayBase.sol";
import {IGatewayV1} from "./v1/IGateway.sol";
import {IGatewayV2} from "./v2/IGateway.sol";
import {OperatingMode} from "./types/Common.sol";

library Functions {
    using Address for address;
    using SafeNativeTransfer for address payable;
    using SafeTokenTransferFrom for IERC20;

    error AgentDoesNotExist();
    error InvalidToken();
    error InvalidAmount();
    error ChannelDoesNotExist();

    /// Looks up an agent contract address, failing if no such mapping exists
    function ensureAgent(bytes32 agentID) internal view returns (address agent) {
        agent = CoreStorage.layout().agents[agentID];
        if (agent == address(0)) {
            revert IGatewayBase.AgentDoesNotExist();
        }
    }

    /// @dev Ensure that the specified parachain has a channel allocated
    function ensureChannel(ChannelID channelID) internal view returns (Channel storage ch) {
        ch = CoreStorage.layout().channels[channelID];
        // A channel always has an agent specified.
        if (ch.agent == address(0)) {
            revert IGatewayV1.ChannelDoesNotExist();
        }
    }

    /// @dev Invoke some code within an agent
    function invokeOnAgent(address agent, address executor, bytes memory data)
        internal
        returns (bytes memory)
    {
        (bool success, bytes memory returndata) = (Agent(payable(agent)).invoke(executor, data));
        return Call.verifyResult(success, returndata);
    }

    /// @dev transfer tokens from the sender to the specified agent
    function transferToAgent(address agent, address token, address sender, uint128 amount)
        internal
    {
        if (!token.isContract()) {
            revert InvalidToken();
        }

        if (amount == 0) {
            revert InvalidAmount();
        }

        IERC20(token).safeTransferFrom(sender, agent, amount);
    }

    /// Creates a new agent contract to act as a proxy for the remote location
    /// identified by `origin`.
    function createAgent(bytes32 origin) internal {
        CoreStorage.Layout storage core = CoreStorage.layout();
        address agent = core.agents[origin];
        if (agent == address(0)) {
            agent = address(new Agent(origin));
            core.agents[origin] = agent;
            emit IGatewayBase.AgentCreated(origin, agent);
        } else {
            revert IGatewayBase.AgentAlreadyCreated();
        }
    }

    /// @dev Transfer ether from an agent
    function withdrawEther(
        address executor,
        address agent,
        address payable recipient,
        uint256 amount
    ) internal {
        bytes memory call = abi.encodeCall(AgentExecutor.transferEther, (recipient, amount));
        invokeOnAgent(agent, executor, call);
    }

    // @dev Transfer Ethereum native token back from polkadot
    function withdrawNativeToken(
        address executor,
        address agent,
        address token,
        address recipient,
        uint128 amount
    ) internal {
        bytes memory call = abi.encodeCall(AgentExecutor.transferToken, (token, recipient, amount));
        invokeOnAgent(agent, executor, call);
    }

    function registerNativeToken(address token) internal {
        // NOTE: Explicitly allow a native token to be re-registered. This offers resiliency
        // in case a previous registration attempt of the same token failed on the remote side.
        // It means that registration can be retried.
        AssetsStorage.Layout storage $ = AssetsStorage.layout();
        TokenInfo storage info = $.tokenRegistry[token];

        if (info.isRegistered && info.isForeign()) {
            // Prevent re-registration of foreign tokens as native tokens
            revert IGatewayBase.TokenAlreadyRegistered();
        } else if (!info.isRegistered) {
            info.isRegistered = true;
        }
    }

    /// Creates a new wrapped ERC20 token for a foreign token
    function registerForeignToken(
        bytes32 foreignTokenID,
        string memory name,
        string memory symbol,
        uint8 decimals
    ) internal returns (Token) {
        AssetsStorage.Layout storage $ = AssetsStorage.layout();
        if ($.tokenAddressOf[foreignTokenID] != address(0)) {
            revert IGatewayBase.TokenAlreadyRegistered();
        }
        Token token = new Token(name, symbol, decimals);
        TokenInfo memory info = TokenInfo({isRegistered: true, foreignID: foreignTokenID});

        $.tokenAddressOf[foreignTokenID] = address(token);
        $.tokenRegistry[address(token)] = info;

        emit IGatewayBase.ForeignTokenRegistered(foreignTokenID, address(token));

        return token;
    }

    function mintForeignToken(bytes32 foreignTokenID, address recipient, uint128 amount)
        internal
    {
        address token = _ensureTokenAddressOf(foreignTokenID);
        Token(token).mint(recipient, amount);
    }

    function dustThreshold() internal view returns (uint256) {
        return 21_000 * tx.gasprice;
    }

    // @dev Get token address by tokenID
    function _ensureTokenAddressOf(bytes32 tokenID) internal view returns (address) {
        AssetsStorage.Layout storage $ = AssetsStorage.layout();
        if ($.tokenAddressOf[tokenID] == address(0)) {
            revert IGatewayBase.TokenNotRegistered();
        }
        return $.tokenAddressOf[tokenID];
    }
}

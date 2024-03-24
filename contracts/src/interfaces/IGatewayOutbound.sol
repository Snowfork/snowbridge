// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.23;

import {OperatingMode, InboundMessage, ParaID, ChannelID, MultiAddress} from "../Types.sol";
import {UD60x18} from "prb/math/src/UD60x18.sol";
import {TokenInfo} from "../storage/AssetsStorage.sol";

interface IGatewayOutbound {
    // Emitted when an outbound message has been accepted for delivery to a Polkadot parachain
    event OutboundMessageAccepted(ChannelID indexed channelID, uint64 nonce, bytes32 indexed messageID, bytes payload);

    /// @dev Emitted once the funds are locked and an outbound message is successfully queued.
    event TokenSent(
        address indexed token,
        address indexed sender,
        ParaID indexed destinationChain,
        MultiAddress destinationAddress,
        uint128 amount
    );

    /// @dev Emitted when a command is sent to register a new wrapped token on AssetHub
    event TokenRegistrationSent(address token);

    /// @dev Check whether a token is registered
    function isTokenRegistered(address token) external view returns (bool);

    /// @dev Quote a fee in Ether for registering a token, covering
    /// 1. Delivery costs to BridgeHub
    /// 2. XCM Execution costs on AssetHub
    function quoteRegisterTokenFee() external view returns (uint256);

    /// @dev Register an ERC20 token and create a wrapped derivative on AssetHub in the `ForeignAssets` pallet.
    function registerToken(address token) external payable;

    /// @dev Quote a fee in Ether for sending a token
    /// 1. Delivery costs to BridgeHub
    /// 2. XCM execution costs on destinationChain
    function quoteSendTokenFee(address token, ParaID destinationChain, uint128 destinationFee)
        external
        view
        returns (uint256);

    /// @dev Send ERC20 tokens to parachain `destinationChain` and deposit into account `destinationAddress`
    function sendToken(
        address token,
        ParaID destinationChain,
        MultiAddress calldata destinationAddress,
        uint128 destinationFee,
        uint128 amount
    ) external payable;

    /// @dev Transfer polkadot native tokens back
    function transferToken(
        address token,
        ParaID destinationChain,
        MultiAddress calldata destinationAddress,
        uint128 destinationFee,
        uint128 amount
    ) external payable;

    /// @dev Get tokenInfo by tokenID
    function getTokenInfo(bytes32 tokenID) external view returns (TokenInfo memory);

    /// @dev Emitted once the polkadot native tokens are burnt and an outbound message is successfully queued.
    event TokenTransfered(
        address indexed token,
        address indexed sender,
        ParaID indexed destinationChain,
        MultiAddress destinationAddress,
        uint128 amount
    );
}

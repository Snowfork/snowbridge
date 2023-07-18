// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.20;

import {OperatingMode, InboundMessage, ParaID} from "../Types.sol";
import {Verification} from "../Verification.sol";

interface IGateway {
    // * Events *

    // Emitted when inbound message has been dispatched
    event InboundMessageDispatched(ParaID indexed origin, uint64 nonce, bool success);

    // Emitted when an outbound message has been accepted for delivery to a Polkadot parachain
    event OutboundMessageAccepted(ParaID indexed dest, uint64 nonce, bytes payload);

    // Emitted when an agent has been created for a consensus system on Polkadot
    event AgentCreated(bytes32 agentID, address agent);

    // Emitted when a channel has been created
    event ChannelCreated(ParaID indexed paraID);

    // Emitted when a channel has been updated
    event ChannelUpdated(ParaID indexed paraID);

    // Emitted when the gateway is upgraded
    event Upgraded(address indexed implementation);

    // Emitted when the operating mode is changed
    event OperatingModeChanged(OperatingMode mode);

    // Emitted when funds are withdrawn from an agent
    event AgentFundsWithdrawn(bytes32 indexed agentID, address indexed recipient, uint256 amount);

    // * Getters *

    function operatingMode() external view returns (OperatingMode);
    function channelOperatingModeOf(ParaID paraID) external view returns (OperatingMode);
    function channelFeeRewardOf(ParaID paraID) external view returns (uint256, uint256);
    function channelNoncesOf(ParaID paraID) external view returns (uint64, uint64);
    function agentOf(bytes32 agentID) external view returns (address);

    // Submit an inbound message for dispatch
    function submitInbound(
        InboundMessage calldata message,
        bytes32[] calldata leafProof,
        Verification.Proof calldata headerProof
    ) external;

    // Features for Ethereum users

    // Register a new fungible asset in the `ForeignAssets` pallet on AssetHub. This new asset
    // is a wrapped token correspondig to the ERC20 token `token`.
    //
    // This instruction is idempotent, and will fail if `ForeignAssets`
    function registerNativeToken(address token) external payable;

    // Send ERC20 tokens to Polkadot. The bridged assets will be minted on AssetHub
    // and then reserve transferred to `recipient` on `finalDestPara`.
    function sendNativeToken(address token, bytes calldata recipient, uint128 amount) external payable;
}

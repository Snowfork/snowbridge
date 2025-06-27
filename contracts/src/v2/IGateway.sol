// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.28;

import {OperatingMode, InboundMessage, Payload} from "./Types.sol";
import {Verification} from "../Verification.sol";

interface IGatewayV2 {
    error AgentAlreadyExists();
    error ShouldNotReachHere();
    error InvalidNetwork();
    error InvalidAsset();
    error InsufficientGasLimit();
    error InsufficientValue();
    error ExceededMaximumValue();
    error TooManyAssets();

    /// Return the current operating mode for outbound messaging
    function operatingMode() external view returns (OperatingMode);

    /// Return the address of the agent contract registered for `agentId`.
    function agentOf(bytes32 agentID) external view returns (address);

    /**
     * Events
     */

    /// Emitted when an agent has been created for a consensus system on Polkadot
    event AgentCreated(bytes32 agentID, address agent);

    /// Emitted when inbound message has been dispatched.The "success" field is "true" if all
    //commands successfully executed, otherwise "false" if all or some of the commands failed.
    event InboundMessageDispatched(
        uint64 indexed nonce, bytes32 topic, bool success, bytes32 rewardAddress
    );

    /// Emitted when a command at `index` within an inbound message identified by `nonce` fails to execute
    event CommandFailed(uint64 indexed nonce, uint256 index);

    /// Emitted when an outbound message has been accepted for delivery to a Polkadot parachain
    event OutboundMessageAccepted(uint64 nonce, Payload payload);

    /// @dev Submit a message from Polkadot for verification and dispatch
    /// @param message A message produced by the OutboundQueue pallet on BridgeHub
    /// @param leafProof A message proof used to verify that the message is in the merkle tree
    ///        committed by the OutboundQueue pallet
    /// @param headerProof A proof that the commitment is included in parachain header that was
    ///        finalized by BEEFY.
    /// @param rewardAddress An `AccountId` on BridgeHub that can claim rewards for relaying
    ///        this message, after the relayer has submitted a delivery receipt back to BridgeHub.
    function v2_submit(
        InboundMessage calldata message,
        bytes32[] calldata leafProof,
        Verification.Proof calldata headerProof,
        bytes32 rewardAddress
    ) external;

    // Send an XCM with arbitrary assets to Polkadot Asset Hub
    //
    // Params:
    //   * `xcm` (bytes): SCALE-encoded VersionedXcm message.
    //   * `assets` (bytes[]): Array of asset specs, constrained to maximum of eight.
    //   * `claimer`: SCALE-encoded XCM location of claimer account.
    //   * `executionFee`: Amount of ether to pay for execution on AssetHub.
    //   * `relayerFee`: Amount of ether to pay for relayer incentivation.
    //
    // Supported asset specs:
    // * ERC20: abi.encode(0, tokenAddress, value)
    //
    // Enough ether should be passed to cover `executionFee` and `relayerFee`.
    //
    // When the message is processed on Asset Hub, `assets` and any excess ether will be
    // received into the assets holding register.
    //
    // The `xcm` should contain the necessary instructions to:
    // 1. Pay XCM execution fees for `xcm`, either from assets in holding,
    //    or from the sovereign account of `msg.sender`.
    // 2. Handle the assets in holding, either depositing them into
    //    some account, or forwarding them to another destination.
    //
    function v2_sendMessage(
        bytes calldata xcm,
        bytes[] calldata assets,
        bytes calldata claimer,
        uint128 executionFee,
        uint128 relayerFee
    ) external payable;

    // Register Ethereum-native token on Polkadot.
    //
    // Params:
    //   * `token` (address): address of a token.
    //   * `network` (uint8): Polkadot=0. Kusama may be added later - it is not supported yet.
    //   * `executionFee`: Amount of ether to pay for execution on AssetHub.
    //   * `relayerFee`: Amount of ether to pay for relayer incentivation.
    //   * `gasCost`: The gas cost for a local transfer
    function v2_registerToken(
        address token,
        uint8 network,
        uint128 executionFee,
        uint128 relayerFee,
        uint64 gasCost
    ) external payable;

    /// @dev Creates a new agent contract to act as a proxy for the remote location
    ///      identified by `id`
    function v2_createAgent(bytes32 id) external;

    /// @dev Return the current outbound nonce.
    function v2_outboundNonce() external view returns (uint64);

    /// @dev Check if an inbound message was previously accepted and dispatched
    function v2_isDispatched(uint64 nonce) external view returns (bool);

    /// @dev Check whether a token is registered
    function isTokenRegistered(address token) external view returns (bool);
}

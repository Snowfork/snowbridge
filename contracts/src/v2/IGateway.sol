// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.28;

import {MultiAddress} from "../MultiAddress.sol";
import {OperatingMode, InboundMessage, Payload} from "./Types.sol";
import {Verification} from "../Verification.sol";
import {UD60x18} from "prb/math/src/UD60x18.sol";

interface IGatewayV2 {
    error ShouldNotReachHere();
    error InvalidNetwork();
    error InvalidAsset();
    error InvalidFee();
    error InsufficientValue();
    error ExceededMaximumValue();

    function operatingMode() external view returns (OperatingMode);

    function agentOf(bytes32 agentID) external view returns (address);

    /**
     * Events
     */

    // V2: Emitted when inbound message has been dispatched
    event InboundMessageDispatched(
        uint64 indexed nonce, bool success, bytes32 indexed rewardAddress
    );

    // v2 Emitted when an outbound message has been accepted for delivery to a Polkadot parachain
    event OutboundMessageAccepted(uint64 nonce, Payload payload);

    // V2

    // Submit a message for verification and dispatch
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

    // Register Ethereum-native token on either Polkadot or Kusama
    //
    // Params:
    //   * `token` (address): address of a token.
    //   * `network` (uint8): Polkadot=0; Kusama=1
    //   * `executionFee`: Amount of ether to pay for execution on AssetHub.
    //   * `relayerFee`: Amount of ether to pay for relayer incentivation.
    function v2_registerToken(
        address token,
        uint8 network,
        uint128 executionFee,
        uint128 relayerFee
    ) external payable;

    // Check if an inbound message was previously accepted and dispatched
    function v2_isDispatched(uint64 nonce) external view returns (bool);

    /// @dev Check whether a token is registered
    function isTokenRegistered(address token) external view returns (bool);
}

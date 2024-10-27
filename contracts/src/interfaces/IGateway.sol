// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.25;

import {MultiAddress} from "../MultiAddress.sol";
import {
    OperatingMode,
    InboundMessage as InboundMessageV1,
    ParaID,
    ChannelID
} from "../v1/Types.sol";
import {InboundMessage as InboundMessageV2} from "../v2/Types.sol";
import {Verification} from "../Verification.sol";
import {UD60x18} from "prb/math/src/UD60x18.sol";

interface IGateway {
    error InvalidToken();
    error InvalidAmount();
    error InvalidDestination();
    error TokenNotRegistered();
    error Unsupported();
    error InvalidDestinationFee();
    error AgentDoesNotExist();
    error TokenAlreadyRegistered();
    error TokenMintFailed();
    error TokenTransferFailed();
    error InvalidProof();
    error InvalidNonce();
    error NotEnoughGas();
    error FeePaymentToLow();
    error InvalidFee();
    error Unauthorized();
    error Disabled();
    error AgentAlreadyCreated();
    error ChannelAlreadyCreated();
    error ChannelDoesNotExist();
    error InvalidChannelUpdate();
    error AgentExecutionFailed(bytes returndata);
    error InvalidAgentExecutionPayload();
    error InvalidConstructorParams();
    error AlreadyInitialized();
    error TooManyAssets();

    /**
     * Events
     */

    // V1: Emitted when inbound message has been dispatched
    event InboundMessageDispatched(
        ChannelID indexed channelID,
        uint64 nonce,
        bytes32 indexed messageID,
        bool success
    );

    // V2: Emitted when inbound message has been dispatched
    event InboundMessageDispatched(
        uint64 indexed nonce, bool success, bytes32 indexed rewardAddress
    );

    // Emitted when an outbound message has been accepted for delivery to a Polkadot parachain
    event OutboundMessageAccepted(
        ChannelID indexed channelID,
        uint64 nonce,
        bytes32 indexed messageID,
        bytes payload
    );

    // v2 Emitted when an outbound message has been accepted for delivery to a Polkadot parachain
    event OutboundMessageAccepted(uint64 nonce, uint256 reward, bytes payload);

    // Emitted when an agent has been created for a consensus system on Polkadot
    event AgentCreated(bytes32 agentID, address agent);

    // Emitted when a channel has been created
    event ChannelCreated(ChannelID indexed channelID);

    // Emitted when a channel has been updated
    event ChannelUpdated(ChannelID indexed channelID);

    // Emitted when the operating mode is changed
    event OperatingModeChanged(OperatingMode mode);

    // Emitted when pricing params updated
    event PricingParametersChanged();

    // Emitted when funds are withdrawn from an agent
    event AgentFundsWithdrawn(
        bytes32 indexed agentID, address indexed recipient, uint256 amount
    );

    // Emitted when foreign token from polkadot registed
    event ForeignTokenRegistered(bytes32 indexed tokenID, address token);

    /**
     * Getters
     */
    function operatingMode() external view returns (OperatingMode);

    function channelOperatingModeOf(ChannelID channelID)
        external
        view
        returns (OperatingMode);

    function channelNoncesOf(ChannelID channelID)
        external
        view
        returns (uint64, uint64);

    function agentOf(bytes32 agentID) external view returns (address);

    function pricingParameters() external view returns (UD60x18, uint128);

    function implementation() external view returns (address);

    /**
     * Messaging
     */

    // Submit a message from a Polkadot network
    function submitV1(
        InboundMessageV1 calldata message,
        bytes32[] calldata leafProof,
        Verification.Proof calldata headerProof
    ) external;

    /**
     * Token Transfers
     */

    // @dev Emitted when the fees updated
    event TokenTransferFeesChanged();

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
    function quoteSendTokenFee(
        address token,
        ParaID destinationChain,
        uint128 destinationFee
    ) external view returns (uint256);

    /// @dev Send ERC20 tokens to parachain `destinationChain` and deposit into account `destinationAddress`
    function sendToken(
        address token,
        ParaID destinationChain,
        MultiAddress calldata destinationAddress,
        uint128 destinationFee,
        uint128 amount
    ) external payable;

    // V2

    // Submit a message for verification and dispatch
    function v2_submit(
        InboundMessageV2 calldata message,
        bytes32[] calldata leafProof,
        Verification.Proof calldata headerProof,
        bytes32 rewardAddress
    ) external;

    // Send an XCM with arbitrary assets to Polkadot Asset Hub
    //
    // Params:
    //   * `xcm` (bytes): SCALE-encoded VersionedXcm message
    //   * `assets` (bytes[]): Array of asset specs, constrained to maximum of eight.
    //
    // Supported asset specs:
    // * ERC20: abi.encode(0, tokenAddress, value)
    //
    // On Asset Hub, the assets will be received into the assets holding register.
    //
    // The `xcm` should contain the necessary instructions to:
    // 1. Pay XCM execution fees for `xcm`, either from assets in holding,
    //    or from the sovereign account of `msg.sender`.
    // 2. Handle the assets in holding, either depositing them into
    //    some account, or forwarding them to another destination.
    //
    // To incentivize message delivery, some amount of ether must be passed and should
    // at least cover the total cost of delivery to Polkadot. This ether be sent across
    // the bridge as WETH, and given to the relayer as compensation and incentivization.
    //
    function v2_sendMessage(
        bytes calldata xcm,
        bytes[] calldata assets,
        bytes calldata claimer
    ) external payable;

    // Register Ethereum-native token on AHP, using `xcmFeeAHP` of `msg.value`
    // to pay for execution on AHP.
    function v2_registerToken(address token, uint128 xcmFeeAHP) external payable;

    // Register Ethereum-native token on AHK, using `xcmFeeAHP` and `xcmFeeAHK`
    // of `msg.value` to pay for execution on AHP and AHK respectively.
    function v2_registerTokenOnKusama(
        address token,
        uint128 xcmFeeAHP,
        uint128 xcmFeeAHK
    ) external payable;

    // Check if an inbound message was previously accepted and dispatched
    function v2_isDispatched(uint64 nonce) external returns (bool);
}

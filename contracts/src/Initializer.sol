// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.25;

import {MerkleProof} from "openzeppelin/utils/cryptography/MerkleProof.sol";
import {Verification} from "./Verification.sol";

import {AssetsV1} from "./AssetsV1.sol";
import {AssetsV2} from "./AssetsV2.sol";

import {AgentExecutor} from "./AgentExecutor.sol";
import {Agent} from "./Agent.sol";
import {
    OperatingMode,
    ParaID,
    TokenInfo,
    MultiAddress,
    Channel,
    ChannelID,
    InboundMessageV1,
    CommandV1,
    AgentExecuteCommandV1,
    TicketV1,
    Costs,
    AgentExecuteParamsV1,
    UpgradeParamsV1,
    CreateAgentParamsV1,
    CreateChannelParamsV1,
    UpdateChannelParamsV1,
    SetOperatingModeParamsV1,
    TransferNativeFromAgentParamsV1,
    SetTokenTransferFeesParamsV1,
    SetPricingParametersParamsV1,
    RegisterForeignTokenParamsV1,
    MintForeignTokenParamsV1,
    TransferNativeTokenParamsV1,
    InboundMessageV2,
    CommandV2,
    CommandKindV2,
    TicketV2,
    UpgradeParamsV2,
    SetOperatingModeParamsV2,
    UnlockNativeTokenParamsV2,
    MintForeignTokenParamsV2,
    CallContractParamsV2
} from "./Types.sol";
import {Upgrade} from "./Upgrade.sol";
import {IGateway} from "./interfaces/IGateway.sol";
import {IInitializable} from "./interfaces/IInitializable.sol";
import {IUpgradable} from "./interfaces/IUpgradable.sol";
import {ERC1967} from "./utils/ERC1967.sol";
import {Address} from "./utils/Address.sol";
import {SafeNativeTransfer} from "./utils/SafeTransfer.sol";
import {Call} from "./utils/Call.sol";
import {Math} from "./utils/Math.sol";
import {ScaleCodec} from "./utils/ScaleCodec.sol";

import {CoreStorage} from "./storage/CoreStorage.sol";
import {PricingStorage} from "./storage/PricingStorage.sol";
import {AssetsStorage} from "./storage/AssetsStorage.sol";
import {OperatorStorage} from "./storage/OperatorStorage.sol";

import {UD60x18, ud60x18, convert} from "prb/math/src/UD60x18.sol";

library Initializer {
    error Unauthorized();

    ParaID constant ASSET_HUB_PARA_ID = ParaID.wrap(1000);
    bytes32 constant ASSET_HUB_AGENT_ID =
        0x81c5ab2571199e3188135178f3c2c8e2d268be1313d029b30f534fa579b69b79;

    ParaID constant BRIDGE_HUB_PARA_ID = ParaID.wrap(1002);
    bytes32 constant BRIDGE_HUB_AGENT_ID =
        0x03170a2e7597b7b7e3d84c05391d139a62b157e78786d8c082f29dcf4c111314;

    // ChannelIDs
    ChannelID internal constant PRIMARY_GOVERNANCE_CHANNEL_ID =
        ChannelID.wrap(bytes32(uint256(1)));
    ChannelID internal constant SECONDARY_GOVERNANCE_CHANNEL_ID =
        ChannelID.wrap(bytes32(uint256(2)));

    // Initial configuration for bridge
    struct Config {
        OperatingMode mode;
        /// @dev The fee charged to users for submitting outbound messages (DOT)
        uint128 deliveryCost;
        /// @dev The ETH/DOT exchange rate
        UD60x18 exchangeRate;
        ParaID assetHubParaID;
        bytes32 assetHubAgentID;
        /// @dev The extra fee charged for registering tokens (DOT)
        uint128 assetHubCreateAssetFee;
        /// @dev The extra fee charged for sending tokens (DOT)
        uint128 assetHubReserveTransferFee;
        /// @dev extra fee to discourage spamming
        uint256 registerTokenFee;
        /// @dev Fee multiplier
        UD60x18 multiplier;
        /// @dev Optional rescueOperator
        address rescueOperator;
    }

    function initialize(bytes calldata data) external {
        // Prevent initialization of storage in implementation contract
        if (ERC1967.load() == address(0)) {
            revert Unauthorized();
        }

        CoreStorage.Layout storage core = CoreStorage.layout();

        Config memory config = abi.decode(data, (Config));

        core.mode = config.mode;

        // Initialize agent for BridgeHub
        address bridgeHubAgent = address(new Agent(BRIDGE_HUB_AGENT_ID));
        core.agents[BRIDGE_HUB_AGENT_ID] = bridgeHubAgent;
        core.agentAddresses[bridgeHubAgent] = BRIDGE_HUB_AGENT_ID;

        // Initialize channel for primary governance track
        core.channels[PRIMARY_GOVERNANCE_CHANNEL_ID] = Channel({
            mode: OperatingMode.Normal,
            agent: bridgeHubAgent,
            inboundNonce: 0,
            outboundNonce: 0
        });

        // Initialize channel for secondary governance track
        core.channels[SECONDARY_GOVERNANCE_CHANNEL_ID] = Channel({
            mode: OperatingMode.Normal,
            agent: bridgeHubAgent,
            inboundNonce: 0,
            outboundNonce: 0
        });

        // Initialize agent for for AssetHub
        address assetHubAgent = address(new Agent(config.assetHubAgentID));
        core.agents[config.assetHubAgentID] = assetHubAgent;
        core.agentAddresses[assetHubAgent] = config.assetHubAgentID;

        // Initialize channel for AssetHub
        core.channels[config.assetHubParaID.into()] = Channel({
            mode: OperatingMode.Normal,
            agent: assetHubAgent,
            inboundNonce: 0,
            outboundNonce: 0
        });

        // Initialize pricing storage
        PricingStorage.Layout storage pricing = PricingStorage.layout();
        pricing.exchangeRate = config.exchangeRate;
        pricing.deliveryCost = config.deliveryCost;
        pricing.multiplier = config.multiplier;

        // Initialize assets storage
        AssetsStorage.Layout storage assets = AssetsStorage.layout();

        assets.assetHubParaID = config.assetHubParaID;
        assets.assetHubAgent = assetHubAgent;
        assets.registerTokenFee = config.registerTokenFee;
        assets.assetHubCreateAssetFee = config.assetHubCreateAssetFee;
        assets.assetHubReserveTransferFee = config.assetHubReserveTransferFee;

        // Initialize operator storage
        OperatorStorage.Layout storage operatorStorage = OperatorStorage.layout();
        operatorStorage.operator = config.rescueOperator;
    }
}

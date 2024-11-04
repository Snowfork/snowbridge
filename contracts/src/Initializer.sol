// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.25;

import {MerkleProof} from "openzeppelin/utils/cryptography/MerkleProof.sol";
import {Verification} from "./Verification.sol";

import {AgentExecutor} from "./AgentExecutor.sol";
import {Agent} from "./Agent.sol";
import {
    OperatingMode,
    ParaID,
    TokenInfo,
    MultiAddress,
    Channel,
    ChannelID
} from "./Types.sol";
import {Upgrade} from "./Upgrade.sol";
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

import {Constants} from "./Constants.sol";

import {UD60x18, ud60x18, convert} from "prb/math/src/UD60x18.sol";

library Initializer {
    error Unauthorized();

    // Initial configuration for bridge
    struct Config {
        OperatingMode mode;
        /// @dev The fee charged to users for submitting outbound messages (DOT)
        uint128 deliveryCost;
        /// @dev The ETH/DOT exchange rate
        UD60x18 exchangeRate;
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
        uint8 foreignTokenDecimals;
        uint128 maxDestinationFee;
        address weth;
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
        address bridgeHubAgent = address(new Agent(Constants.BRIDGE_HUB_AGENT_ID));
        core.agents[Constants.BRIDGE_HUB_AGENT_ID] = bridgeHubAgent;

        // Initialize channel for primary governance track
        core.channels[Constants.PRIMARY_GOVERNANCE_CHANNEL_ID] = Channel({
            mode: OperatingMode.Normal,
            agent: bridgeHubAgent,
            inboundNonce: 0,
            outboundNonce: 0
        });

        // Initialize channel for secondary governance track
        core.channels[Constants.SECONDARY_GOVERNANCE_CHANNEL_ID] = Channel({
            mode: OperatingMode.Normal,
            agent: bridgeHubAgent,
            inboundNonce: 0,
            outboundNonce: 0
        });

        // Initialize agent for for AssetHub
        address assetHubAgent = address(new Agent(Constants.ASSET_HUB_AGENT_ID));
        core.agents[Constants.ASSET_HUB_AGENT_ID] = assetHubAgent;

        // Initialize channel for AssetHub
        core.channels[Constants.ASSET_HUB_PARA_ID.into()] = Channel({
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

        assets.assetHubParaID = Constants.ASSET_HUB_PARA_ID;
        assets.assetHubAgent = assetHubAgent;
        assets.registerTokenFee = config.registerTokenFee;
        assets.assetHubCreateAssetFee = config.assetHubCreateAssetFee;
        assets.assetHubReserveTransferFee = config.assetHubReserveTransferFee;
        assets.foreignTokenDecimals = config.foreignTokenDecimals;
        assets.maxDestinationFee = config.maxDestinationFee;
        assets.weth = config.weth;

        // Initialize operator storage
        OperatorStorage.Layout storage operatorStorage = OperatorStorage.layout();
        operatorStorage.operator = config.rescueOperator;
    }
}

// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.28;

import {IERC20} from "../interfaces/IERC20.sol";

import {SafeTokenTransferFrom} from "../utils/SafeTransfer.sol";

import {AssetsStorage, TokenInfo} from "../storage/AssetsStorage.sol";
import {CoreStorage} from "../storage/CoreStorage.sol";
import {PricingStorage} from "../storage/PricingStorage.sol";
import {SubstrateTypes} from "../SubstrateTypes.sol";
import {MultiAddress} from "../types/Common.sol";
import {Address} from "../utils/Address.sol";
import {AgentExecutor} from "../AgentExecutor.sol";
import {Agent} from "../Agent.sol";
import {Call} from "../utils/Call.sol";
import {Token} from "../Token.sol";
import {Upgrade} from "../Upgrade.sol";
import {Functions} from "../Functions.sol";
import {IGatewayBase} from "../interfaces/IGatewayBase.sol";
import {IGatewayV1} from "./IGateway.sol";

import {
    ParaID,
    Ticket,
    Costs,
    AgentExecuteCommand,
    AgentExecuteParams,
    CreateAgentParams,
    CreateChannelParams,
    UpdateChannelParams,
    UpgradeParams,
    SetOperatingModeParams,
    TransferNativeFromAgentParams,
    SetTokenTransferFeesParams,
    SetPricingParametersParams,
    UnlockNativeTokenParams,
    RegisterForeignTokenParams,
    MintForeignTokenParams
} from "./Types.sol";

library HandlersV1 {
    using Address for address;
    using SafeTokenTransferFrom for IERC20;

    function agentExecute(address executor, bytes calldata data) external {
        AgentExecuteParams memory params = abi.decode(data, (AgentExecuteParams));

        address agent = Functions.ensureAgent(params.agentID);

        if (params.payload.length == 0) {
            revert IGatewayBase.InvalidAgentExecutionPayload();
        }

        (AgentExecuteCommand command, bytes memory commandParams) =
            abi.decode(params.payload, (AgentExecuteCommand, bytes));
        if (command == AgentExecuteCommand.TransferToken) {
            (address token, address recipient, uint128 amount) =
                abi.decode(commandParams, (address, address, uint128));
            Functions.withdrawNativeToken(executor, agent, token, recipient, amount);
        }
    }

    /// @dev Create an agent for a consensus system on Polkadot
    function createAgent(bytes calldata data) external {
        CreateAgentParams memory params = abi.decode(data, (CreateAgentParams));
        Functions.createAgent(params.agentID);
    }

    /// @dev Perform an upgrade of the gateway
    function upgrade(bytes calldata data) external {
        UpgradeParams memory params = abi.decode(data, (UpgradeParams));
        Upgrade.upgrade(params.impl, params.implCodeHash, params.initParams);
    }

    // @dev Set the operating mode of the gateway
    function setOperatingMode(bytes calldata data) external {
        CoreStorage.Layout storage $ = CoreStorage.layout();
        SetOperatingModeParams memory params = abi.decode(data, (SetOperatingModeParams));
        $.mode = params.mode;
        emit IGatewayBase.OperatingModeChanged(params.mode);
    }

    // @dev Transfer funds from an agent to a recipient account
    function transferNativeFromAgent(address executor, bytes calldata data) external {
        TransferNativeFromAgentParams memory params =
            abi.decode(data, (TransferNativeFromAgentParams));

        address agent = Functions.ensureAgent(params.agentID);

        Functions.withdrawEther(executor, agent, payable(params.recipient), params.amount);
        emit IGatewayV1.AgentFundsWithdrawn(params.agentID, params.recipient, params.amount);
    }

    // @dev Set token fees of the gateway
    function setTokenTransferFees(bytes calldata data) external {
        AssetsStorage.Layout storage $ = AssetsStorage.layout();
        SetTokenTransferFeesParams memory params = abi.decode(data, (SetTokenTransferFeesParams));
        $.assetHubCreateAssetFee = params.assetHubCreateAssetFee;
        $.assetHubReserveTransferFee = params.assetHubReserveTransferFee;
        $.registerTokenFee = params.registerTokenFee;
        emit IGatewayV1.TokenTransferFeesChanged();
    }

    // @dev Set pricing params of the gateway
    function setPricingParameters(bytes calldata data) external {
        PricingStorage.Layout storage pricing = PricingStorage.layout();
        SetPricingParametersParams memory params = abi.decode(data, (SetPricingParametersParams));
        pricing.exchangeRate = params.exchangeRate;
        pricing.deliveryCost = params.deliveryCost;
        pricing.multiplier = params.multiplier;
        emit IGatewayV1.PricingParametersChanged();
    }

    // @dev Register a new fungible Polkadot token for an agent
    function registerForeignToken(bytes calldata data) external {
        RegisterForeignTokenParams memory params = abi.decode(data, (RegisterForeignTokenParams));
        Functions.registerForeignToken(
            params.foreignTokenID, params.name, params.symbol, params.decimals
        );
    }

    // @dev Transfer Ethereum native token back from polkadot
    function unlockNativeToken(address executor, bytes calldata data) external {
        UnlockNativeTokenParams memory params = abi.decode(data, (UnlockNativeTokenParams));
        address agent = Functions.ensureAgent(params.agentID);
        Functions.withdrawNativeToken(
            executor, agent, params.token, params.recipient, params.amount
        );
    }

    // @dev Mint foreign token from polkadot
    function mintForeignToken(bytes calldata data) external {
        MintForeignTokenParams memory params = abi.decode(data, (MintForeignTokenParams));
        Functions.mintForeignToken(params.foreignTokenID, params.recipient, params.amount);
    }
}

//SPDX-License-Identifier: GPL-3.0-or-later

// Copyright (C) Moondance Labs Ltd.
// This file is part of Tanssi.
// Tanssi is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
// Tanssi is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
// You should have received a copy of the GNU General Public License
// along with Tanssi.  If not, see <http://www.gnu.org/licenses/>
pragma solidity 0.8.25;

import {console2, Script} from "forge-std/Script.sol";
import {UD60x18, ud60x18} from "prb/math/src/UD60x18.sol";
import {ParaID} from "../src/Types.sol";

contract HelperConfig is Script {
    BeefyClientConfig public activeBeefyClientConfig;
    GatewayConfig public activeGatewayConfig;
    GatewayInitConfig public activeGatewayInitConfig;

    struct BeefyClientConfig {
        uint256 randaoCommitDelay;
        uint256 randaoCommitExpiration;
        uint256 minimumSignatures;
    }

    struct GatewayConfig {
        address beefyClient;
        address agentExecutor;
        ParaID bridgeHubParaID;
        bytes32 bridgeHubAgentID;
        uint8 foreignTokenDecimals;
        uint128 maxDestinationFee;
    }

    struct GatewayInitConfig {
        ParaID assetHubParaID;
        bytes32 assetHubAgentID;
        bool rejectOutboundMessages;
        uint128 deliveryCost;
        uint128 registerTokenFee;
        uint128 assetHubCreateAssetFee;
        uint128 assetHubReserveTransferFee;
        UD60x18 exchangeRate;
        UD60x18 multiplier;
    }

    uint256 public DEFAULT_ANVIL_PRIVATE_KEY = 0x2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6;

    constructor(string memory testnet) {
        (activeBeefyClientConfig, activeGatewayConfig, activeGatewayInitConfig) = getChainConfig(testnet);
    }

    function getBeefyClientConfig() public view returns (BeefyClientConfig memory beefyClientConfig) {
        return activeBeefyClientConfig;
    }

    function getGatewayConfig() public view returns (GatewayConfig memory gatewayConfig) {
        return activeGatewayConfig;
    }

    function getGatewayInitConfig() public view returns (GatewayInitConfig memory gatewayInitConfig) {
        return activeGatewayInitConfig;
    }

    function getChainConfig(string memory testnet)
        public
        view
        returns (
            BeefyClientConfig memory beefyClientConfig,
            GatewayConfig memory gatewayConfig,
            GatewayInitConfig memory gatewayInitConfig
        )
    {
        string memory root = vm.projectRoot();
        string memory path = string.concat(root, "/scripts/chain_data.json");
        string memory json = vm.readFile(path);

        //! Make sure chainid is present in the json or this will just revert without giving any information
        uint256 chainId = block.chainid;
        string memory jsonPath = string.concat("$.", vm.toString(chainId));
        if (
            (
                keccak256(abi.encodePacked(testnet)) == keccak256("stagelight")
                    || keccak256(abi.encodePacked(testnet)) == keccak256("dancelight")
            ) && block.chainid == 11155111
        ) {
            jsonPath = string.concat(jsonPath, string.concat(".", testnet));
        }

        beefyClientConfig = _loadBeefyClientConfig(json, jsonPath);
        gatewayConfig = _loadGatewayConfig(json, jsonPath);
        gatewayInitConfig = _loadGatewayInitConfig(json, jsonPath);
    }

    function _loadBeefyClientConfig(string memory json, string memory jsonPath)
        private
        pure
        returns (BeefyClientConfig memory beefyClientConfig)
    {
        beefyClientConfig = BeefyClientConfig({
            randaoCommitDelay: abi.decode(vm.parseJson(json, string.concat(jsonPath, ".randomCommitDelay")), (uint256)),
            randaoCommitExpiration: abi.decode(
                vm.parseJson(json, string.concat(jsonPath, ".randaoCommitExpiration")), (uint256)
            ),
            minimumSignatures: abi.decode(vm.parseJson(json, string.concat(jsonPath, ".minimumSignatures")), (uint256))
        });
    }

    function _loadGatewayConfig(string memory json, string memory jsonPath)
        private
        pure
        returns (GatewayConfig memory gatewayConfig)
    {
        gatewayConfig = GatewayConfig({
            beefyClient: abi.decode(vm.parseJson(json, string.concat(jsonPath, ".beefyClient")), (address)),
            agentExecutor: abi.decode(vm.parseJson(json, string.concat(jsonPath, ".agentExecutor")), (address)),
            bridgeHubParaID: abi.decode(vm.parseJson(json, string.concat(jsonPath, ".bridgeHubParaID")), (ParaID)),
            bridgeHubAgentID: abi.decode(vm.parseJson(json, string.concat(jsonPath, ".bridgeHubAgentID")), (bytes32)),
            foreignTokenDecimals: abi.decode(vm.parseJson(json, string.concat(jsonPath, ".foreignTokenDecimals")), (uint8)),
            maxDestinationFee: abi.decode(vm.parseJson(json, string.concat(jsonPath, ".maxDestinationFee")), (uint128))
        });
    }

    function _loadGatewayInitConfig(string memory json, string memory jsonPath)
        private
        pure
        returns (GatewayInitConfig memory gatewayInitConfig)
    {
        gatewayInitConfig = GatewayInitConfig({
            assetHubParaID: abi.decode(vm.parseJson(json, string.concat(jsonPath, ".assetHubParaID")), (ParaID)),
            assetHubAgentID: abi.decode(vm.parseJson(json, string.concat(jsonPath, ".assetHubAgentID")), (bytes32)),
            rejectOutboundMessages: abi.decode(
                vm.parseJson(json, string.concat(jsonPath, ".rejectOutboundMessages")), (bool)
            ),
            deliveryCost: abi.decode(vm.parseJson(json, string.concat(jsonPath, ".deliveryCost")), (uint128)),
            registerTokenFee: abi.decode(vm.parseJson(json, string.concat(jsonPath, ".registerTokenFee")), (uint128)),
            assetHubCreateAssetFee: abi.decode(
                vm.parseJson(json, string.concat(jsonPath, ".assetHubCreateAssetFee")), (uint128)
            ),
            assetHubReserveTransferFee: abi.decode(
                vm.parseJson(json, string.concat(jsonPath, ".assetHubReserveTransferFee")), (uint128)
            ),
            exchangeRate: abi.decode(vm.parseJson(json, string.concat(jsonPath, ".exchangeRate")), (UD60x18)),
            multiplier: abi.decode(vm.parseJson(json, string.concat(jsonPath, ".multiplier")), (UD60x18))
        });
    }
}

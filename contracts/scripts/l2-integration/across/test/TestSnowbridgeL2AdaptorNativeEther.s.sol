// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.33;

import {Script, console} from "forge-std/Script.sol";
import {IERC20} from "openzeppelin/token/ERC20/IERC20.sol";
import {WETH9} from "canonical-weth/WETH9.sol";

import {SnowbridgeL2Adaptor} from "../../../../src/l2-integration/SnowbridgeL2Adaptor.sol";
import {ISpokePool} from "../../../../src/l2-integration/interfaces/ISpokePool.sol";
import {DepositParams, SendParams, SwapParams} from "../../../../src/l2-integration/Types.sol";
import {
    CHAIN_ID as SEPOLIA_CHAIN_ID,
    BASE_CHAIN_ID as SEPOLIA_BASE_CHAIN_ID,
    WETH9 as SEPOLIA_WETH9,
    BASE_WETH9 as SEPOLIA_BASE_WETH9,
    TIME_BUFFER as SEPOLIA_TIME_BUFFER
} from "../constants/Sepolia.sol";
import {
    CHAIN_ID as MAINNET_CHAIN_ID,
    BASE_CHAIN_ID as MAINNET_BASE_CHAIN_ID,
    WETH9 as MAINNET_WETH9,
    BASE_WETH9 as MAINNET_BASE_WETH9,
    TIME_BUFFER as MAINNET_TIME_BUFFER,
    ARBITRUM_CHAIN_ID as MAINNET_ARBITRUM_CHAIN_ID,
    ARBITRUM_WETH9 as MAINNET_ARBITRUM_WETH9
} from "../constants/Mainnet.sol";

contract TestSnowbridgeL2AdaptorNativeEther is Script {
    function run() public {
        vm.startBroadcast();
        address recipient = vm.envAddress("RECIPIENT_ADDRESS");
        address payable l2SnowbridgeAdaptor;

        DepositParams memory params;
        SendParams memory sendParams;
        bytes[] memory assets = new bytes[](0);

        if (
            keccak256(bytes(vm.envString("L1_NETWORK"))) == keccak256(bytes("mainnet"))
                && keccak256(bytes(vm.envString("L2_NETWORK"))) == keccak256(bytes("base"))
        ) {
            l2SnowbridgeAdaptor = payable(vm.envAddress("L2_BASE_SNOWBRIDGE_ADAPTOR_ADDRESS"));
            // Mainnet configuration
            params = DepositParams({
                inputToken: address(0),
                outputToken: address(0),
                inputAmount: 1_200_000_000_000_000, // 0.0012 ETH
                outputAmount: 1_000_000_000_000_000, // 0.001 ETH
                destinationChainId: MAINNET_CHAIN_ID,
                fillDeadlineBuffer: MAINNET_TIME_BUFFER
            });

            // tx from https://etherscan.io/tx/0x57d799b6e564c8db30fa91e5d311528814a6a29a22eee3c279e15d73778b1892
            sendParams = SendParams({
                xcm: hex"050c140d0102080001010054d82b42bcd22b175d71d62ef2114defcf14344c4b88acf0eb4356737d7fdb4a2c881a4e2c885398241abf54c551d4308c60bb0e9f2f860c26d1ec9528fb30a5fd",
                assets: assets,
                claimer: hex"0001010054d82b42bcd22b175d71d62ef2114defcf14344c4b88acf0eb4356737d7fdb4a",
                executionFee: 5_688_771_233_667,
                relayerFee: 50_035_501_219_494
            });
        } else if (
            keccak256(bytes(vm.envString("L1_NETWORK"))) == keccak256(bytes("mainnet"))
                && keccak256(bytes(vm.envString("L2_NETWORK"))) == keccak256(bytes("arbitrum"))
        ) {
            l2SnowbridgeAdaptor = payable(vm.envAddress("L2_ARBITRUM_SNOWBRIDGE_ADAPTOR_ADDRESS"));
            // Mainnet configuration
            params = DepositParams({
                inputToken: address(0),
                outputToken: address(0),
                inputAmount: 1_200_000_000_000_000, // 0.0012 ETH
                outputAmount: 1_000_000_000_000_000, // 0.001 ETH
                destinationChainId: MAINNET_CHAIN_ID,
                fillDeadlineBuffer: MAINNET_TIME_BUFFER
            });

            // tx from https://etherscan.io/tx/0x57d799b6e564c8db30fa91e5d311528814a6a29a22eee3c279e15d73778b1892
            sendParams = SendParams({
                xcm: hex"050c140d0102080001010054d82b42bcd22b175d71d62ef2114defcf14344c4b88acf0eb4356737d7fdb4a2c881a4e2c885398241abf54c551d4308c60bb0e9f2f860c26d1ec9528fb30a5fd",
                assets: assets,
                claimer: hex"0001010054d82b42bcd22b175d71d62ef2114defcf14344c4b88acf0eb4356737d7fdb4a",
                executionFee: 5_688_771_233_667,
                relayerFee: 50_035_501_219_494
            });
        } else if (
            keccak256(bytes(vm.envString("L1_NETWORK"))) == keccak256(bytes("sepolia"))
                && keccak256(bytes(vm.envString("L2_NETWORK"))) == keccak256(bytes("base-sepolia"))
        ) {
            // Sepolia configuration
            l2SnowbridgeAdaptor = payable(vm.envAddress("L2_BASE_SNOWBRIDGE_ADAPTOR_ADDRESS"));
            params = DepositParams({
                inputToken: address(0),
                outputToken: address(0),
                inputAmount: 12_000_000_000_000_000, // 0.012 ETH
                outputAmount: 10_000_000_000_000_000, // 0.01 ETH
                destinationChainId: SEPOLIA_CHAIN_ID,
                fillDeadlineBuffer: SEPOLIA_TIME_BUFFER
            });

            // tx from https://sepolia.etherscan.io/tx/0x7e1668a805d24e0e51a04a51f6d6dc0a4b87dfe85f04eb76328c206700567d2b
            sendParams = SendParams({
                xcm: hex"050c140d010208000101005827013ddc4082f8252f8729bd2f06e77e7863dea9202a6f0e7a2c34e356e85a2c964edfa9919080fefce42be38a07df8d7586c641f9f88a75b27c1e0d6001fa34",
                assets: assets,
                claimer: hex"000101005827013ddc4082f8252f8729bd2f06e77e7863dea9202a6f0e7a2c34e356e85a",
                executionFee: 33_346_219_347_761,
                relayerFee: 553_808_951_460_256
            });
        } else if (
            keccak256(bytes(vm.envString("L1_NETWORK"))) == keccak256(bytes("sepolia"))
                && keccak256(bytes(vm.envString("L2_NETWORK")))
                    == keccak256(bytes("arbitrum-sepolia"))
        ) {
            // Sepolia configuration
            l2SnowbridgeAdaptor = payable(vm.envAddress("L2_ARBITRUM_SNOWBRIDGE_ADAPTOR_ADDRESS"));
            params = DepositParams({
                inputToken: address(0),
                outputToken: address(0),
                inputAmount: 12_000_000_000_000_000, // 0.012 ETH
                outputAmount: 10_000_000_000_000_000, // 0.01 ETH
                destinationChainId: SEPOLIA_CHAIN_ID,
                fillDeadlineBuffer: SEPOLIA_TIME_BUFFER
            });

            // tx from https://sepolia.etherscan.io/tx/0x7e1668a805d24e0e51a04a51f6d6dc0a4b87dfe85f04eb76328c206700567d2b
            sendParams = SendParams({
                xcm: hex"050c140d010208000101005827013ddc4082f8252f8729bd2f06e77e7863dea9202a6f0e7a2c34e356e85a2c964edfa9919080fefce42be38a07df8d7586c641f9f88a75b27c1e0d6001fa34",
                assets: assets,
                claimer: hex"000101005827013ddc4082f8252f8729bd2f06e77e7863dea9202a6f0e7a2c34e356e85a",
                executionFee: 33_346_219_347_761,
                relayerFee: 553_808_951_460_256
            });
        } else {
            revert("Unsupported L2 network");
        }

        SnowbridgeL2Adaptor(l2SnowbridgeAdaptor)
        .sendEtherAndCall{
            value: params.inputAmount
        }(params, sendParams, recipient, keccak256("TestNativeEtherL2AdaptorTopicId"));
    }
}

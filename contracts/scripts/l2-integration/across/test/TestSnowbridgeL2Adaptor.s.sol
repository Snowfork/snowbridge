// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {Script, console} from "forge-std/Script.sol";
import {IERC20} from "openzeppelin/token/ERC20/IERC20.sol";
import {WETH9} from "canonical-weth/WETH9.sol";

import {SnowbridgeL2Adaptor} from "../../../../src/l2-integration/SnowbridgeL2Adaptor.sol";
import {ISpokePool} from "../../../../src/l2-integration/interfaces/ISpokePool.sol";
import {DepositParams, SendParams, SwapParams} from "../../../../src/l2-integration/Types.sol";

import {
    USDC as SEPOLIA_USDC,
    BASE_USDC as SEPOLIA_BASE_USDC,
    CHAIN_ID as SEPOLIA_CHAIN_ID,
    BASE_CHAIN_ID as SEPOLIA_BASE_CHAIN_ID,
    BASE_WETH9 as SEPOLIA_BASE_WETH9,
    TIME_BUFFER as SEPOLIA_TIME_BUFFER
} from "../constants/Sepolia.sol";

import {
    USDC as MAINNET_USDC,
    BASE_USDC as MAINNET_BASE_USDC,
    CHAIN_ID as MAINNET_CHAIN_ID,
    BASE_CHAIN_ID as MAINNET_BASE_CHAIN_ID,
    BASE_WETH9 as MAINNET_BASE_WETH9,
    TIME_BUFFER as MAINNET_TIME_BUFFER
} from "../constants/Mainnet.sol";

contract TestSnowbridgeL2Adaptor is Script {
    function run() public {
        vm.startBroadcast();

        address payable l2SnowbridgeAdaptor =
            payable(vm.envAddress("L2_SNOWBRIDGE_ADAPTOR_ADDRESS"));
        address recipient = vm.envAddress("RECIPIENT_ADDRESS");

        address BASE_USDC_ADDRESS;
        address USDC_ADDRESS;
        uint256 CHAIN_ID;
        uint32 TIME_BUFFER;
        if (keccak256(bytes(vm.envString("L1_NETWORK"))) == keccak256(bytes("mainnet"))) {
            BASE_USDC_ADDRESS = MAINNET_BASE_USDC;
            USDC_ADDRESS = MAINNET_USDC;
            CHAIN_ID = MAINNET_CHAIN_ID;
            TIME_BUFFER = MAINNET_TIME_BUFFER;
        } else if (keccak256(bytes(vm.envString("L1_NETWORK"))) == keccak256(bytes("sepolia"))) {
            BASE_USDC_ADDRESS = SEPOLIA_BASE_USDC;
            USDC_ADDRESS = SEPOLIA_USDC;
            CHAIN_ID = SEPOLIA_CHAIN_ID;
            TIME_BUFFER = SEPOLIA_TIME_BUFFER;
        } else {
            revert("Unsupported L1 network");
        }

        DepositParams memory params = DepositParams({
            inputToken: BASE_USDC_ADDRESS,
            outputToken: USDC_ADDRESS,
            inputAmount: 1_000_000, // 1 USDC
            outputAmount: 200_000, // 0.2 USDC
            destinationChainId: CHAIN_ID,
            fillDeadlineBuffer: TIME_BUFFER
        });
        SwapParams memory swapParams = SwapParams({
            inputAmountForFee: 500_000, // 0.5 USDC for fees
            poolFee: 500,
            sqrtPriceLimitX96: 0
        });
        SendParams memory sendParams;
        // Send the 0.1 USDC to Polkadot
        bytes[] memory assets = new bytes[](1);
        if (keccak256(bytes(vm.envString("L1_NETWORK"))) == keccak256(bytes("mainnet"))) {
            // tx from https://etherscan.io/tx/0x7dd9bc769edcdeaf8c9a3fe41081cf5a1fc560968387426837c3f2b02dbf2115
            assets[0] =
                hex"0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a0b86991c6218b36c1d19d4a2e9eb0ce3606eb4800000000000000000000000000000000000000000000000000000000000186a0";
            sendParams = SendParams({
                xcm: hex"050c140d0102080001010054d82b42bcd22b175d71d62ef2114defcf14344c4b88acf0eb4356737d7fdb4a2caecfbd12bb35e640a8a2a05945b48d332bd677bb5d0f674be45e6a30872f9c18",
                assets: assets,
                claimer: hex"0001010054d82b42bcd22b175d71d62ef2114defcf14344c4b88acf0eb4356737d7fdb4a",
                executionFee: 5_688_737_408_032,
                relayerFee: 50_187_270_355_445
            });
        } else if (keccak256(bytes(vm.envString("L1_NETWORK"))) == keccak256(bytes("sepolia"))) {
            // tx from https://sepolia.etherscan.io/tx/0x7068be9a9fecd2d3fbdca0e28bf1a84d4c05789dacd34cc46eef0d2a4fdd43fb
            assets[0] =
                hex"00000000000000000000000000000000000000000000000000000000000000000000000000000000000000001c7d4b196cb0c7b01d743fbc6116a902379c723800000000000000000000000000000000000000000000000000000000000186a0";
            sendParams = SendParams({
                xcm: hex"050c140d010208000101005827013ddc4082f8252f8729bd2f06e77e7863dea9202a6f0e7a2c34e356e85a2cfdbcb5bc4870d25ce6b36b2d6d927b00a1373ebe803d5fd20fcbe8c5c3c866bb",
                assets: assets,
                claimer: hex"000101005827013ddc4082f8252f8729bd2f06e77e7863dea9202a6f0e7a2c34e356e85a",
                executionFee: 33_329_707_255_987,
                relayerFee: 559_885_563_730_065
            });
        } else {
            revert("Unsupported L1 network");
        }

        IERC20(params.inputToken).approve(l2SnowbridgeAdaptor, params.inputAmount);

        SnowbridgeL2Adaptor(l2SnowbridgeAdaptor)
            .sendTokenAndCall(
                params,
                swapParams,
                sendParams,
                recipient,
                keccak256("TestSnowbridgeL2AdaptorTopicId")
            );

        return;
    }
}

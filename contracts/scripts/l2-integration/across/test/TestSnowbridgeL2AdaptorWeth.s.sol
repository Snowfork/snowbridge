// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

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
    TIME_BUFFER as MAINNET_TIME_BUFFER
} from "../constants/Mainnet.sol";

contract TestSnowbridgeL2AdaptorWeth is Script {
    function run() public {
        vm.startBroadcast();

        address payable l2SnowbridgeAdaptor =
            payable(vm.envAddress("L2_SNOWBRIDGE_ADAPTOR_ADDRESS"));
        address recipient = vm.envAddress("RECIPIENT_ADDRESS");
        uint256 CHAIN_ID;
        uint32 TIME_BUFFER;
        address inputToken;
        address outputToken;
        uint256 inputAmount;
        uint256 outputAmount;
        if (keccak256(bytes(vm.envString("L1_NETWORK"))) == keccak256(bytes("mainnet"))) {
            CHAIN_ID = MAINNET_CHAIN_ID;
            TIME_BUFFER = MAINNET_TIME_BUFFER;
            inputAmount = 1_200_000_000_000_000; // 0.0012 ETH
            outputAmount = 1_000_000_000_000_000; // 0.001 ETH
            inputToken = MAINNET_BASE_WETH9;
            outputToken = MAINNET_WETH9;
        } else if (keccak256(bytes(vm.envString("L1_NETWORK"))) == keccak256(bytes("sepolia"))) {
            CHAIN_ID = SEPOLIA_CHAIN_ID;
            TIME_BUFFER = SEPOLIA_TIME_BUFFER;
            inputAmount = 12_000_000_000_000_000; // 0.012 ETH
            outputAmount = 10_000_000_000_000_000; // 0.01 ETH
            inputToken = SEPOLIA_BASE_WETH9;
            outputToken = SEPOLIA_WETH9;
        } else {
            revert("Unsupported L1 network");
        }
        DepositParams memory params = DepositParams({
            inputToken: inputToken,
            outputToken: outputToken,
            inputAmount: inputAmount,
            outputAmount: outputAmount,
            destinationChainId: CHAIN_ID,
            fillDeadlineBuffer: TIME_BUFFER
        });
        // Send 0.001 ETH to Polkadot,
        bytes[] memory assets = new bytes[](0);
        SendParams memory sendParams;

        if (keccak256(bytes(vm.envString("L1_NETWORK"))) == keccak256(bytes("mainnet"))) {
            // tx from https://etherscan.io/tx/0x57d799b6e564c8db30fa91e5d311528814a6a29a22eee3c279e15d73778b1892
            sendParams = SendParams({
                xcm: hex"050c140d0102080001010054d82b42bcd22b175d71d62ef2114defcf14344c4b88acf0eb4356737d7fdb4a2c881a4e2c885398241abf54c551d4308c60bb0e9f2f860c26d1ec9528fb30a5fd",
                assets: assets,
                claimer: hex"0001010054d82b42bcd22b175d71d62ef2114defcf14344c4b88acf0eb4356737d7fdb4a",
                executionFee: 5_688_771_233_667,
                relayerFee: 50_035_501_219_494
            });
        } else if (keccak256(bytes(vm.envString("L1_NETWORK"))) == keccak256(bytes("sepolia"))) {
            // tx from https://sepolia.etherscan.io/tx/0x7e1668a805d24e0e51a04a51f6d6dc0a4b87dfe85f04eb76328c206700567d2b
            sendParams = SendParams({
                xcm: hex"050c140d010208000101005827013ddc4082f8252f8729bd2f06e77e7863dea9202a6f0e7a2c34e356e85a2c964edfa9919080fefce42be38a07df8d7586c641f9f88a75b27c1e0d6001fa34",
                assets: assets,
                claimer: hex"000101005827013ddc4082f8252f8729bd2f06e77e7863dea9202a6f0e7a2c34e356e85a",
                executionFee: 33_346_219_347_761,
                relayerFee: 553_808_951_460_256
            });
        } else {
            revert("Unsupported L1 network");
        }
        WETH9(payable(params.inputToken)).deposit{value: params.inputAmount}();

        IERC20(params.inputToken).approve(l2SnowbridgeAdaptor, params.inputAmount);

        SnowbridgeL2Adaptor(l2SnowbridgeAdaptor)
            .sendEtherAndCall(params, sendParams, recipient, keccak256("TestWethL2AdaptorTopicId"));

        return;
    }
}

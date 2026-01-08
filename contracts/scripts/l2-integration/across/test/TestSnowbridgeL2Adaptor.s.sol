// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {Script, console} from "forge-std/Script.sol";
import {IERC20} from "openzeppelin/token/ERC20/IERC20.sol";
import {WETH9} from "canonical-weth/WETH9.sol";

import {SnowbridgeL2Adaptor} from "../SnowbridgeL2Adaptor.sol";
import {USDC, BASE_USDC, CHAIN_ID, BASE_CHAIN_ID, BASE_WETH9} from "../constants/Sepolia.sol";
import {ISpokePool, IMessageHandler} from "../interfaces/ISpokePool.sol";
import {SwapParams, SendParams} from "../Types.sol";

contract TestSnowbridgeL2Adaptor is Script {
    function run() public {
        vm.startBroadcast();

        address payable l2SnowbridgeAdaptor =
            payable(vm.envAddress("L2_SNOWBRIDGE_ADAPTOR_ADDRESS"));
        address recipient = vm.envAddress("RECIPIENT_ADDRESS");

        SwapParams memory params = SwapParams({
            inputToken: BASE_USDC,
            outputToken: USDC,
            inputAmount: 110_000, // 0.11 USDC
            outputAmount: 100_000, // 0.1 USDC
            destinationChainId: CHAIN_ID
        });
        // Send the 0.1 USDC to Polkadot, tx from https://sepolia.etherscan.io/tx/0x7068be9a9fecd2d3fbdca0e28bf1a84d4c05789dacd34cc46eef0d2a4fdd43fb
        bytes[] memory assets = new bytes[](1);
        assets[0] =
            hex"00000000000000000000000000000000000000000000000000000000000000000000000000000000000000001c7d4b196cb0c7b01d743fbc6116a902379c723800000000000000000000000000000000000000000000000000000000000186a0";
        SendParams memory sendParams = SendParams({
            xcm: hex"050c140d010208000101005827013ddc4082f8252f8729bd2f06e77e7863dea9202a6f0e7a2c34e356e85a2cfdbcb5bc4870d25ce6b36b2d6d927b00a1373ebe803d5fd20fcbe8c5c3c866bb",
            assets: assets,
            claimer: hex"000101005827013ddc4082f8252f8729bd2f06e77e7863dea9202a6f0e7a2c34e356e85a",
            executionFee: 33_329_707_255_987,
            relayerFee: 559_885_563_730_065,
            l2Fee: 20_000_000_000_000
        });

        uint256 nativeFeeAmount =
            sendParams.relayerFee + sendParams.executionFee + sendParams.l2Fee;

        IERC20(params.inputToken).approve(l2SnowbridgeAdaptor, params.inputAmount);

        SnowbridgeL2Adaptor(l2SnowbridgeAdaptor)
        .sendTokenAndCall{
            value: nativeFeeAmount
        }(params, sendParams, recipient, keccak256("TestSnowbridgeL2AdaptorTopicId"));

        return;
    }
}

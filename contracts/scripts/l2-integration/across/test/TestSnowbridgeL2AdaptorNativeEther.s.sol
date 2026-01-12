// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {Script, console} from "forge-std/Script.sol";
import {IERC20} from "openzeppelin/token/ERC20/IERC20.sol";
import {WETH9} from "canonical-weth/WETH9.sol";

import {SnowbridgeL2Adaptor} from "../../../../src/l2-integration/SnowbridgeL2Adaptor.sol";
import {
    USDC,
    BASE_USDC,
    CHAIN_ID,
    BASE_CHAIN_ID,
    BASE_WETH9,
    TIME_BUFFER
} from "../constants/Sepolia.sol";
import {ISpokePool} from "../../../../src/l2-integration/interfaces/ISpokePool.sol";
import {SwapParams, SendParams} from "../../../../src/l2-integration/Types.sol";

contract TestSnowbridgeL2AdaptorNativeEther is Script {
    function run() public {
        vm.startBroadcast();

        address payable l2SnowbridgeAdaptor =
            payable(vm.envAddress("L2_SNOWBRIDGE_ADAPTOR_ADDRESS"));
        address recipient = vm.envAddress("RECIPIENT_ADDRESS");
        SwapParams memory params = SwapParams({
            inputToken: address(0),
            outputToken: address(0),
            inputAmount: 11_000_000_000_000_000, // 0.011 ETH
            outputAmount: 10_000_000_000_000_000, // 0.01 ETH
            destinationChainId: CHAIN_ID,
            fillDeadlineBuffer: TIME_BUFFER
        });
        // Send 0.01 ETH to Polkadot, tx from https://sepolia.etherscan.io/tx/0x7e1668a805d24e0e51a04a51f6d6dc0a4b87dfe85f04eb76328c206700567d2b
        bytes[] memory assets = new bytes[](0);
        SendParams memory sendParams = SendParams({
            xcm: hex"050c140d010208000101005827013ddc4082f8252f8729bd2f06e77e7863dea9202a6f0e7a2c34e356e85a2c964edfa9919080fefce42be38a07df8d7586c641f9f88a75b27c1e0d6001fa34",
            assets: assets,
            claimer: hex"000101005827013ddc4082f8252f8729bd2f06e77e7863dea9202a6f0e7a2c34e356e85a",
            executionFee: 33_346_219_347_761,
            relayerFee: 553_808_951_460_256,
            l2Fee: 1_000_000_000_000_000
        });

        uint256 totalAmount = sendParams.relayerFee + sendParams.executionFee + sendParams.l2Fee
            + params.outputAmount;

        SnowbridgeL2Adaptor(l2SnowbridgeAdaptor)
        .sendNativeEtherAndCall{
            value: totalAmount
        }(params, sendParams, recipient, keccak256("TestSnowbridgeL2AdaptorTopicId"));

        return;
    }
}

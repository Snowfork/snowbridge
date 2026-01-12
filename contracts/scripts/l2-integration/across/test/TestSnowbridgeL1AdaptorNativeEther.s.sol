// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {Script, console} from "forge-std/Script.sol";
import {IERC20} from "openzeppelin/token/ERC20/IERC20.sol";

import {SnowbridgeL1Adaptor} from "../../../../src/l2-integration/SnowbridgeL1Adaptor.sol";
import {USDC, BASE_USDC, CHAIN_ID, BASE_CHAIN_ID, TIME_BUFFER} from "../constants/Sepolia.sol";
import {ISpokePool} from "../../../../src/l2-integration/interfaces/ISpokePool.sol";
import {SwapParams, SendParams} from "../../../../src/l2-integration/Types.sol";

contract TestSnowbridgeL1AdaptorNativeEther is Script {
    function run() public {
        vm.startBroadcast();

        address payable l1SnowbridgeAdaptor =
            payable(vm.envAddress("L1_SNOWBRIDGE_ADAPTOR_ADDRESS"));

        address recipient = vm.envAddress("RECIPIENT_ADDRESS");

        SwapParams memory params = SwapParams({
            inputToken: address(0),
            outputToken: address(0),
            inputAmount: 11_000_000_000_000_000, // 0.011 ETH
            outputAmount: 10_000_000_000_000_000, // 0.01 ETH
            destinationChainId: BASE_CHAIN_ID,
            fillDeadlineBuffer: TIME_BUFFER
        });

        SnowbridgeL1Adaptor(l1SnowbridgeAdaptor)
        .depositNativeEther{
            value: params.inputAmount
        }(params, recipient, keccak256("TestSnowbridgeL1AdaptorTopicId"));

        return;
    }
}

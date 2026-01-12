// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {Script, console} from "forge-std/Script.sol";
import {IERC20} from "openzeppelin/token/ERC20/IERC20.sol";

import {SnowbridgeL1Adaptor} from "../../../../src/l2-integration/SnowbridgeL1Adaptor.sol";
import {USDC, BASE_USDC, CHAIN_ID, BASE_CHAIN_ID, TIME_BUFFER} from "../constants/Sepolia.sol";
import {ISpokePool} from "../../../../src/l2-integration/interfaces/ISpokePool.sol";
import {SwapParams, SendParams} from "../../../../src/l2-integration/Types.sol";

contract TestSnowbridgeL1Adaptor is Script {
    function setUp() public {}

    function run() public {
        vm.startBroadcast();

        address payable l1SnowbridgeAdaptor =
            payable(vm.envAddress("L1_SNOWBRIDGE_ADAPTOR_ADDRESS"));

        address recipient = vm.envAddress("RECIPIENT_ADDRESS");

        SwapParams memory params = SwapParams({
            inputToken: USDC,
            outputToken: BASE_USDC,
            inputAmount: 1_100_000, // 1.1 USDC
            outputAmount: 1_050_000, // 1.05 BASE_USDC
            destinationChainId: BASE_CHAIN_ID,
            fillDeadlineBuffer: TIME_BUFFER
        });

        IERC20(params.inputToken).transfer(l1SnowbridgeAdaptor, params.inputAmount);

        SnowbridgeL1Adaptor(l1SnowbridgeAdaptor)
            .depositToken(params, recipient, keccak256("TestSnowbridgeL1AdaptorTopicId"));

        return;
    }
}

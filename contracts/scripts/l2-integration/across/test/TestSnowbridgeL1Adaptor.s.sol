// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import {Script, console} from "forge-std/Script.sol";
import {IERC20} from "openzeppelin/token/ERC20/IERC20.sol";
import {SafeERC20} from "openzeppelin/token/ERC20/utils/SafeERC20.sol";

import {SnowbridgeL1Adaptor} from "../../../../src/l2-integration/SnowbridgeL1Adaptor.sol";
import {ISpokePool} from "../../../../src/l2-integration/interfaces/ISpokePool.sol";
import {DepositParams, SendParams} from "../../../../src/l2-integration/Types.sol";

import {
    USDC as SEPOLIA_USDC,
    BASE_USDC as SEPOLIA_BASE_USDC,
    CHAIN_ID as SEPOLIA_CHAIN_ID,
    BASE_CHAIN_ID as SEPOLIA_BASE_CHAIN_ID,
    TIME_BUFFER as SEPOLIA_TIME_BUFFER
} from "../constants/Sepolia.sol";
import {
    USDC as MAINNET_USDC,
    BASE_USDC as MAINNET_BASE_USDC,
    CHAIN_ID as MAINNET_CHAIN_ID,
    BASE_CHAIN_ID as MAINNET_BASE_CHAIN_ID,
    TIME_BUFFER as MAINNET_TIME_BUFFER
} from "../constants/Mainnet.sol";

contract TestSnowbridgeL1Adaptor is Script {
    using SafeERC20 for IERC20;

    function setUp() public {}

    function run() public {
        vm.startBroadcast();

        address payable l1SnowbridgeAdaptor =
            payable(vm.envAddress("L1_SNOWBRIDGE_ADAPTOR_ADDRESS"));

        address recipient = vm.envAddress("RECIPIENT_ADDRESS");

        address USDC_ADDRESS;
        address BASE_USDC_ADDRESS;
        uint256 BASE_CHAIN_ID;
        uint32 TIME_BUFFER;

        if (keccak256(bytes(vm.envString("L1_NETWORK"))) == keccak256(bytes("mainnet"))) {
            USDC_ADDRESS = MAINNET_USDC;
            BASE_USDC_ADDRESS = MAINNET_BASE_USDC;
            BASE_CHAIN_ID = MAINNET_BASE_CHAIN_ID;
            TIME_BUFFER = MAINNET_TIME_BUFFER;
        } else if (keccak256(bytes(vm.envString("L1_NETWORK"))) == keccak256(bytes("sepolia"))) {
            USDC_ADDRESS = SEPOLIA_USDC;
            BASE_USDC_ADDRESS = SEPOLIA_BASE_USDC;
            BASE_CHAIN_ID = SEPOLIA_BASE_CHAIN_ID;
            TIME_BUFFER = SEPOLIA_TIME_BUFFER;
        } else {
            revert("Unsupported L1 network");
        }

        DepositParams memory params = DepositParams({
            inputToken: USDC_ADDRESS,
            outputToken: BASE_USDC_ADDRESS,
            inputAmount: 1_100_000, // 1.1 USDC
            outputAmount: 1_050_000, // 1.05 BASE_USDC
            destinationChainId: BASE_CHAIN_ID,
            fillDeadlineBuffer: TIME_BUFFER
        });

        IERC20(params.inputToken).forceApprove(l1SnowbridgeAdaptor, params.inputAmount);

        SnowbridgeL1Adaptor(l1SnowbridgeAdaptor)
            .depositToken(params, recipient, keccak256("TestERC20Deposit"));

        return;
    }
}

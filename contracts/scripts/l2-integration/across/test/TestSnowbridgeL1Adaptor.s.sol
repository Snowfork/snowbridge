// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.33;

import {Script, console} from "forge-std/Script.sol";
import {IERC20} from "openzeppelin/token/ERC20/IERC20.sol";

import {SnowbridgeL1Adaptor} from "../../../../src/l2-integration/SnowbridgeL1Adaptor.sol";
import {ISpokePool} from "../../../../src/l2-integration/interfaces/ISpokePool.sol";
import {DepositParams, SendParams} from "../../../../src/l2-integration/Types.sol";

import {
    USDC as SEPOLIA_USDC,
    BASE_USDC as SEPOLIA_BASE_USDC,
    CHAIN_ID as SEPOLIA_CHAIN_ID,
    BASE_CHAIN_ID as SEPOLIA_BASE_CHAIN_ID,
    TIME_BUFFER as SEPOLIA_TIME_BUFFER,
    ARBITRUM_USDC as SEPOLIA_ARBITRUM_USDC,
    ARBITRUM_CHAIN_ID as SEPOLIA_ARBITRUM_CHAIN_ID,
    ARBITRUM_WETH9 as SEPOLIA_ARBITRUM_WETH9
} from "../constants/Sepolia.sol";
import {
    USDC as MAINNET_USDC,
    BASE_USDC as MAINNET_BASE_USDC,
    CHAIN_ID as MAINNET_CHAIN_ID,
    BASE_CHAIN_ID as MAINNET_BASE_CHAIN_ID,
    TIME_BUFFER as MAINNET_TIME_BUFFER
} from "../constants/Mainnet.sol";

contract TestSnowbridgeL1Adaptor is Script {
    function setUp() public {}

    function run() public {
        vm.startBroadcast();

        address payable l1SnowbridgeAdaptor =
            payable(vm.envAddress("L1_SNOWBRIDGE_ADAPTOR_ADDRESS"));

        address recipient = vm.envAddress("RECIPIENT_ADDRESS");

        address USDC_ADDRESS;
        address L2_USDC_ADDRESS;
        uint256 L2_CHAIN_ID;
        uint32 TIME_BUFFER;

        DepositParams memory params;

        if (
            keccak256(bytes(vm.envString("L1_NETWORK"))) == keccak256(bytes("mainnet"))
                && keccak256(bytes(vm.envString("L2_NETWORK"))) == keccak256(bytes("base-mainnet"))
        ) {
            USDC_ADDRESS = MAINNET_USDC;
            L2_USDC_ADDRESS = MAINNET_BASE_USDC;
            L2_CHAIN_ID = MAINNET_BASE_CHAIN_ID;
            TIME_BUFFER = MAINNET_TIME_BUFFER;
            params = DepositParams({
                inputToken: USDC_ADDRESS,
                outputToken: L2_USDC_ADDRESS,
                inputAmount: 1_100_000, // 1.1 USDC
                outputAmount: 1_050_000, // 1.05 BASE_USDC
                destinationChainId: L2_CHAIN_ID,
                fillDeadlineBuffer: TIME_BUFFER
            });
        } else if (
            keccak256(bytes(vm.envString("L1_NETWORK"))) == keccak256(bytes("sepolia"))
                && keccak256(bytes(vm.envString("L2_NETWORK"))) == keccak256(bytes("base-sepolia"))
        ) {
            USDC_ADDRESS = SEPOLIA_USDC;
            L2_USDC_ADDRESS = SEPOLIA_BASE_USDC;
            L2_CHAIN_ID = SEPOLIA_BASE_CHAIN_ID;
            TIME_BUFFER = SEPOLIA_TIME_BUFFER;
            params = DepositParams({
                inputToken: USDC_ADDRESS,
                outputToken: L2_USDC_ADDRESS,
                inputAmount: 50_100_000, // 50.1 USDC
                outputAmount: 50_000_000, // 50 BASE_USDC
                destinationChainId: L2_CHAIN_ID,
                fillDeadlineBuffer: TIME_BUFFER
            });
        } else if (
            keccak256(bytes(vm.envString("L1_NETWORK"))) == keccak256(bytes("sepolia"))
                && keccak256(bytes(vm.envString("L2_NETWORK")))
                    == keccak256(bytes("arbitrum-sepolia"))
        ) {
            USDC_ADDRESS = SEPOLIA_USDC;
            L2_USDC_ADDRESS = SEPOLIA_ARBITRUM_USDC;
            L2_CHAIN_ID = SEPOLIA_ARBITRUM_CHAIN_ID;
            TIME_BUFFER = SEPOLIA_TIME_BUFFER;
            params = DepositParams({
                inputToken: USDC_ADDRESS,
                outputToken: L2_USDC_ADDRESS,
                inputAmount: 50_100_000, // 50.1 USDC
                outputAmount: 50_000_000, // 50 BASE_USDC
                destinationChainId: L2_CHAIN_ID,
                fillDeadlineBuffer: TIME_BUFFER
            });
        } else {
            revert("Unsupported L1 network");
        }

        IERC20(params.inputToken).transfer(l1SnowbridgeAdaptor, params.inputAmount);

        SnowbridgeL1Adaptor(l1SnowbridgeAdaptor)
            .depositToken(params, recipient, keccak256("TestERC20Deposit"));

        return;
    }
}

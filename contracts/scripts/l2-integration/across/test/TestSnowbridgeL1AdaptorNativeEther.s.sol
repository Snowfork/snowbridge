// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.33;

import {Script, console} from "forge-std/Script.sol";
import {IERC20} from "openzeppelin/token/ERC20/IERC20.sol";

import {SnowbridgeL1Adaptor} from "../../../../src/l2-integration/SnowbridgeL1Adaptor.sol";
import {ISpokePool} from "../../../../src/l2-integration/interfaces/ISpokePool.sol";
import {DepositParams, SendParams} from "../../../../src/l2-integration/Types.sol";
import {
    BASE_CHAIN_ID as SEPOLIA_BASE_CHAIN_ID,
    TIME_BUFFER as SEPOLIA_TIME_BUFFER,
    ARBITRUM_CHAIN_ID as SEPOLIA_ARBITRUM_CHAIN_ID,
    BASE_WETH9 as SEPOLIA_BASE_WETH9,
    ARBITRUM_WETH9 as SEPOLIA_ARBITRUM_WETH9
} from "../constants/Sepolia.sol";
import {
    BASE_CHAIN_ID as MAINNET_BASE_CHAIN_ID,
    TIME_BUFFER as MAINNET_TIME_BUFFER,
    BASE_WETH9 as MAINNET_BASE_WETH9
} from "../constants/Mainnet.sol";

contract TestSnowbridgeL1AdaptorNativeEther is Script {
    function run() public {
        vm.startBroadcast();

        address payable l1SnowbridgeAdaptor =
            payable(vm.envAddress("L1_SNOWBRIDGE_ADAPTOR_ADDRESS"));

        address recipient = vm.envAddress("RECIPIENT_ADDRESS");

        uint256 L2_CHAIN_ID;
        uint32 TIME_BUFFER;
        address L2_WETH9;
        DepositParams memory params;
        if (
            keccak256(bytes(vm.envString("L1_NETWORK"))) == keccak256(bytes("mainnet"))
                && keccak256(bytes(vm.envString("L2_NETWORK"))) == keccak256(bytes("base-mainnet"))
        ) {
            L2_CHAIN_ID = MAINNET_BASE_CHAIN_ID;
            TIME_BUFFER = MAINNET_TIME_BUFFER;
            L2_WETH9 = MAINNET_BASE_WETH9;
            params = DepositParams({
                inputToken: address(0),
                outputToken: L2_WETH9,
                inputAmount: 1_100_000_000_000_000, // 0.0011 ETH
                outputAmount: 1_000_000_000_000_000, // 0.001 ETH
                destinationChainId: L2_CHAIN_ID,
                fillDeadlineBuffer: TIME_BUFFER
            });
        } else if (
            keccak256(bytes(vm.envString("L1_NETWORK"))) == keccak256(bytes("sepolia"))
                && keccak256(bytes(vm.envString("L2_NETWORK"))) == keccak256(bytes("base-sepolia"))
        ) {
            L2_CHAIN_ID = SEPOLIA_BASE_CHAIN_ID;
            TIME_BUFFER = SEPOLIA_TIME_BUFFER;
            L2_WETH9 = SEPOLIA_BASE_WETH9;
            params = DepositParams({
                inputToken: address(0),
                outputToken: L2_WETH9,
                inputAmount: 101_000_000_000_000_000, // 0.101 ETH
                outputAmount: 100_000_000_000_000_000, // 0.1 ETH
                destinationChainId: L2_CHAIN_ID,
                fillDeadlineBuffer: TIME_BUFFER
            });
        } else if (
            keccak256(bytes(vm.envString("L1_NETWORK"))) == keccak256(bytes("sepolia"))
                && keccak256(bytes(vm.envString("L2_NETWORK")))
                    == keccak256(bytes("arbitrum-sepolia"))
        ) {
            L2_CHAIN_ID = SEPOLIA_ARBITRUM_CHAIN_ID;
            TIME_BUFFER = SEPOLIA_TIME_BUFFER;
            L2_WETH9 = SEPOLIA_ARBITRUM_WETH9;
            params = DepositParams({
                inputToken: address(0),
                outputToken: L2_WETH9,
                inputAmount: 101_000_000_000_000_000, // 0.101 ETH
                outputAmount: 100_000_000_000_000_000, // 0.1 ETH
                destinationChainId: L2_CHAIN_ID,
                fillDeadlineBuffer: TIME_BUFFER
            });
        } else {
            revert("Unsupported L1 network");
        }

        // Prefund the adaptor with native Ether for the outgoing deposit
        (bool prefunded,) = l1SnowbridgeAdaptor.call{value: params.inputAmount}("");
        require(prefunded, "Failed to prefund adaptor with ETH");

        SnowbridgeL1Adaptor(l1SnowbridgeAdaptor)
            .depositNativeEther(params, recipient, keccak256("TestNativeEtherDeposit"));

        return;
    }
}

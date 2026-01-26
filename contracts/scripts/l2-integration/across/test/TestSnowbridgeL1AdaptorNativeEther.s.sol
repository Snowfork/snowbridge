// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.33;

import {Script, console} from "forge-std/Script.sol";
import {IERC20} from "openzeppelin/token/ERC20/IERC20.sol";

import {SnowbridgeL1Adaptor} from "../../../../src/l2-integration/SnowbridgeL1Adaptor.sol";
import {ISpokePool} from "../../../../src/l2-integration/interfaces/ISpokePool.sol";
import {DepositParams, SendParams} from "../../../../src/l2-integration/Types.sol";
import {
    BASE_CHAIN_ID as SEPOLIA_BASE_CHAIN_ID,
    TIME_BUFFER as SEPOLIA_TIME_BUFFER
} from "../constants/Sepolia.sol";
import {
    BASE_CHAIN_ID as MAINNET_BASE_CHAIN_ID,
    TIME_BUFFER as MAINNET_TIME_BUFFER
} from "../constants/Mainnet.sol";

contract TestSnowbridgeL1AdaptorNativeEther is Script {
    function run() public {
        vm.startBroadcast();

        address payable l1SnowbridgeAdaptor =
            payable(vm.envAddress("L1_SNOWBRIDGE_ADAPTOR_ADDRESS"));

        address payable recipient = payable(vm.envAddress("RECIPIENT_ADDRESS"));

        uint256 BASE_CHAIN_ID;
        uint32 TIME_BUFFER;
        if (keccak256(bytes(vm.envString("L1_NETWORK"))) == keccak256(bytes("mainnet"))) {
            BASE_CHAIN_ID = MAINNET_BASE_CHAIN_ID;
            TIME_BUFFER = MAINNET_TIME_BUFFER;
        } else if (keccak256(bytes(vm.envString("L1_NETWORK"))) == keccak256(bytes("sepolia"))) {
            BASE_CHAIN_ID = SEPOLIA_BASE_CHAIN_ID;
            TIME_BUFFER = SEPOLIA_TIME_BUFFER;
        } else {
            revert("Unsupported L1 network");
        }

        DepositParams memory params = DepositParams({
            inputToken: address(0),
            outputToken: address(0),
            inputAmount: 1_100_000_000_000_000, // 0.0011 ETH
            outputAmount: 1_000_000_000_000_000, // 0.001 ETH
            destinationChainId: BASE_CHAIN_ID,
            fillDeadlineBuffer: TIME_BUFFER
        });

        // Prefund the adaptor with native Ether for the outgoing deposit
        (bool prefunded,) = l1SnowbridgeAdaptor.call{value: params.inputAmount}("");
        require(prefunded, "Failed to prefund adaptor with ETH");

        SnowbridgeL1Adaptor(l1SnowbridgeAdaptor)
            .depositNativeEther(params, recipient, keccak256("TestNativeEtherDeposit"));

        return;
    }
}

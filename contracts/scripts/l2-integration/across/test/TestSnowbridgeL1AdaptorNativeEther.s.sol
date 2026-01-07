// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {Script, console} from "forge-std/Script.sol";
import {IERC20} from "openzeppelin/token/ERC20/IERC20.sol";

import {SnowbridgeL1Adaptor} from "../SnowbridgeL1Adaptor.sol";
import {USDC, BASE_USDC, CHAIN_ID, BASE_CHAIN_ID} from "../constants/Sepolia.sol";
import {ISpokePool, IMessageHandler} from "../interfaces/ISpokePool.sol";
import {SwapParams, SendParams} from "../Types.sol";

contract TestSnowbridgeL1AdaptorNativeEther is Script {
    uint256 internal deployerPrivateKey = vm.envUint("DEPLOYER_KEY");
    address deployerAddr = vm.addr(deployerPrivateKey);

    function setUp() public {}

    function run() public {
        vm.startBroadcast(deployerPrivateKey);

        address payable l1SnowbridgeAdaptor =
            payable(vm.envAddress("L1_SNOWBRIDGE_ADAPTOR_ADDRESS"));

        SwapParams memory params = SwapParams({
            inputToken: address(0),
            outputToken: address(0),
            inputAmount: 11_000_000_000_000_000, // 0.011 ETH
            outputAmount: 10_000_000_000_000_000, // 0.01 ETH
            destinationChainId: BASE_CHAIN_ID
        });

        SnowbridgeL1Adaptor(l1SnowbridgeAdaptor)
        .depositNativeEther{
            value: params.inputAmount
        }(params, deployerAddr, keccak256("TestSnowbridgeL1AdaptorTopicId"));

        vm.stopBroadcast();
        return;
    }
}

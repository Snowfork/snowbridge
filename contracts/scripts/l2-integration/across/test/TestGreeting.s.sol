// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;
import {Script, console} from "forge-std/Script.sol";
import {IERC20} from "openzeppelin/token/ERC20/IERC20.sol";
import {Greeter} from "./Greeter.sol";
import {USDC, BASE_USDC, CHAIN_ID, BASE_CHAIN_ID} from "../constants/Sepolia.sol";
import {ISpokePool, IMessageHandler} from "../interfaces/ISpokePool.sol";
import {SwapParams} from "../Types.sol";

contract TestGreeting is Script {
    uint256 internal deployerPrivateKey = vm.envUint("DEPLOYER_KEY");
    address deployerAddr = vm.addr(deployerPrivateKey);

    function setUp() public {}

    function run() public {
        vm.startBroadcast(deployerPrivateKey);

        address l1Greeter = vm.envAddress("L1_GREETER_ADDRESS");
        address l2Greeter = vm.envAddress("L2_GREETER_ADDRESS");
        bool isL1 = vm.envBool("IS_L1");
        if (isL1) {
            console.log("Sending greeting from L1 Sepolia to L2 Base");
            //Todo: adjust outputAmount based on fee calculations from Across SDK
            SwapParams memory params = SwapParams({
                inputToken: USDC,
                outputToken: BASE_USDC,
                inputAmount: 1_100_000, // 1.1 USDC
                outputAmount: 1_050_000, // 1.05 BASE USDC
                destinationChainId: BASE_CHAIN_ID
            });

            IERC20(params.inputToken).transfer(l1Greeter, params.inputAmount);

            Greeter(l1Greeter).swapTokenAndGreeting(params, "Hello from L1 Sepolia!", deployerAddr);
        } else {
            console.log("Sending greeting from L2 Base to L1 Sepolia");
            SwapParams memory params = SwapParams({
                inputToken: BASE_USDC,
                outputToken: USDC,
                inputAmount: 110_000, // 0.11 Base USDC
                outputAmount: 100_000, // 0.1 USDC
                destinationChainId: CHAIN_ID
            });

            IERC20(params.inputToken).transfer(l2Greeter, params.inputAmount);

            Greeter(l2Greeter).swapTokenAndGreeting(params, "Hello from L2 Base!", deployerAddr);
        }

        vm.stopBroadcast();
        return;
    }
}

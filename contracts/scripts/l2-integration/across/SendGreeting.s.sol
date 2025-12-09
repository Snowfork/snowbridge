// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;
import {Script, console} from "forge-std/Script.sol";
import {IERC20} from "openzeppelin/token/ERC20/IERC20.sol";
import {Greeter} from "./Greeter.sol";
import {USDC, BASE_USDC, BASE_CHAIN_ID} from "./Constants.sol";

contract SendGreeting is Script {
    uint256 internal deployerPrivateKey = vm.envUint("DEPLOYER_KEY");
    address deployerAddr = vm.addr(deployerPrivateKey);

    function setUp() public {}

    function run() public {
        vm.startBroadcast(deployerPrivateKey);

        address l1Greeter = vm.envAddress("L1_GREETER_ADDRESS");

        //Todo: adjust outputAmount based on fee calculations from Across SDK
        Greeter.SwapParams memory params = Greeter.SwapParams({
            inputToken: USDC,
            outputToken: BASE_USDC,
            inputAmount: 110_000, // 0.11 USDC
            outputAmount: 100_000, // 0.1 USDC
            destinationChainId: BASE_CHAIN_ID
        });

        IERC20(params.inputToken).transfer(l1Greeter, params.inputAmount);

        Greeter(l1Greeter).swapTokenAndGreeting(params, "Hello from L1 Sepolia!", deployerAddr);

        vm.stopBroadcast();
        return;
    }
}

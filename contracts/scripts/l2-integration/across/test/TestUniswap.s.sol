// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Script.sol";
import "forge-std/console.sol";
import "openzeppelin/token/ERC20/IERC20.sol";

import {ISwapRouter} from "../interfaces/ISwapRouter.sol";

contract SwapScript is Script {
    function run() external {
        vm.startBroadcast();

        ISwapRouter router = ISwapRouter(0x65669fE35312947050C450Bd5d36e6361F85eC12);

        address WETH = 0xfFf9976782d46CC05630D1f6eBAb18b2324d6B14;

        address DAI = 0xe9354B0Bac99ea17e8162d97Adc0583aB2029479;

        uint24 FEE = 3000;

        uint256 amountIn = 0.001 ether;

        IERC20(WETH).approve(address(router), amountIn);

        ISwapRouter.ExactInputSingleParams memory params = ISwapRouter.ExactInputSingleParams({
            tokenIn: WETH,
            tokenOut: DAI,
            fee: FEE,
            recipient: msg.sender,
            deadline: block.timestamp + 300, // 5 minutes from now
            amountIn: amountIn,
            amountOutMinimum: 0, // ⚠️ fine for tests only
            sqrtPriceLimitX96: 0
        });

        uint256 amountOut = router.exactInputSingle(params);

        console.log("Swapped WETH for DAI:", amountIn, amountOut);

        vm.stopBroadcast();
    }
}

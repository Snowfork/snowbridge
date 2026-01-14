// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Script.sol";
import "forge-std/console.sol";
import "openzeppelin/token/ERC20/IERC20.sol";
import {WETH9} from "canonical-weth/WETH9.sol";

import {ISwapRouter} from "../../../../src/l2-integration/interfaces/ISwapRouter.sol";

contract SwapScript is Script {
    function run() external {
        vm.startBroadcast();

        ISwapRouter router = ISwapRouter(0xE592427A0AEce92De3Edee1F18E0157C05861564);

        address USDC = 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48;

        address WETH = 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2;

        uint24 POOL_FEE = 500; // 0.05%

        uint256 amountInMaximum = 1_000_000; // 1 USDC (6 decimals)
        uint256 amountOut = 55_876_007_763_477; // ~0.000055876007763477 WETH (18 decimals)

        IERC20(USDC).approve(address(router), amountInMaximum);

        ISwapRouter.ExactOutputSingleParams memory params = ISwapRouter.ExactOutputSingleParams({
            tokenIn: USDC,
            tokenOut: WETH,
            fee: POOL_FEE,
            recipient: msg.sender,
            deadline: block.timestamp + 300, // 5 minutes from now
            amountInMaximum: amountInMaximum,
            amountOut: amountOut,
            sqrtPriceLimitX96: 0
        });

        uint256 amountIn = router.exactOutputSingle(params);

        //Swapped USDC for WETH: 184208 55876007763477
        console.log("Swapped USDC for WETH:", amountIn, amountOut);

        WETH9(payable(WETH)).withdraw(amountOut);
        payable(msg.sender).transfer(amountOut);

        vm.stopBroadcast();
    }
}

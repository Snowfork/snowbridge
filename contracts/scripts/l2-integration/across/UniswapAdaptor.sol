// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {ISwapRouter} from "./Interfaces.sol";
import "openzeppelin/token/ERC20/IERC20.sol";

contract UniswapSepoliaSwap {
    ISwapRouter public constant router = ISwapRouter(0x65669fE35312947050C450Bd5d36e6361F85eC12);

    address public constant WETH = 0xfFf9976782d46CC05630D1f6eBAb18b2324d6B14;

    address public constant USDC = 0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238;

    uint24 public constant FEE = 3000;

    function swapWETHtoUSDC(uint256 amountIn) external returns (uint256 amountOut) {
        // pull WETH from caller
        IERC20(WETH).transferFrom(msg.sender, address(this), amountIn);

        // approve router
        IERC20(WETH).approve(address(router), amountIn);

        ISwapRouter.ExactInputSingleParams memory params = ISwapRouter.ExactInputSingleParams({
            tokenIn: WETH,
            tokenOut: USDC,
            fee: FEE,
            recipient: msg.sender,
            deadline: block.timestamp + 300, // 5 minutes from now
            amountIn: amountIn,
            amountOutMinimum: 0, // ⚠️ fine for tests only
            sqrtPriceLimitX96: 0
        });

        amountOut = router.exactInputSingle(params);
    }
}

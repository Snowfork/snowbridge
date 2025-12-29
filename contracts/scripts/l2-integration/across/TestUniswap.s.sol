// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Script.sol";
import "./UniswapSepoliaSwap.sol";
import "openzeppelin/token/ERC20/IERC20.sol";
import "forge-std/console.sol";

contract SwapScript is Script {
    function run() external {
        vm.startBroadcast();

        address payable swapperContractAddress =
            payable(vm.envAddress("UNISWAP_SEPOLIA_SWAP_ADDRESS"));
        UniswapSepoliaSwap swapperContract = UniswapSepoliaSwap(swapperContractAddress);

        address WETH = swapperContract.WETH();
        IERC20 weth = IERC20(WETH);

        uint256 amountIn = 0.001 ether;

        weth.approve(address(swapperContract), amountIn);
        uint256 amountOut = swapperContract.swapWETHtoUSDC(amountIn);
        console.log("Swapped WETH for USDC:", amountIn, amountOut);

        vm.stopBroadcast();
    }
}

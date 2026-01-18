// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Script.sol";
import "forge-std/console.sol";
import "openzeppelin/token/ERC20/IERC20.sol";
import {WETH9} from "canonical-weth/WETH9.sol";

import {ISwapRouter} from "../interfaces/ISwapRouter.sol";
import {ISwapQuoter} from "../interfaces/ISwapQuoter.sol";

import {
    USDC as MAINNET_USDC,
    WETH9 as MAINNET_WETH9,
    UNISWAP_QUOTER as MAINNET_UNISWAP_QUOTER
} from "../constants/Mainnet.sol";
import {
    USDC as SEPOLIA_USDC,
    WETH9 as SEPOLIA_WETH9,
    UNISWAP_QUOTER as SEPOLIA_UNISWAP_QUOTER
} from "../constants/Sepolia.sol";

contract TestUniswapQuoter is Script {
    function run() external {
        vm.startBroadcast();

        uint24 POOL_FEE = 500; // 0.05%
        uint256 amountIn;

        if (keccak256(bytes(vm.envString("L1_NETWORK"))) == keccak256(bytes("mainnet"))) {
            uint256 amountOut = 55_876_007_763_477; // ~0.000055876007763477 WETH (18 decimals)
            amountIn =
                getQuote(MAINNET_UNISWAP_QUOTER, MAINNET_USDC, MAINNET_WETH9, amountOut, POOL_FEE);
        } else if (keccak256(bytes(vm.envString("L1_NETWORK"))) == keccak256(bytes("sepolia"))) {
            uint256 amountOut = 593_215_270_986_052; // ~0.000593215270986052 WETH (18 decimals)
            amountIn =
                getQuote(SEPOLIA_UNISWAP_QUOTER, SEPOLIA_USDC, SEPOLIA_WETH9, amountOut, POOL_FEE);
        } else {
            revert("Unsupported L1 network");
        }

        console.log("Required USDC for WETH:", amountIn);
        uint256 amountInMaximum = amountIn * (10_000 + POOL_FEE) / 10_000;
        console.log("Required USDC for WETH with slippage:", amountInMaximum);
    }

    function getQuote(
        address quoter,
        address tokenIn,
        address tokenOut,
        uint256 amountOut,
        uint24 poolFee
    ) internal returns (uint256) {
        try ISwapQuoter(quoter)
            .quoteExactOutputSingle(
                ISwapQuoter.QuoteExactOutputSingleParams({
                    tokenIn: tokenIn,
                    tokenOut: tokenOut,
                    fee: poolFee,
                    amount: amountOut,
                    sqrtPriceLimitX96: 0
                })
            ) returns (
            uint256 result0, uint160, uint32, uint256
        ) {
            // Quoter ALWAYS reverts, so this branch should never be reached
            return result0;
        } catch (bytes memory reason) {
            // The quoter returns the result in the revert data
            // Expected revert signature: abi.encodeWithSignature("QuoteResult(uint256,uint160,uint32,uint256)", ...)
            if (reason.length > 4) {
                // Decode skipping the 4-byte error selector
                uint256 result0;

                assembly {
                    let encodedData := add(reason, 0x24) // Skip length prefix and error selector
                    result0 := mload(encodedData)
                }
                return result0;
            } else {
                revert("Quoter failed with empty or invalid revert data");
            }
        }
    }
}

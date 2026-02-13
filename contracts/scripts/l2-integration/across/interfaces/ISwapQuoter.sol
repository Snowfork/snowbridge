// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.33;

interface ISwapQuoter {
    struct QuoteExactOutputSingleParams {
        address tokenIn;
        address tokenOut;
        uint256 amount;
        uint24 fee;
        uint160 sqrtPriceLimitX96;
    }

    /// @notice Given an output amount of an asset and pool fee, returns the required input amount
    /// @param params The parameters necessary for the quote, encoded as `QuoteExactOutputSingleParams` in calldata
    /// @return amountIn The amount of the input token
    /// @return sqrtPriceX96After The sqrt price after the swap
    /// @return initializedTicksCrossed The number of initialized ticks crossed to complete the swap
    /// @return gasEstimate The gas estimate for the swap
    function quoteExactOutputSingle(QuoteExactOutputSingleParams memory params)
        external
        returns (
            uint256 amountIn,
            uint160 sqrtPriceX96After,
            uint32 initializedTicksCrossed,
            uint256 gasEstimate
        );
}

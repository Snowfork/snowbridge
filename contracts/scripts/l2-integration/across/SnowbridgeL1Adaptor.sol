// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {IERC20} from "openzeppelin/token/ERC20/IERC20.sol";
import {SafeERC20} from "openzeppelin/token/ERC20/utils/SafeERC20.sol";
import {ISpokePool} from "./Interfaces.sol";
import {SwapParams} from "./Types.sol";

contract SnowbridgeL1Adaptor {
    using SafeERC20 for IERC20;

    ISpokePool public immutable SPOKE_POOL;
    uint32 public waitTime;

    constructor(address _spokePool, uint32 _waitTime) {
        SPOKE_POOL = ISpokePool(_spokePool);
        waitTime = _waitTime;
    }

    // Swap ERC20 token on L1 to get other token on L2, the fee should be calculated off-chain
    // following https://docs.across.to/reference/api-reference#get-swap-approval
    function swapToken(SwapParams calldata params, address recipient) public {
        IERC20(params.inputToken).safeTransfer(address(this), params.inputAmount);
        IERC20(params.inputToken).approve(address(SPOKE_POOL), params.inputAmount);

        SPOKE_POOL.deposit(
            bytes32(uint256(uint160(recipient))),
            bytes32(uint256(uint160(recipient))),
            bytes32(uint256(uint160(params.inputToken))),
            bytes32(uint256(uint160(params.outputToken))),
            params.inputAmount,
            params.outputAmount,
            params.destinationChainId,
            bytes32(0), // exclusiveRelayer, zero means any relayer can fill
            uint32(block.timestamp - waitTime), // quoteTimestamp set to 10 minutes before now
            uint32(block.timestamp + waitTime), // fillDeadline set to 10 minutes after now
            0, // exclusivityDeadline, zero means no exclusivity
            "" // empty message
        );
    }
}

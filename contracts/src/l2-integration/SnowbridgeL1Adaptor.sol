// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {IERC20} from "openzeppelin/token/ERC20/IERC20.sol";
import {SafeERC20} from "openzeppelin/token/ERC20/utils/SafeERC20.sol";
import {WETH9} from "canonical-weth/WETH9.sol";
import {ISpokePool, IMessageHandler} from "./interfaces/ISpokePool.sol";
import {DepositParams, Instructions, Call} from "./Types.sol";

contract SnowbridgeL1Adaptor {
    using SafeERC20 for IERC20;
    ISpokePool public immutable SPOKE_POOL;
    WETH9 public immutable L1_WETH9;
    WETH9 public immutable L2_WETH9;

    /**************************************
     *              EVENTS                *
     **************************************/

    event DepositCallInvoked(bytes32 topic, uint256 depositId);

    constructor(address _spokePool, address _l1weth9, address _l2weth9) {
        SPOKE_POOL = ISpokePool(_spokePool);
        L1_WETH9 = WETH9(payable(_l1weth9));
        L2_WETH9 = WETH9(payable(_l2weth9));
    }

    // Send ERC20 token on L1 to L2 via the Across protocol.
    // The fee (params.inputAmount - params.outputAmount) should be calculated off-chain
    // following https://docs.across.to/reference/api-reference#get-swap-approval
    // Tokens are pulled from the caller via safeTransferFrom.
    function depositToken(DepositParams calldata params, address recipient, bytes32 topic) public {
        require(params.inputToken != address(0), "Input token cannot be zero address");
        checkInputs(params, recipient);

        // Pull tokens from the caller to avoid relying on pre-funding and then approve SpokePool
        IERC20(params.inputToken).safeTransferFrom(msg.sender, address(this), params.inputAmount);
        IERC20(params.inputToken).forceApprove(address(SPOKE_POOL), params.inputAmount);

        SPOKE_POOL.deposit(
            bytes32(uint256(uint160(recipient))),
            bytes32(uint256(uint160(recipient))),
            bytes32(uint256(uint160(params.inputToken))),
            bytes32(uint256(uint160(params.outputToken))),
            params.inputAmount,
            params.outputAmount,
            params.destinationChainId,
            bytes32(0), // exclusiveRelayer, zero means any relayer can fill
            uint32(block.timestamp), // quoteTimestamp set to current block timestamp
            uint32(block.timestamp + params.fillDeadlineBuffer), // fillDeadline set to fillDeadlineBuffer seconds in the future
            0, // exclusivityDeadline, zero means no exclusivity
            "" // empty message
        );

        // Forward any remaining balance of the input token back to the recipient to avoid trapping funds
        uint256 remaining = IERC20(params.inputToken).balanceOf(address(this));
        if (remaining > 0) {
            IERC20(params.inputToken).safeTransfer(recipient, remaining);
        }

        uint256 depositId = SPOKE_POOL.numberOfDeposits() - 1;
        emit DepositCallInvoked(topic, depositId);
    }

    // Send native Ether on L1 to L2 by first wrapping it to WETH, then depositing via SPOKE_POOL.
    function depositNativeEther(
        DepositParams calldata params,
        address payable recipient,
        bytes32 topic
    ) public payable {
        require(
            params.inputToken == address(0),
            "Input token must be zero address for native ETH deposits"
        );
        checkInputs(params, recipient);

        // Wrap native ETH to L1 WETH
        L1_WETH9.deposit{value: params.inputAmount}();

        // Approve SPOKE_POOL to spend the wrapped WETH
        IERC20(address(L1_WETH9)).forceApprove(address(SPOKE_POOL), params.inputAmount);

        // Deposit WETH via SPOKE_POOL
        SPOKE_POOL.deposit(
            bytes32(uint256(uint160(address(recipient)))),
            bytes32(uint256(uint160(address(recipient)))),
            bytes32(uint256(uint160(address(L1_WETH9)))),
            bytes32(uint256(uint160(address(L2_WETH9)))),
            params.inputAmount,
            params.outputAmount,
            params.destinationChainId,
            bytes32(0), // exclusiveRelayer, zero means any relayer can fill
            uint32(block.timestamp), // quoteTimestamp set to current block timestamp
            uint32(block.timestamp + params.fillDeadlineBuffer), // fillDeadline set to fillDeadlineBuffer seconds in the future
            0, // exclusivityDeadline, zero means no exclusivity
            "" // empty message
        );

        // Forward any remaining balance back to the recipient to avoid trapping funds
        uint256 remaining = address(this).balance;
        if (remaining > 0) {
            (bool success,) = recipient.call{value: remaining}("");
            require(success, "Failed to transfer remaining ether to recipient");
        }

        uint256 depositId = SPOKE_POOL.numberOfDeposits() - 1;
        emit DepositCallInvoked(topic, depositId);
    }

    function checkInputs(DepositParams calldata params, address recipient) internal pure {
        require(params.inputAmount > 0, "Input amount must be greater than zero");
        require(params.outputAmount > 0, "Output amount must be greater than zero");
        require(params.outputAmount <= params.inputAmount, "Output amount exceeds input amount");
        require(params.fillDeadlineBuffer > 0, "Fill deadline buffer must be greater than zero");
        require(params.destinationChainId != 0, "Destination chain ID cannot be zero");
        require(recipient != address(0), "Recipient cannot be zero address");
    }

    receive() external payable {}
}

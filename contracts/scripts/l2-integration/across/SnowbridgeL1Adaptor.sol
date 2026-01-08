// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {IERC20} from "openzeppelin/token/ERC20/IERC20.sol";
import {SafeERC20} from "openzeppelin/token/ERC20/utils/SafeERC20.sol";
import {WETH9} from "canonical-weth/WETH9.sol";
import {ISpokePool, IMessageHandler} from "./interfaces/ISpokePool.sol";
import {SwapParams, Instructions, Call} from "./Types.sol";

contract SnowbridgeL1Adaptor {
    using SafeERC20 for IERC20;
    ISpokePool public immutable SPOKE_POOL;
    IMessageHandler public immutable MULTI_CALL_HANDLER;
    WETH9 public immutable L1_WETH9;
    WETH9 public immutable L2_WETH9;

    /**************************************
     *              EVENTS                *
     **************************************/

    event DepositCallInvoked(bytes32 topic, uint256 depositId);

    constructor(address _spokePool, address _handler, address _l1weth9, address _l2weth9) {
        SPOKE_POOL = ISpokePool(_spokePool);
        MULTI_CALL_HANDLER = IMessageHandler(_handler);
        L1_WETH9 = WETH9(payable(_l1weth9));
        L2_WETH9 = WETH9(payable(_l2weth9));
    }

    // Send ERC20 token on L1 to L2, the fee should be calculated off-chain
    // following https://docs.across.to/reference/api-reference#get-swap-approval
    function depositToken(SwapParams calldata params, address recipient, bytes32 topic) public {
        require(params.inputToken != address(0), "Input token cannot be zero address");
        checkInputs(params, recipient);
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
        // Emit event with the depositId of the deposit
        uint256 depositId = SPOKE_POOL.numberOfDeposits() - 1;
        emit DepositCallInvoked(topic, depositId);
    }

    // Send native Ether on L1 to L2
    function depositNativeEther(SwapParams calldata params, address recipient, bytes32 topic)
        public
        payable
    {
        require(
            params.inputToken == address(0),
            "Input token must be zero address for native ETH deposits"
        );
        checkInputs(params, recipient);
        require(msg.value >= params.inputAmount, "Sent value must be at least input amount");

        L1_WETH9.deposit{value: params.inputAmount}();
        IERC20(address(L1_WETH9)).forceApprove(address(SPOKE_POOL), params.inputAmount);

        // Prepare the calls to be executed on L2 upon deposit fulfillment
        // First, withdraw the output amount of WETH on L2
        // Then, send the withdrawn ETH to the recipient
        Call[] memory calls = new Call[](2);
        calls[0] = Call({
            target: address(L2_WETH9),
            callData: abi.encodeCall(L2_WETH9.withdraw, (params.outputAmount)),
            value: 0
        });
        calls[1] = Call({target: recipient, callData: "", value: params.outputAmount});
        Instructions memory instructions =
            Instructions({calls: calls, fallbackRecipient: recipient});

        SPOKE_POOL.deposit(
            bytes32(uint256(uint160(recipient))),
            bytes32(uint256(uint160(address(MULTI_CALL_HANDLER)))),
            bytes32(uint256(uint160(address(L1_WETH9)))),
            bytes32(uint256(uint160(address(L2_WETH9)))),
            params.inputAmount,
            params.outputAmount,
            params.destinationChainId,
            bytes32(0), // exclusiveRelayer, zero means any relayer can fill
            uint32(block.timestamp), // quoteTimestamp set to current block timestamp
            uint32(block.timestamp + params.fillDeadlineBuffer), // fillDeadline set to fillDeadlineBuffer seconds in the future
            0, // exclusivityDeadline, zero means no exclusivity
            abi.encode(instructions)
        );
        // Refund any excess ETH sent
        if (msg.value > params.inputAmount) {
            payable(msg.sender).transfer(msg.value - params.inputAmount);
        }
        uint256 depositId = SPOKE_POOL.numberOfDeposits() - 1;
        emit DepositCallInvoked(topic, depositId);
    }

    function checkInputs(SwapParams calldata params, address recipient) internal pure {
        require(params.inputAmount > 0, "Input amount must be greater than zero");
        require(params.outputAmount > 0, "Output amount must be greater than zero");
        require(params.outputAmount <= params.inputAmount, "Output amount exceeds input amount");
        require(params.fillDeadlineBuffer > 0, "Fill deadline buffer must be greater than zero");
        require(params.destinationChainId != 0, "Destination chain ID cannot be zero");
        require(recipient != address(0), "Recipient cannot be zero address");
    }

    receive() external payable {}
}

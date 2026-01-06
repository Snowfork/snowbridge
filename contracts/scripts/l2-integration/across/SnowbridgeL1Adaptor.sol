// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {IERC20} from "openzeppelin/token/ERC20/IERC20.sol";
import {WETH9} from "canonical-weth/WETH9.sol";
import {ISpokePool, IMessageHandler} from "./interfaces/ISpokePool.sol";
import {SwapParams, Instructions, Call} from "./Types.sol";

contract SnowbridgeL1Adaptor {
    ISpokePool public immutable SPOKE_POOL;
    IMessageHandler public immutable MULTI_CALL_HANDLER;
    WETH9 public immutable L1_WETH9;
    WETH9 public immutable L2_WETH9;
    uint32 public TIME_BUFFER;

    constructor(
        address _spokePool,
        address _handler,
        address _l1weth9,
        address _l2weth9,
        uint32 _timeBuffer
    ) {
        SPOKE_POOL = ISpokePool(_spokePool);
        MULTI_CALL_HANDLER = IMessageHandler(_handler);
        L1_WETH9 = WETH9(payable(_l1weth9));
        L2_WETH9 = WETH9(payable(_l2weth9));
        TIME_BUFFER = _timeBuffer;
    }

    // Send ERC20 token on L1 to L2, the fee should be calculated off-chain
    // following https://docs.across.to/reference/api-reference#get-swap-approval
    function depositToken(SwapParams calldata params, address recipient) public {
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
            uint32(block.timestamp - TIME_BUFFER), // quoteTimestamp set to 10 minutes before now
            uint32(block.timestamp + TIME_BUFFER), // fillDeadline set to 10 minutes after now
            0, // exclusivityDeadline, zero means no exclusivity
            "" // empty message
        );
    }

    // Send native Ether on L1 to L2
    function depositNativeEther(SwapParams calldata params, address recipient) public payable {
        require(params.inputToken == address(0));
        require(params.inputAmount > params.outputAmount, "Input and output amount mismatch");

        require(msg.value >= params.inputAmount, "Insufficient ETH amount sent");

        L1_WETH9.deposit{value: msg.value}();
        IERC20(address(L1_WETH9)).approve(address(SPOKE_POOL), params.inputAmount);

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
            uint32(block.timestamp - TIME_BUFFER), // quoteTimestamp set to 10 minutes before now
            uint32(block.timestamp + TIME_BUFFER), // fillDeadline set to 10 minutes after now
            0, // exclusivityDeadline, zero means no exclusivity
            abi.encode(instructions)
        );
    }

    receive() external payable {}
}

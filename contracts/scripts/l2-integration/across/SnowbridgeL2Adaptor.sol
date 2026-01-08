// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {IERC20} from "openzeppelin/token/ERC20/IERC20.sol";
import {SafeERC20} from "openzeppelin/token/ERC20/utils/SafeERC20.sol";
import {WETH9} from "canonical-weth/WETH9.sol";
import {ISpokePool, IMessageHandler} from "./interfaces/ISpokePool.sol";
import {IGatewayV2} from "../../../src/v2/IGateway.sol";
import {SwapParams, Instructions, Call, SendParams} from "./Types.sol";

contract SnowbridgeL2Adaptor {
    using SafeERC20 for IERC20;

    ISpokePool public immutable SPOKE_POOL;
    IMessageHandler public immutable MULTI_CALL_HANDLER;
    IGatewayV2 public immutable GATEWAY;
    WETH9 public immutable L1_WETH9;
    WETH9 public immutable L2_WETH9;
    uint32 public TIME_BUFFER;

    /**************************************
     *              EVENTS                *
     **************************************/

    event DepositCallInvoked(bytes32 topic, uint256 depositId);

    constructor(
        address _spokePool,
        address _handler,
        address _gateway,
        address _l1weth9,
        address _l2weth9,
        uint32 _timeBuffer
    ) {
        SPOKE_POOL = ISpokePool(_spokePool);
        GATEWAY = IGatewayV2(_gateway);
        MULTI_CALL_HANDLER = IMessageHandler(_handler);
        L1_WETH9 = WETH9(payable(_l1weth9));
        L2_WETH9 = WETH9(payable(_l2weth9));
        TIME_BUFFER = _timeBuffer;
    }

    // Send ERC20 token to Polkadot, the fee should be calculated off-chain
    function sendTokenAndCall(
        SwapParams calldata params,
        SendParams calldata sendParams,
        address recipient,
        bytes32 topic
    ) public payable {
        require(params.inputToken != address(0), "Input token cannot be zero address");
        require(params.inputAmount > 0, "Input amount must be greater than zero");
        require(params.outputAmount > 0, "Output amount must be greater than zero");
        require(params.outputAmount <= params.inputAmount, "Output amount exceeds input amount");
        require(recipient != address(0), "Recipient cannot be zero address");
        require(sendParams.relayerFee > 0, "Relayer fee must be greater than zero");
        require(sendParams.executionFee > 0, "Execution fee must be greater than zero");

        // Calculate total fees: cross-chain fees + L2 fee
        uint256 sendFeeAmount = sendParams.relayerFee + sendParams.executionFee;
        uint256 totalFeeAmount = sendFeeAmount + sendParams.l2Fee;
        require(
            msg.value >= totalFeeAmount,
            "Sent value must be greater than or equal to total fee amount"
        );

        IERC20(params.inputToken).safeTransferFrom(msg.sender, address(this), params.inputAmount);
        IERC20(params.inputToken).forceApprove(address(SPOKE_POOL), params.inputAmount);
        L2_WETH9.deposit{value: totalFeeAmount}();
        IERC20(address(L2_WETH9)).forceApprove(address(SPOKE_POOL), totalFeeAmount);

        // The first deposit is used to fund the handler contract on Ethereum with WETH,
        // which is then converted to ETH to cover the cross-chain fees from Ethereum to Polkadot
        // for the subsequent cross-chain call.
        Call[] memory calls = new Call[](1);
        calls[0] = Call({
            target: address(L1_WETH9),
            callData: abi.encodeCall(L1_WETH9.withdraw, (sendFeeAmount)),
            value: 0
        });
        Instructions memory instructions =
            Instructions({calls: calls, fallbackRecipient: address(MULTI_CALL_HANDLER)});
        SPOKE_POOL.deposit(
            bytes32(uint256(uint160(recipient))),
            bytes32(uint256(uint160(address(MULTI_CALL_HANDLER)))),
            bytes32(uint256(uint160(address(L2_WETH9)))),
            bytes32(uint256(uint160(address(L1_WETH9)))),
            totalFeeAmount,
            sendFeeAmount,
            params.destinationChainId,
            bytes32(0), // exclusiveRelayer, zero means any relayer can fill
            uint32(block.timestamp), // quoteTimestamp set to current block timestamp
            uint32(block.timestamp + TIME_BUFFER), // fillDeadline set to TIME_BUFFER seconds in the future
            0, // exclusivityDeadline, zero means no exclusivity
            abi.encode(instructions)
        );

        // Second deposit for the actual token swap and cross-chain call to Polkadot
        // For USDT, we need to reset allowance to zero before setting it to a new value
        calls = new Call[](3);
        calls[0] = Call({
            target: address(params.outputToken),
            callData: abi.encodeCall(IERC20.approve, (address(GATEWAY), 0)),
            value: 0
        });
        calls[1] = Call({
            target: address(params.outputToken),
            callData: abi.encodeCall(IERC20.approve, (address(GATEWAY), params.outputAmount)),
            value: 0
        });
        calls[2] = Call({
            target: address(GATEWAY),
            callData: abi.encodeCall(
                IGatewayV2.v2_sendMessage,
                (
                    sendParams.xcm,
                    sendParams.assets,
                    sendParams.claimer,
                    sendParams.executionFee,
                    sendParams.relayerFee
                )
            ),
            value: sendFeeAmount
        });
        instructions = Instructions({calls: calls, fallbackRecipient: recipient});
        SPOKE_POOL.deposit(
            bytes32(uint256(uint160(recipient))),
            bytes32(uint256(uint160(address(MULTI_CALL_HANDLER)))),
            bytes32(uint256(uint160(params.inputToken))),
            bytes32(uint256(uint160(params.outputToken))),
            params.inputAmount,
            params.outputAmount,
            params.destinationChainId,
            bytes32(0),
            uint32(block.timestamp),
            uint32(block.timestamp + TIME_BUFFER),
            0,
            abi.encode(instructions)
        );
        // Refund any excess ETH sent
        if (msg.value > totalFeeAmount) {
            payable(msg.sender).transfer(msg.value - totalFeeAmount);
        }
        // Emit event with the depositId of the second deposit
        uint256 depositId = SPOKE_POOL.numberOfDeposits() - 1;
        emit DepositCallInvoked(topic, depositId);
    }

    // Send native Ether to Polkadot, the fee should be calculated off-chain
    function sendNativeEtherAndCall(
        SwapParams calldata params,
        SendParams calldata sendParams,
        address recipient,
        bytes32 topic
    ) public payable {
        require(
            params.inputToken == address(0),
            "Input token must be zero address for native ETH deposits"
        );
        require(params.inputAmount > 0, "Input amount must be greater than zero");
        require(params.outputAmount > 0, "Output amount must be greater than zero");
        require(recipient != address(0), "Recipient cannot be zero address");
        require(sendParams.relayerFee > 0, "Relayer fee must be greater than zero");
        require(sendParams.executionFee > 0, "Execution fee must be greater than zero");
        require(sendParams.l2Fee > 0, "L2 fee must be greater than zero");
        require(
            params.inputAmount == params.outputAmount + sendParams.l2Fee,
            "Input amount must equal output amount plus L2 fee"
        );
        // Calculate total amount: input ETH + cross-chain relay and execution fees
        uint256 totalAmount = params.inputAmount + sendParams.relayerFee + sendParams.executionFee;
        require(
            msg.value >= totalAmount, "Sent value must be greater than or equal to total amount"
        );

        L2_WETH9.deposit{value: totalAmount}();
        IERC20(address(L2_WETH9)).forceApprove(address(SPOKE_POOL), totalAmount);

        // The deposit is used to fund the handler contract on the destination chain with WETH,
        // which is then converted to ETH to cover the cross-chain fees from Ethereum to Polkadot
        uint256 totalOutputAmount = totalAmount - sendParams.l2Fee;
        Call[] memory calls = new Call[](2);
        calls[0] = Call({
            target: address(L1_WETH9),
            callData: abi.encodeCall(L1_WETH9.withdraw, (totalOutputAmount)),
            value: 0
        });
        calls[1] = Call({
            target: address(GATEWAY),
            callData: abi.encodeCall(
                IGatewayV2.v2_sendMessage,
                (
                    sendParams.xcm,
                    sendParams.assets,
                    sendParams.claimer,
                    sendParams.executionFee,
                    sendParams.relayerFee
                )
            ),
            value: totalOutputAmount
        });
        Instructions memory instructions =
            Instructions({calls: calls, fallbackRecipient: recipient});
        SPOKE_POOL.deposit(
            bytes32(uint256(uint160(recipient))),
            bytes32(uint256(uint160(address(MULTI_CALL_HANDLER)))),
            bytes32(uint256(uint160(address(L2_WETH9)))),
            bytes32(uint256(uint160(address(L1_WETH9)))),
            totalAmount,
            totalOutputAmount,
            params.destinationChainId,
            bytes32(0), // exclusiveRelayer, zero means any relayer can fill
            uint32(block.timestamp), // quoteTimestamp set to current block timestamp
            uint32(block.timestamp + TIME_BUFFER), // fillDeadline set to TIME_BUFFER seconds in the future
            0, // exclusivityDeadline, zero means no exclusivity
            abi.encode(instructions)
        );
        // Refund any excess ETH sent
        if (msg.value > totalAmount) {
            payable(msg.sender).transfer(msg.value - totalAmount);
        }
        uint256 depositId = SPOKE_POOL.numberOfDeposits() - 1;
        emit DepositCallInvoked(topic, depositId);
    }

    receive() external payable {}
}

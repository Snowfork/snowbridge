// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {IERC20} from "openzeppelin/token/ERC20/IERC20.sol";
import {SafeERC20} from "openzeppelin/token/ERC20/utils/SafeERC20.sol";
import {ISpokePool, IMessageHandler} from "./interfaces/ISpokePool.sol";
import {IGatewayV2} from "../../../src/v2/IGateway.sol";
import {SwapParams, Instructions, Call, SendParams} from "./Types.sol";
import {WETH9} from "canonical-weth/WETH9.sol";

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

    event L2CallInvoked(bytes32 topic, uint256 depositId);

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
        uint256 sendFeeAmount =
            sendParams.relayerFee + sendParams.executionFee;
        uint256 totalFeeAmount = sendFeeAmount + sendParams.l2Fee;

        IERC20(params.inputToken).safeTransferFrom(msg.sender, address(this), params.inputAmount);
        IERC20(params.inputToken).approve(address(SPOKE_POOL), params.inputAmount);

        L2_WETH9.deposit{value: totalFeeAmount}();
        IERC20(address(L2_WETH9)).approve(address(SPOKE_POOL), totalFeeAmount);

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
            uint32(block.timestamp - TIME_BUFFER), // quoteTimestamp set to 10 minutes before now
            uint32(block.timestamp + TIME_BUFFER), // fillDeadline set to 10 minutes after now
            0, // exclusivityDeadline, zero means no exclusivity
            abi.encode(instructions)
        );

        // A second deposit for the actual token swap and cross-chain call
        calls = new Call[](2);
        calls[0] = Call({
            target: address(params.outputToken),
            callData: abi.encodeCall(IERC20.approve, (address(GATEWAY), params.outputAmount)),
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
            uint32(block.timestamp - TIME_BUFFER),
            uint32(block.timestamp + TIME_BUFFER),
            0,
            abi.encode(instructions)
        );
        // Emit event with the depositId of the second deposit
        uint256 depositId = SPOKE_POOL.numberOfDeposits() - 1;
        emit L2CallInvoked(topic, depositId);
    }

    // Send native Ether to Polkadot, the fee should be calculated off-chain
    function sendNativeEtherAndCall(
        SwapParams calldata params,
        SendParams calldata sendParams,
        address recipient,
        bytes32 topic
    ) public payable {
        require(params.inputToken == address(0));
        require(
            params.inputAmount == params.outputAmount + sendParams.l2Fee,
            "Input and output amount mismatch"
        );
        uint256 totalAmount = params.inputAmount + sendParams.relayerFee + sendParams.executionFee;
        require(msg.value == totalAmount, "Incorrect ETH amount sent");

        L2_WETH9.deposit{value: totalAmount}();
        IERC20(address(L2_WETH9)).approve(address(SPOKE_POOL), totalAmount);

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
            Instructions({calls: calls, fallbackRecipient: address(MULTI_CALL_HANDLER)});
        SPOKE_POOL.deposit(
            bytes32(uint256(uint160(recipient))),
            bytes32(uint256(uint160(address(MULTI_CALL_HANDLER)))),
            bytes32(uint256(uint160(address(L2_WETH9)))),
            bytes32(uint256(uint160(address(L1_WETH9)))),
            totalAmount,
            totalOutputAmount,
            params.destinationChainId,
            bytes32(0), // exclusiveRelayer, zero means any relayer can fill
            uint32(block.timestamp - TIME_BUFFER), // quoteTimestamp set to 10 minutes before now
            uint32(block.timestamp + TIME_BUFFER), // fillDeadline set to 10 minutes after now
            0, // exclusivityDeadline, zero means no exclusivity
            abi.encode(instructions)
        );
        uint256 depositId = SPOKE_POOL.numberOfDeposits() - 1;
        emit L2CallInvoked(topic, depositId);
    }

    receive() external payable {}
}

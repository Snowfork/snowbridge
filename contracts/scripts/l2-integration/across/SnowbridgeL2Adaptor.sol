// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {IERC20} from "openzeppelin/token/ERC20/IERC20.sol";
import {SafeERC20} from "openzeppelin/token/ERC20/utils/SafeERC20.sol";
import {ISpokePool, IMessageHandler} from "./Interfaces.sol";
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
    uint32 public waitTime;

    constructor(
        address _spokePool,
        address _handler,
        address _gateway,
        address _l1weth9,
        address _l2weth9,
        uint32 _waitTime
    ) {
        SPOKE_POOL = ISpokePool(_spokePool);
        GATEWAY = IGatewayV2(_gateway);
        MULTI_CALL_HANDLER = IMessageHandler(_handler);
        L1_WETH9 = WETH9(payable(_l1weth9));
        L2_WETH9 = WETH9(payable(_l2weth9));
        waitTime = _waitTime;
    }

    // Swap ERC20 token on Sepolia to get other token on L2, the fee should be calculated off-chain
    function swapTokenAndCall(
        SwapParams calldata params,
        SendParams calldata sendParams,
        uint256 nativeFeeAmount,
        address recipient
    ) public {
        uint256 sendFeeAmount =
            sendParams.relayerFee + sendParams.executionFee;
        require(nativeFeeAmount > sendFeeAmount, "Native fee must be greater than send fees");

        IERC20(params.inputToken).safeTransfer(address(this), params.inputAmount);
        payable(address(this)).transfer(nativeFeeAmount);

        IERC20(address(L2_WETH9)).approve(address(SPOKE_POOL), nativeFeeAmount);
        IERC20(params.inputToken).approve(address(SPOKE_POOL), params.inputAmount);

        // A first deposit is used to fund the handler contract on the destination chain to cover cross-chain fees
        // from Ethereum to Polkadot
        Call[] memory calls = new Call[](1);
        calls[0] = Call({
            target: address(L1_WETH9),
            callData: abi.encodeCall(L1_WETH9.withdraw, (sendFeeAmount)),
            value: 0
        });
        Instructions memory instructions =
            Instructions({calls: calls, fallbackRecipient: recipient});
        SPOKE_POOL.deposit{
            value: nativeFeeAmount
        }(
            bytes32(uint256(uint160(recipient))),
            bytes32(uint256(uint160(address(MULTI_CALL_HANDLER)))),
            bytes32(uint256(uint160(address(L2_WETH9)))),
            bytes32(uint256(uint160(address(L1_WETH9)))),
            nativeFeeAmount,
            sendFeeAmount,
            params.destinationChainId,
            bytes32(0), // exclusiveRelayer, zero means any relayer can fill
            uint32(block.timestamp - waitTime), // quoteTimestamp set to 10 minutes before now
            uint32(block.timestamp + waitTime), // fillDeadline set to 10 minutes after now
            0, // exclusivityDeadline, zero means no exclusivity
            abi.encode(instructions)
        );

        // A second deposit for the actual token swap and cross-chain call
        calls = new Call[](1);
        calls[0] = Call({
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
        instructions = Instructions({calls: calls, fallbackRecipient: address(MULTI_CALL_HANDLER)});
        SPOKE_POOL.deposit(
            bytes32(uint256(uint160(recipient))),
            bytes32(uint256(uint160(address(MULTI_CALL_HANDLER)))),
            bytes32(uint256(uint160(params.inputToken))),
            bytes32(uint256(uint160(params.outputToken))),
            params.inputAmount,
            params.outputAmount,
            params.destinationChainId,
            bytes32(0),
            uint32(block.timestamp - waitTime),
            uint32(block.timestamp + waitTime),
            0,
            abi.encode(instructions)
        );
    }

    /// @dev Agents can receive ether permissionlessly.
    /// This is important, as agents are used to lock ether.
    receive() external payable {}
}

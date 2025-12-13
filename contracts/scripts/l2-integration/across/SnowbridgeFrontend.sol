// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {IERC20} from "openzeppelin/token/ERC20/IERC20.sol";
import {SafeERC20} from "openzeppelin/token/ERC20/utils/SafeERC20.sol";
import {Ownable} from "openzeppelin/access/Ownable.sol";
import {ISpokePool, IMessageHandler} from "./Interfaces.sol";
import {IGatewayV2} from "../../../src/v2/IGateway.sol";
import {SwapParams, Instructions, Call, SendParams} from "./Types.sol";

contract SnowbridgeFrontend is Ownable {
    using SafeERC20 for IERC20;

    ISpokePool public immutable SPOKE_POOL;
    IGatewayV2 public immutable GATEWAY;
    address public immutable HANDLER;
    address public immutable L1WETH9;
    address public immutable L2WETH9;

    constructor(
        address _spokePool,
        address _handler,
        address _gateway,
        address _l1weth9,
        address _l2weth9
    ) Ownable() {
        SPOKE_POOL = ISpokePool(_spokePool);
        GATEWAY = IGatewayV2(_gateway);
        HANDLER = _handler;
        L1WETH9 = _l1weth9;
        L2WETH9 = _l2weth9;
    }

    // Swap ERC20 token on Sepolia to get other token on L2, the fee should be calculated off-chain
    function swapTokenAndCall(
        SwapParams calldata params,
        SendParams calldata sendParams,
        uint256 nativeFeeAmount,
        address recipient
    ) public {
        // Fund the native fee to the remote HANDLER which will pay the relayer and execution fees and call
        // Snowbridge Gateway to execute the message
        require(
            nativeFeeAmount > sendParams.relayerFee + sendParams.executionFee,
            "Native fee must be greater than send fees"
        );
        IERC20(L2WETH9).approve(address(SPOKE_POOL), nativeFeeAmount);
        SPOKE_POOL.deposit(
            bytes32(uint256(uint160(recipient))),
            bytes32(uint256(uint160(HANDLER))),
            bytes32(uint256(uint160(L2WETH9))),
            bytes32(uint256(uint160(L1WETH9))),
            nativeFeeAmount,
            sendParams.relayerFee + sendParams.executionFee,
            params.destinationChainId,
            bytes32(0), // exclusiveRelayer, zero means any relayer can fill
            uint32(block.timestamp - 600), // quoteTimestamp set to 10 minutes before now
            uint32(block.timestamp + 600), // fillDeadline set to 10 minutes after now
            0, // exclusivityDeadline, zero means no exclusivity
            bytes("")
        );

        // Approve input token to SpokePool and prepare message for cross-chain call
        IERC20(params.inputToken).approve(address(SPOKE_POOL), params.inputAmount);
        Call[] memory calls = new Call[](1);
        bytes memory sendMessageCalldata = abi.encodeCall(
            IGatewayV2.v2_sendMessage,
            (
                sendParams.xcm,
                sendParams.assets,
                sendParams.claimer,
                sendParams.executionFee,
                sendParams.relayerFee
            )
        );
        calls[0] = Call({
            target: address(GATEWAY),
            callData: sendMessageCalldata,
            value: sendParams.executionFee + sendParams.relayerFee
        });
        Instructions memory instructions =
            Instructions({calls: calls, fallbackRecipient: recipient});
        bytes memory message = abi.encode(instructions);
        SPOKE_POOL.deposit(
            bytes32(uint256(uint160(recipient))),
            bytes32(uint256(uint160(HANDLER))),
            bytes32(uint256(uint160(params.inputToken))),
            bytes32(uint256(uint160(params.outputToken))),
            params.inputAmount,
            params.outputAmount,
            params.destinationChainId,
            bytes32(0),
            uint32(block.timestamp - 600),
            uint32(block.timestamp + 600),
            0,
            message
        );
    }
}

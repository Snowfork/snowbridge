// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {IERC20} from "openzeppelin/token/ERC20/IERC20.sol";
import {SafeERC20} from "openzeppelin/token/ERC20/utils/SafeERC20.sol";
import {Ownable} from "openzeppelin/access/Ownable.sol";
import {ISpokePool, IMessageHandler} from "./Interfaces.sol";
import {IGatewayV2} from "../../../src/v2/IGateway.sol";
import {SwapParams, Instructions, Call, SendParams} from "./Types.sol";
import {WETH9} from "canonical-weth/WETH9.sol";

contract SnowbridgeFrontend is Ownable {
    using SafeERC20 for IERC20;

    ISpokePool public immutable SPOKE_POOL;
    IGatewayV2 public immutable GATEWAY;
    address public immutable HANDLER;
    WETH9 public immutable L1WETH9;
    WETH9 public immutable L2WETH9;

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
        L1WETH9 = WETH9(payable(_l1weth9));
        L2WETH9 = WETH9(payable(_l2weth9));
    }

    // Swap ERC20 token on Sepolia to get other token on L2, the fee should be calculated off-chain
    function swapTokenAndCall(
        SwapParams calldata params,
        SendParams calldata sendParams,
        uint256 nativeFeeAmount,
        address recipient
    ) public {
        require(
            nativeFeeAmount > sendParams.relayerFee + sendParams.executionFee,
            "Native fee must be greater than send fees"
        );
        IERC20(address(L2WETH9)).approve(address(SPOKE_POOL), nativeFeeAmount);
        IERC20(params.inputToken).approve(address(SPOKE_POOL), params.inputAmount);

        // A first deposit for the token swap on the destination chain
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
            bytes("")
        );

        // A second deposit to cover the cross-chain v2_sendMessage call along with fees
        Call[] memory calls = new Call[](2);
        uint256 outputNativeFeeAmount = sendParams.relayerFee + sendParams.executionFee;
        calls[0] = Call({
            target: address(L1WETH9),
            callData: abi.encodeCall(L1WETH9.withdraw, (outputNativeFeeAmount)),
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
            value: outputNativeFeeAmount
        });
        Instructions memory instructions =
            Instructions({calls: calls, fallbackRecipient: recipient});
        SPOKE_POOL.deposit{
            value: nativeFeeAmount
        }(
            bytes32(uint256(uint160(recipient))),
            bytes32(uint256(uint160(HANDLER))),
            bytes32(uint256(uint160(address(L2WETH9)))),
            bytes32(uint256(uint160(address(L1WETH9)))),
            nativeFeeAmount,
            outputNativeFeeAmount,
            params.destinationChainId,
            bytes32(0), // exclusiveRelayer, zero means any relayer can fill
            uint32(block.timestamp - 600), // quoteTimestamp set to 10 minutes before now
            uint32(block.timestamp + 600), // fillDeadline set to 10 minutes after now
            0, // exclusivityDeadline, zero means no exclusivity
            abi.encode(instructions)
        );
    }

    /// @dev Agents can receive ether permissionlessly.
    /// This is important, as agents are used to lock ether.
    receive() external payable {}
}

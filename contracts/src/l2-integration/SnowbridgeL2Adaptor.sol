// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.34;

import {IERC20} from "openzeppelin/token/ERC20/IERC20.sol";
import {SafeERC20} from "openzeppelin/token/ERC20/utils/SafeERC20.sol";
import {WETH9} from "canonical-weth/WETH9.sol";
import {ISpokePool, IMessageHandler} from "./interfaces/ISpokePool.sol";
import {IGatewayV2} from "../v2/IGateway.sol";
import {DepositParams, Instructions, Call, SendParams, SwapParams} from "./Types.sol";

/// @title L2 Across Adaptor for Snowbridge V2 inbound flows
///
/// @dev End users on the L2 call `sendTokenAndCall` and `sendEtherAndCall` directly to
///      bridge ERC20 / native ETH back to Ethereum and onward to Polkadot. These
///      functions pull funds from the caller in the same call — via
///      `safeTransferFrom(msg.sender, address(this), inputAmount)` for ERC20 or
///      `require(msg.value == inputAmount)` for native ETH — and forward the pulled
///      amount straight to the SpokePool deposit. No function in this contract moves
///      a pre-existing contract-held balance to a user-controlled destination: any
///      assets accidentally sent to the adaptor (via `receive()` or stray ERC20
///      transfers) are not reachable by any public entry point.
///
///      The public, unauthenticated entry points are therefore safe — the caller can
///      only spend their own funds; pre-existing balances cannot be swept.
///
///      Recipient requirements: `recipient` is the fallback address used in two places.
///      If the paired L1 `CallContract` fails on mainnet, any trapped funds are swept
///      back to `recipient` on Ethereum. If the Across fees are not profitable and no
///      relayer fills the deposit, the SpokePool refunds the assets to `recipient` on
///      the originating L2. It is typically an EOA. If it is a contract, it MUST be
///      able to receive the relevant assets (native ETH and/or the input ERC20) on
///      BOTH the L2 and Ethereum mainnet, since the same address is used on both
///      chains. Integrating UIs should surface this requirement to users.
///
///      Bug bounty note: reports claiming that "any EOA can call these functions and
///      steal funds" are out of scope. The contract holds no balance on behalf of other
///      users; each call transacts exclusively with `msg.sender`'s assets.
contract SnowbridgeL2Adaptor {
    using SafeERC20 for IERC20;

    ISpokePool public immutable SPOKE_POOL;
    IMessageHandler public immutable MULTI_CALL_HANDLER;
    IGatewayV2 public immutable GATEWAY;
    WETH9 public immutable L1_WETH9;
    WETH9 public immutable L2_WETH9;

    /**************************************
     *              EVENTS                *
     **************************************/

    event DepositCallInvoked(bytes32 topic, uint256 depositId);

    constructor(
        address _spokePool,
        address _handler,
        address _gateway,
        address _l1weth9,
        address _l2weth9
    ) {
        SPOKE_POOL = ISpokePool(_spokePool);
        GATEWAY = IGatewayV2(_gateway);
        MULTI_CALL_HANDLER = IMessageHandler(_handler);
        L1_WETH9 = WETH9(payable(_l1weth9));
        L2_WETH9 = WETH9(payable(_l2weth9));
    }

    // Send ERC20 token to Polkadot, the fee should be calculated off-chain
    function sendTokenAndCall(
        DepositParams calldata params,
        SwapParams calldata swapParams,
        SendParams calldata sendParams,
        address recipient,
        bytes32 topic
    ) public {
        require(params.inputToken != address(0), "Input token cannot be zero address");
        checkInputsWithSwapParams(params, swapParams, sendParams, recipient);
        require(
            params.inputAmount > params.outputAmount + swapParams.inputAmount,
            "Input amount must cover output amount and fee amount"
        );

        IERC20(params.inputToken).safeTransferFrom(msg.sender, address(this), params.inputAmount);
        IERC20(params.inputToken).forceApprove(address(SPOKE_POOL), params.inputAmount);

        // deposit: token swap and cross-chain call
        uint256 depositId = _depositTokenAndSendMessage(params, swapParams, sendParams, recipient);

        // Emit event with the depositId of the second deposit
        emit DepositCallInvoked(topic, depositId);
    }

    // Send native Ether or WETH to Polkadot, the fee should be calculated off-chain
    function sendEtherAndCall(
        DepositParams calldata params,
        SendParams calldata sendParams,
        address recipient,
        bytes32 topic
    ) public payable {
        require(
            params.inputToken == address(0) || params.inputToken == address(L2_WETH9),
            "Input token must be zero address or L2 WETH address for native ETH deposits"
        );
        checkInputs(params, sendParams, recipient);
        uint256 totalOutputAmount =
            params.outputAmount + sendParams.executionFee + sendParams.relayerFee;
        require(
            params.inputAmount > totalOutputAmount,
            "Input amount must cover output amount and fee amount"
        );
        if (params.inputToken == address(0)) {
            // Deposit native ETH
            require(
                msg.value == params.inputAmount,
                "Sent value must be greater than or equal to total amount"
            );
            L2_WETH9.deposit{value: params.inputAmount}();
        } else {
            // Deposit WETH
            IERC20(address(L2_WETH9))
                .safeTransferFrom(msg.sender, address(this), params.inputAmount);
        }

        IERC20(address(L2_WETH9)).forceApprove(address(SPOKE_POOL), params.inputAmount);

        // deposit: WETH and cross-chain call
        uint256 depositId =
            _depositEtherAndSendMessage(params, sendParams, recipient, totalOutputAmount);

        emit DepositCallInvoked(topic, depositId);
    }

    function _depositTokenAndSendMessage(
        DepositParams calldata params,
        SwapParams calldata swapParams,
        SendParams calldata sendParams,
        address recipient
    ) internal returns (uint256 depositId) {
        uint32 fillDeadline = uint32(block.timestamp + params.fillDeadlineBuffer);
        Call[] memory calls = new Call[](7);
        calls[0] = Call({
            target: address(params.outputToken),
            callData: abi.encodeCall(IERC20.approve, (address(swapParams.router), 0)),
            value: 0
        });
        calls[1] = Call({
            target: address(params.outputToken),
            callData: abi.encodeCall(
                IERC20.approve, (address(swapParams.router), swapParams.inputAmount)
            ),
            value: 0
        });
        calls[2] =
            Call({target: address(swapParams.router), callData: swapParams.callData, value: 0});
        calls[3] = Call({
            target: address(L1_WETH9),
            callData: abi.encodeCall(
                L1_WETH9.withdraw, (sendParams.relayerFee + sendParams.executionFee)
            ),
            value: 0
        });
        calls[4] = Call({
            target: address(params.outputToken),
            callData: abi.encodeCall(IERC20.approve, (address(GATEWAY), 0)),
            value: 0
        });
        calls[5] = Call({
            target: address(params.outputToken),
            callData: abi.encodeCall(IERC20.approve, (address(GATEWAY), params.outputAmount)),
            value: 0
        });
        calls[6] = Call({
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
            value: sendParams.relayerFee + sendParams.executionFee
        });
        Instructions memory instructions =
            Instructions({calls: calls, fallbackRecipient: recipient});

        SPOKE_POOL.deposit(
            bytes32(uint256(uint160(recipient))),
            bytes32(uint256(uint160(address(MULTI_CALL_HANDLER)))),
            bytes32(uint256(uint160(params.inputToken))),
            bytes32(uint256(uint160(params.outputToken))),
            params.inputAmount,
            params.outputAmount + swapParams.inputAmount,
            params.destinationChainId,
            bytes32(0), // exclusiveRelayer, zero means any relayer can fill
            uint32(block.timestamp), // quoteTimestamp set to current block timestamp
            fillDeadline, // fillDeadline set to fillDeadlineBuffer seconds in the future
            0, // exclusivityDeadline, zero means no exclusivity
            abi.encode(instructions)
        );
        depositId = SPOKE_POOL.numberOfDeposits() - 1;
    }

    function _depositEtherAndSendMessage(
        DepositParams calldata params,
        SendParams calldata sendParams,
        address recipient,
        uint256 totalOutputAmount
    ) internal returns (uint256 depositId) {
        // The deposit is used to fund the handler contract on the destination chain with WETH,
        // which is then converted to ETH to cover the cross-chain fees from Ethereum to Polkadot
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
        uint32 fillDeadline = uint32(block.timestamp + params.fillDeadlineBuffer);
        uint256 destinationChainId = params.destinationChainId;
        SPOKE_POOL.deposit(
            bytes32(uint256(uint160(recipient))),
            bytes32(uint256(uint160(address(MULTI_CALL_HANDLER)))),
            bytes32(uint256(uint160(address(L2_WETH9)))),
            bytes32(uint256(uint160(address(L1_WETH9)))),
            params.inputAmount,
            totalOutputAmount,
            destinationChainId,
            bytes32(0), // exclusiveRelayer, zero means any relayer can fill
            uint32(block.timestamp), // quoteTimestamp set to current block timestamp
            fillDeadline, // fillDeadline set to fillDeadlineBuffer seconds in the future
            0, // exclusivityDeadline, zero means no exclusivity
            abi.encode(instructions)
        );
        depositId = SPOKE_POOL.numberOfDeposits() - 1;
    }

    function checkInputs(
        DepositParams calldata params,
        SendParams calldata sendParams,
        address recipient
    ) internal pure {
        require(params.inputAmount > 0, "Input amount must be greater than zero");
        require(params.outputAmount > 0, "Output amount must be greater than zero");
        require(sendParams.relayerFee > 0, "Relayer fee must be greater than zero");
        require(sendParams.executionFee > 0, "Execution fee must be greater than zero");
        require(params.fillDeadlineBuffer > 0, "Fill deadline buffer must be greater than zero");
        require(params.destinationChainId != 0, "Destination chain ID cannot be zero");
        require(recipient != address(0), "Recipient cannot be zero address");
    }

    function checkInputsWithSwapParams(
        DepositParams calldata params,
        SwapParams calldata swapParams,
        SendParams calldata sendParams,
        address recipient
    ) internal pure {
        checkInputs(params, sendParams, recipient);
        require(swapParams.inputAmount > 0, "Input amount for fee must be greater than zero");
        require(swapParams.router != address(0), "Swap router cannot be zero address");
        require(swapParams.callData.length > 0, "Swap callData cannot be empty");
    }

    receive() external payable {}
}

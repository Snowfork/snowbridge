// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import {IERC20} from "openzeppelin/token/ERC20/IERC20.sol";
import {SafeERC20} from "openzeppelin/token/ERC20/utils/SafeERC20.sol";
import {WETH9} from "canonical-weth/WETH9.sol";
import {ISpokePool, IMessageHandler} from "./interfaces/ISpokePool.sol";
import {DepositParams, Instructions, Call} from "./Types.sol";

contract SnowbridgeL1Adaptor {
    using SafeERC20 for IERC20;
    ISpokePool public immutable SPOKE_POOL;
    IMessageHandler public immutable MULTI_CALL_HANDLER;
    WETH9 public immutable L1_WETH9;
    WETH9 public immutable L2_WETH9;

    struct DepositCallParams {
        bytes32 depositor;
        bytes32 recipient;
        bytes32 inputToken;
        bytes32 outputToken;
        uint256 inputAmount;
        uint256 outputAmount;
        uint256 destinationChainId;
        uint32 quoteTimestamp;
        uint32 fillDeadline;
        bytes message;
    }

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

    // Send ERC20 token or native Ether on L1 to L2, the fee (params.inputAmount - params.outputAmount) should be calculated off-chain
    // following https://docs.across.to/reference/api-reference#get-swap-approval
    // The function assumes that tokens have been pre-funded and transferred to this contract via Snowbridge unlock prior to invocation.
    // For native ETH deposits, set params.inputToken to address(0).
    function depositToken(DepositParams calldata params, address recipient, bytes32 topic)
        public
        payable
    {
        checkInputs(params, recipient);

        DepositCallParams memory depositCallParams;

        if (params.inputToken == address(0)) {
            // Native ETH deposit: wrap to WETH9 and execute instructions on L2 to unwrap and send ETH
            bytes memory message = _encodeNativeEtherInstructions(recipient, params.outputAmount);
            depositCallParams = DepositCallParams({
                depositor: bytes32(uint256(uint160(recipient))),
                recipient: bytes32(uint256(uint160(address(MULTI_CALL_HANDLER)))),
                inputToken: bytes32(uint256(uint160(address(L1_WETH9)))),
                outputToken: bytes32(uint256(uint160(address(L2_WETH9)))),
                inputAmount: params.inputAmount,
                outputAmount: params.outputAmount,
                destinationChainId: params.destinationChainId,
                quoteTimestamp: uint32(block.timestamp),
                fillDeadline: uint32(block.timestamp + params.fillDeadlineBuffer),
                message: message
            });
        } else {
            // ERC20 token deposit
            depositCallParams = DepositCallParams({
                depositor: bytes32(uint256(uint160(recipient))),
                recipient: bytes32(uint256(uint160(recipient))),
                inputToken: bytes32(uint256(uint160(params.inputToken))),
                outputToken: bytes32(uint256(uint160(params.outputToken))),
                inputAmount: params.inputAmount,
                outputAmount: params.outputAmount,
                destinationChainId: params.destinationChainId,
                quoteTimestamp: uint32(block.timestamp),
                fillDeadline: uint32(block.timestamp + params.fillDeadlineBuffer),
                message: ""
            });
        }

        bytes memory encodedCall = _encodeDepositCall(depositCallParams);
        (bool success,) = address(SPOKE_POOL).delegatecall(encodedCall);
        require(success, "Deposit failed");
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

    function _encodeNativeEtherInstructions(address recipient, uint256 outputAmount)
        internal
        view
        returns (bytes memory)
    {
        Call[] memory calls = new Call[](2);
        calls[0] = Call({
            target: address(L2_WETH9),
            callData: abi.encodeCall(L2_WETH9.withdraw, (outputAmount)),
            value: 0
        });
        calls[1] = Call({target: recipient, callData: "", value: outputAmount});
        Instructions memory instructions =
            Instructions({calls: calls, fallbackRecipient: recipient});
        return abi.encode(instructions);
    }

    function _encodeDepositCall(DepositCallParams memory p) internal pure returns (bytes memory) {
        return abi.encodeWithSelector(
            ISpokePool.deposit.selector,
            p.depositor,
            p.recipient,
            p.inputToken,
            p.outputToken,
            p.inputAmount,
            p.outputAmount,
            p.destinationChainId,
            bytes32(0), // exclusiveRelayer
            p.quoteTimestamp,
            p.fillDeadline,
            0, // exclusivityDeadline
            p.message
        );
    }

    receive() external payable {}
}

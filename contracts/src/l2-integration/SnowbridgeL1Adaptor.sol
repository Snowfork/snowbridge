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
    event DepositCallFailed(bytes32 topic);

    constructor(address _spokePool, address _handler, address _l1weth9, address _l2weth9) {
        SPOKE_POOL = ISpokePool(_spokePool);
        L1_WETH9 = WETH9(payable(_l1weth9));
        L2_WETH9 = WETH9(payable(_l2weth9));
    }

    // Send ERC20 token on L1 to L2, the fee (params.inputAmount - params.outputAmount) should be calculated off-chain
    // following https://docs.across.to/reference/api-reference#get-swap-approval
    // The call requires pre-funding of the contract with the input tokens.
    function depositToken(DepositParams calldata params, address recipient, bytes32 topic) public {
        require(params.inputToken != address(0), "Input token cannot be zero address");
        checkInputs(params, recipient);
        IERC20(params.inputToken).forceApprove(address(SPOKE_POOL), params.inputAmount);

        // Build payload and perform low-level call
        bytes memory payloadToken = _encodeDepositTokenPayload(params, recipient);
        (bool depositSucceeded,) = address(SPOKE_POOL).call(payloadToken);

        // Always forward any remaining balance of the input token back to the recipient to avoid trapping funds
        uint256 remaining = IERC20(params.inputToken).balanceOf(address(this));
        if (remaining > 0) {
            IERC20(params.inputToken).safeTransfer(recipient, remaining);
        }

        if (depositSucceeded) {
            uint256 depositId = SPOKE_POOL.numberOfDeposits() - 1;
            emit DepositCallInvoked(topic, depositId);
        } else {
            emit DepositCallFailed(topic);
        }
    }

    // Send native Ether on L1 to L2. Native ETH is sent with the transaction via msg.value;
    // The call requires pre-funding of the contract with native Ether.
    function depositNativeEther(DepositParams calldata params, address recipient, bytes32 topic)
        public
    {
        require(
            params.inputToken == address(0),
            "Input token must be zero address for native ETH deposits"
        );
        checkInputs(params, recipient);

        bytes memory payloadNative = _encodeDepositNativePayload(params, recipient);
        (bool depositSucceeded,) =
            address(SPOKE_POOL).call{value: params.inputAmount}(payloadNative);

        // Always forward any remaining balance of native Ether back to the recipient to avoid trapping funds
        uint256 remaining = address(this).balance;
        if (remaining > 0) {
            (bool success,) = payable(recipient).call{value: remaining}("");
            require(success, "Failed to transfer remaining ether to recipient");
        }

        if (depositSucceeded) {
            uint256 depositId = SPOKE_POOL.numberOfDeposits() - 1;
            emit DepositCallInvoked(topic, depositId);
        } else {
            emit DepositCallFailed(topic);
        }
    }

    function checkInputs(DepositParams calldata params, address recipient) internal pure {
        require(params.inputAmount > 0, "Input amount must be greater than zero");
        require(params.outputAmount > 0, "Output amount must be greater than zero");
        require(params.outputAmount <= params.inputAmount, "Output amount exceeds input amount");
        require(params.fillDeadlineBuffer > 0, "Fill deadline buffer must be greater than zero");
        require(params.destinationChainId != 0, "Destination chain ID cannot be zero");
        require(recipient != address(0), "Recipient cannot be zero address");
    }

    // Build payload for token deposit call
    function _encodeDepositTokenPayload(DepositParams calldata params, address recipient)
        internal
        view
        returns (bytes memory)
    {
        return abi.encodeWithSelector(
            ISpokePool.deposit.selector,
            bytes32(uint256(uint160(recipient))),
            bytes32(uint256(uint160(recipient))),
            bytes32(uint256(uint160(params.inputToken))),
            bytes32(uint256(uint160(params.outputToken))),
            params.inputAmount,
            params.outputAmount,
            params.destinationChainId,
            bytes32(0),
            uint32(block.timestamp),
            uint32(block.timestamp + params.fillDeadlineBuffer),
            0,
            bytes("")
        );
    }

    // Build payload for native deposit call
    function _encodeDepositNativePayload(DepositParams calldata params, address recipient)
        internal
        view
        returns (bytes memory)
    {
        return abi.encodeWithSelector(
            ISpokePool.deposit.selector,
            bytes32(uint256(uint160(recipient))),
            bytes32(uint256(uint160(address(recipient)))),
            bytes32(uint256(uint160(address(L1_WETH9)))),
            bytes32(uint256(uint160(address(L2_WETH9)))),
            params.inputAmount,
            params.outputAmount,
            params.destinationChainId,
            bytes32(0),
            uint32(block.timestamp),
            uint32(block.timestamp + params.fillDeadlineBuffer),
            0,
            bytes("")
        );
    }

    receive() external payable {}
}

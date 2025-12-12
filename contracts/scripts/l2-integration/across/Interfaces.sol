// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

interface ISpokePool {
    function deposit(
        bytes32 depositor,
        bytes32 recipient,
        bytes32 inputToken,
        bytes32 outputToken,
        uint256 inputAmount,
        uint256 outputAmount,
        uint256 destinationChainId,
        bytes32 exclusiveRelayer,
        uint32 quoteTimestamp,
        uint32 fillDeadline,
        uint32 exclusivityDeadline,
        bytes calldata message
    ) external payable;
}

interface IMessageHandler {
    function handleV3AcrossMessage(
        address tokenSent,
        uint256 amount,
        address relayer,
        bytes memory message
    ) external;
}

struct Call {
    address target;
    bytes callData;
    uint256 value;
}

struct Instructions {
    // Calls that will be attempted
    Call[] calls;
    // Where the tokens go if any part of the call fails
    // Leftover tokens are sent here as well if the action succeeds
    address fallbackRecipient;
}

struct SwapParams {
    address inputToken;
    address outputToken;
    uint256 inputAmount;
    uint256 outputAmount;
    uint256 destinationChainId;
}

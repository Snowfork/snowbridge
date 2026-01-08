// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

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
    uint32 fillDeadlineBuffer;
}

struct SendParams {
    bytes xcm;
    bytes[] assets;
    bytes claimer;
    uint128 executionFee;
    uint128 relayerFee;
    // Fee to be paid on L2 in native token
    uint128 l2Fee;
}

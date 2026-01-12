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

// Parameters for performing a swap
struct SwapParams {
    //input token address on source chain
    address inputToken;
    //output token address on destination chain
    address outputToken;
    // amount for input token to swap
    uint256 inputAmount;
    // amount for output token to receive
    uint256 outputAmount;
    // destination chain id
    uint256 destinationChainId;
    // fill deadline buffer in seconds
    uint32 fillDeadlineBuffer;
}

// The first five parameters correspond to the same parameters in Gateway.sol's v2_sendMessage function. See Gateway.sol for more details.
// The final parameter is specific to the seperated call used to prefund both the execution fee and the relayer fee.
struct SendParams {
    bytes xcm;
    bytes[] assets;
    bytes claimer;
    uint128 executionFee;
    uint128 relayerFee;
    // Fees are paid on L2 in the native token
    uint128 l2Fee;
}

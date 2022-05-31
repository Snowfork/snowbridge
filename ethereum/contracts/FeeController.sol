// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

// Something that can burn a fee from a feepayer account.
interface FeeController {
    // NOTE: Should never revert or the bridge will be broken
    function handleFee(address feePayer, uint256 _amount) external;
}

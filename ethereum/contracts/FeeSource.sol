// SPDX-License-Identifier: MIT
pragma solidity >=0.7.6;
pragma experimental ABIEncoderV2;

// Something that can burn a fee from a feepayer account.
interface FeeSource {
    function burnFee(address feePayer, uint256 _amount) external;
}

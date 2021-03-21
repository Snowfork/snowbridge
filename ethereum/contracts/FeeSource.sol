// SPDX-License-Identifier: MIT
pragma solidity >=0.7.6;
pragma experimental ABIEncoderV2;

interface FeeSource {
    function burnFee(address feePayer, uint256 _amount) external;
}

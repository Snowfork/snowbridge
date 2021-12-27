// SPDX-License-Identifier: Apache-2.0
pragma solidity >=0.7.6;
pragma experimental ABIEncoderV2;

import "../FeeSource.sol";

contract MockFeeSource is FeeSource {
    function burnFee(address, uint256 _amount) pure external override {
        // Simulate the case where the user has no funds,
        require(_amount != 1024, "User has no funds to burn");
    }
}

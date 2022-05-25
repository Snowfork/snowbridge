// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "../FeeSource.sol";

contract MockFeeSource is FeeSource {
    function burnFee(address, uint256 _amount) pure external override {
        // Simulate the case where the user has no funds,
        require(_amount != 1024, "User has no funds to burn");
    }
}

// SPDX-License-Identifier: Apache-2.0
pragma solidity >=0.7.6;
pragma experimental ABIEncoderV2;

contract MockApp {

    event Unlocked(uint256 amount);

    function unlock(uint256 _amount) public {
        emit Unlocked(_amount);
    }

}

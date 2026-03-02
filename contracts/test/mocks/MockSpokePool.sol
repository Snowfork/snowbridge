// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.33;

import {ISpokePool} from "../../src/l2-integration/interfaces/ISpokePool.sol";

contract MockSpokePool is ISpokePool {
    uint256 private _numberOfDeposits;

    function deposit(
        bytes32,
        bytes32,
        bytes32,
        bytes32,
        uint256,
        uint256,
        uint256,
        bytes32,
        uint32,
        uint32,
        uint32,
        bytes calldata
    ) external payable override {
        _numberOfDeposits++;
    }

    function numberOfDeposits() external view override returns (uint256) {
        return _numberOfDeposits;
    }
}

contract MockSpokePoolReverting is ISpokePool {
    function deposit(
        bytes32,
        bytes32,
        bytes32,
        bytes32,
        uint256,
        uint256,
        uint256,
        bytes32,
        uint32,
        uint32,
        uint32,
        bytes calldata
    ) external payable override {
        revert("MockSpokePool: deposit reverted");
    }

    function numberOfDeposits() external view override returns (uint256) {
        return 0;
    }
}

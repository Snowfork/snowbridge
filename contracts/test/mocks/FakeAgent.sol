// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.33;

import {SnowbridgeL1Adaptor} from "../../src/l2-integration/SnowbridgeL1Adaptor.sol";
import {DepositParams} from "../../src/l2-integration/Types.sol";
import {IERC20} from "openzeppelin/token/ERC20/IERC20.sol";

/// @dev A contract that impersonates a Gateway Agent by exposing a GATEWAY() getter
/// returning the real gateway address, without actually being registered in the Gateway.
contract FakeAgent {
    address public immutable GATEWAY;

    constructor(address _gateway) {
        GATEWAY = _gateway;
    }

    function callDepositToken(
        SnowbridgeL1Adaptor adaptor,
        DepositParams calldata params,
        address recipient,
        bytes32 topic
    ) external {
        adaptor.depositToken(params, recipient, topic);
    }

    function callDepositNativeEther(
        SnowbridgeL1Adaptor adaptor,
        DepositParams calldata params,
        address recipient,
        bytes32 topic
    ) external {
        adaptor.depositNativeEther(params, recipient, topic);
    }

    receive() external payable {}
}

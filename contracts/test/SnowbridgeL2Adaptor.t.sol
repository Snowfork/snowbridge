// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.34;

import {Test} from "forge-std/Test.sol";
import {SnowbridgeL2Adaptor} from "../src/l2-integration/SnowbridgeL2Adaptor.sol";
import {SendParams} from "../src/l2-integration/Types.sol";

contract SnowbridgeL2AdaptorMock is SnowbridgeL2Adaptor {
    constructor() SnowbridgeL2Adaptor(address(1), address(2), address(3), address(4), address(5)) {}

    function gatewayValue(SendParams calldata sendParams) external pure returns (uint256) {
        return _gatewayValue(sendParams);
    }
}

contract SnowbridgeL2AdaptorTest is Test {
    SnowbridgeL2AdaptorMock mock;

    function setUp() public {
        mock = new SnowbridgeL2AdaptorMock();
    }

    function testGatewayValue() public {
        SendParams memory params = SendParams({
            xcm: "",
            assets: new bytes[](0),
            claimer: "",
            executionFee: 100,
            relayerFee: 200,
            destinationExecutionFee: 300
        });

        assertEq(mock.gatewayValue(params), 600);
    }

    function testFuzzGatewayValue(uint128 executionFee, uint128 relayerFee, uint128 destinationExecutionFee) public {
        SendParams memory params = SendParams({
            xcm: "",
            assets: new bytes[](0),
            claimer: "",
            executionFee: executionFee,
            relayerFee: relayerFee,
            destinationExecutionFee: destinationExecutionFee
        });

        uint256 expectedValue = uint256(executionFee) + uint256(relayerFee) + uint256(destinationExecutionFee);
        assertEq(mock.gatewayValue(params), expectedValue);
    }
}

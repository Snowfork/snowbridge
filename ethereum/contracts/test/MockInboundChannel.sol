// SPDX-License-Identifier: Apache-2.0
pragma solidity >=0.7.6;
pragma experimental ABIEncoderV2;


contract MockInboundChannel {

    function submitToApp(address app, bytes calldata data) public returns (bool) {
        (bool success, ) = app.call(data);
        return success;
    }

}

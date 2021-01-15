// SPDX-License-Identifier: MIT
pragma solidity >=0.6.2;
pragma experimental ABIEncoderV2;

// Application contains methods that all applications must implement
abstract contract Application {
    /**
     * @dev Handles a SCALE encoded message from the Bridge
     */
    function handle(bytes memory _data) public virtual;

    /**
     * @dev Registers the Bridge contract on the application
     */
    function register(address _bridge) public virtual;
}

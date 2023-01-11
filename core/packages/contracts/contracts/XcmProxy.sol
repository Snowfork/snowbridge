// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "@openzeppelin/contracts/access/Ownable.sol";

/// @title XcmProxy
/// @notice A simple pass through XcmProxy.
contract XcmProxy is Ownable {
    /// @dev Calls into the XCM executor
    /// @param executor The address of the XCM executor.
    /// @param encodedCall The encoded call to execute the xcm message.
    /// @return bool than indicates success of the call.
    function execute(
        address executor,
        bytes calldata encodedCall
    ) external onlyOwner returns (bool) {
        (bool success, ) = executor.delegatecall(encodedCall);
        return success;
    }
}

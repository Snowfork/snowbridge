// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "@openzeppelin/contracts/access/Ownable.sol";

import "./XcmAssetLookup.sol";

/// @title XcmProxy
/// @notice A simple pass through XcmProxy.
contract XcmProxy is Ownable {
    /// @dev Calls into the XCM executor
    /// @param _executor The address of the XCM executor.
    /// @param _lookup The lookup used to resolve assets.
    /// @param _payload The XCM payload.
    /// @return bool than indicates success of the call.
    function execute(
        address _executor,
        XcmAssetLookup _lookup,
        bytes calldata _payload
    ) external onlyOwner returns (bool) {
        (bool success, ) = _executor.delegatecall(_payload);
        return success;
    }
}

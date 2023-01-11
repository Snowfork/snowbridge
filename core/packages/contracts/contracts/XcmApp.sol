// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "./XcmProxy.sol";
import "./XcmAssetManager.sol";

/// @title Xcm App
/// @notice Implements XCM for the EVM.
contract XcmApp is Ownable {
    /// @dev Asset look up.
    XcmAssetLookup public immutable assetLookup;

    /// @dev A mapping or 32 byte hashed origins to proxy accounts.
    mapping(bytes32 => XcmProxy) public proxies;

    /// @dev Emitted when a proxy is successfully dispatched too.
    /// @param origin The hashed origin.
    /// @param proxy The proxy account.
    /// @param executor The address of the executor.
    /// @param success The dispatch was successful.
    event XcmExecuted(
        bytes32 origin,
        XcmProxy proxy,
        address executor,
        bool success,
        // TODO: Remove debug data
        bytes debug4
    );

    /// @dev Called from an unauthorized sender.
    error Unauthorized();

    /// @param lookup Where to look up xcm assets.
    constructor(XcmAssetLookup lookup) {
        assetLookup = lookup;
    }

    /// @dev The signature of the xcm execution function.
    bytes4 private constant EXEC_XCM_FUNC = bytes4(keccak256("execute(address,(uint8,bytes)[])"));

    /// @dev Looks up the proxy and executor and executes the payload.
    /// @param origin The hashed origin.
    /// @param executor The identifier for the executor version.
    /// @param instructions The XCM payload to be executed.
    function dispatchToProxy(
        bytes32 origin,
        address executor,
        bytes calldata instructions
    ) external onlyOwner {
        XcmProxy proxy = proxies[origin];
        // JIT create proxy if it does not exist.
        if (proxy == XcmProxy(address(0))) {
            proxy = new XcmProxy();
            proxies[origin] = proxy;
        }

        // encode a call to the xcm executor
        bytes memory encodedCall = bytes.concat(
            EXEC_XCM_FUNC,
            abi.encode(assetLookup),
            instructions
        );

        // Dispatch to proxy.
        bool success = proxy.execute(executor, encodedCall);
        emit XcmExecuted(origin, proxy, executor, success, encodedCall);
    }
}

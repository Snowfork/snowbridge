// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "./ChannelRegistry.sol";
import "./XcmProxy.sol";
import "./XcmAssetManager.sol";

/// @title Xcm App
/// @notice Implements XCM for the EVM.
contract XcmApp {
    /// @dev Channels which are allowed to call this app.
    ChannelRegistry public immutable registry;

    /// @dev Asset look up.
    XcmAssetLookup public immutable assetLookup;

    /// @dev A mapping or 32 byte hashed origins to proxy accounts.
    mapping(bytes32 => XcmProxy) public proxies;

    /// @dev Emitted when a proxy is successfully dispatched too.
    /// @param origin The hashed origin.
    /// @param proxy The proxy account.
    /// @param executor The address of the executor.
    /// @param success The dispatch was successful.
    event XcmExecuted(bytes32 origin, XcmProxy proxy, address executor, bool success);

    /// @dev Called from an unauthorized sender.
    error Unauthorized();

    /// @param _registry The channel registry which is the source of messages.
    constructor(ChannelRegistry _registry, XcmAssetLookup _assetLookup) {
        registry = _registry;
        assetLookup = _assetLookup;
    }

    /// @dev Looks up the proxy and executor and executes the payload.
    /// @param _origin The hashed origin.
    /// @param _executor The identifier for the executor version.
    /// @param _payload The XCM payload to be executed.
    function dispatchToProxy(bytes32 _origin, address _executor, bytes calldata _payload) external {
        // TODO: Should permissionless channels be able to call in here???
        if (!registry.isInboundChannel(msg.sender)) {
            revert Unauthorized();
        }

        XcmProxy proxy = proxies[_origin];
        // JIT create proxy if it does not exist.
        if (proxy == XcmProxy(address(0))) {
            proxy = new XcmProxy();
            proxies[_origin] = proxy;
        }

        // Dispatch to proxy.
        bool success = proxy.execute(_executor, assetLookup, _payload);
        emit XcmExecuted(_origin, proxy, _executor, success);
    }
}

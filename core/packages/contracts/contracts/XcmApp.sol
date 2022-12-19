// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "./ChannelRegistry.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

// TODO: put encoding functions here
//import "./XCMAppPallet.sol";

/// @title Proxy
/// @notice A simple pass through proxy.
contract Proxy is Ownable {
    /// @dev
    /// @param _executor The address of the XCM executor.
    /// @param _payload The XCM payload.
    /// @return bool than indicates success of the call.
    function execute(address _executor, bytes calldata _payload) external onlyOwner returns(bool) { 
        (bool success, ) = _executor.delegatecall(
            _payload
        );
        return success;
    }
}

/// @title Xcm App
/// @notice Implements XCM for the EVM.
contract XcmApp {
    /// @dev Channels which are allowed to call this app.
    ChannelRegistry public immutable registry;

    /// @dev A mapping or 32 byte hashed origins to proxy accounts.
    mapping(bytes32 => Proxy) public proxies;

    /// @dev A list of approved xcm executors.
    mapping(bytes8 => address) public executors;

    /// @dev Emitted when a proxy is successfully dispatched too.
    /// @param origin The hashed origin.
    /// @param proxy The proxy account.
    /// @param executor The address of the executor.
    /// @param success The dispatch was successful.
    event XcmExecuted(bytes32 origin, Proxy proxy, address executor, bool success);

    /// @dev Called from an unauthorized sender.
    error Unauthorized();

    /// @dev The executor already exists.
    error ExecutorExists();

    /// @dev The executor was not found.
    error ExecutorNotFound();

    /// @param _registry The channel registry which is the source of messages.
    constructor(ChannelRegistry _registry) {
        registry = _registry;
    }

    /// @dev Approves an executor version.
    /// @param _version An identifier for the version.
    /// @param _executor The executor to approve.
    function approveExecutor(bytes8 _version, address _executor) external {
        // TODO: Should permissionless channels be able to call in here???
        if (!registry.isInboundChannel(msg.sender)) {
            revert Unauthorized();
        }
        if(executors[_version] != address(0)) { revert ExecutorExists(); }
        executors[_version] = _executor;
    }

    /// @dev Revokes an executor version.
    /// @param _version An identifier for the version.
    function revokeExecutor(bytes8 _version) external {
        // TODO: Should permissionless channels be able to call in here???
        if (!registry.isInboundChannel(msg.sender)) {
            revert Unauthorized();
        }
        if(executors[_version] == address(0)) { revert ExecutorNotFound(); }
        delete executors[_version];
    }

    /// @dev Looks up the proxy and executor and executes the payload.
    /// @param _origin The hashed origin.
    /// @param _executor The identifier for the executor version.
    /// @param _payload The XCM payload to be executed.
    function dispatchToProxy(bytes32 _origin, bytes8 _executor, bytes  calldata _payload) external {
        // TODO: Should permissionless channels be able to call in here???
        if (!registry.isInboundChannel(msg.sender)) {
            revert Unauthorized();
        }

        Proxy proxy = proxies[_origin];
        // JIT create proxy if it does not exist.
        if(proxy == Proxy(address(0))) {
            proxy = new Proxy();
            proxies[_origin] = proxy;
        }

        // Find an approved executor.
        address executor = executors[_executor];
        if(executor == address(0)) {
            revert ExecutorNotFound();
        }
        
        // Dispatch to proxy.
        bool success = proxy.execute(executor, _payload);
        emit XcmExecuted(_origin, proxy, executor, success);
    }
}

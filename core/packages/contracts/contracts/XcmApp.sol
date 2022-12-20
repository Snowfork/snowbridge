// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "./ChannelRegistry.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

// TODO: put encoding functions here
//import "./XCMAppPallet.sol";

/// @title Proxy
/// @notice A simple pass through proxy.
contract Proxy is Ownable {
    /// @dev Calls into the XCM executor
    /// @param _executor The address of the XCM executor.
    /// @param _payload The XCM payload.
    /// @return bool than indicates success of the call.
    function execute(address _executor, bytes calldata _payload) external onlyOwner returns (bool) {
        (bool success, ) = _executor.delegatecall(_payload);
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

    /// @dev Emitted when a proxy is successfully dispatched too.
    /// @param origin The hashed origin.
    /// @param proxy The proxy account.
    /// @param executor The address of the executor.
    /// @param success The dispatch was successful.
    event XcmExecuted(bytes32 origin, Proxy proxy, address executor, bool success);

    /// @dev Called from an unauthorized sender.
    error Unauthorized();

    /// @param _registry The channel registry which is the source of messages.
    constructor(ChannelRegistry _registry) {
        registry = _registry;
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

        Proxy proxy = proxies[_origin];
        // JIT create proxy if it does not exist.
        if (proxy == Proxy(address(0))) {
            proxy = new Proxy();
            proxies[_origin] = proxy;
        }

        // Dispatch to proxy.
        bool success = proxy.execute(_executor, _payload);
        emit XcmExecuted(_origin, proxy, _executor, success);
    }
}

/// @dev Executes Xcm instructions.
contract XcmExecutor {
    /// @dev Represents the type of instruction.
    enum InstructionKind {
        /// @dev Transact allows abritrary call to another contract.
        Transact
    }

    /// @dev
    struct Instruction {
        /// @dev the type of instruction.
        InstructionKind kind;
        /// @dev the data provided for execution.
        bytes arguments;
    }

    /// @dev Data needed for xcm Transact.
    struct TransactData {
        /// @dev The contract to call.
        address target;
        /// @dev The abi encoded payload with function selector.
        bytes payload;
    }

    /// @dev The entry point for an payload.
    function execute(Instruction[] calldata instructions) external {
        // TODO: registers like origin, holding, etc...
        for (uint i = 0; i < instructions.length; i++) {
            if (instructions[i].kind == InstructionKind.Transact) {
                // 0x00 = Transact
                transact(abi.decode(instructions[i].arguments, (TransactData)));
            } else {
                revert("Unknown instruction");
            }
        }
    }

    /// @dev single transact instruction.
    function transact(TransactData memory data) internal {
        (bool success, ) = data.target.call(data.payload);
        require(success, "Transact failed");
    }
}

/// @dev test app
contract DownstreamTestApp {
    event RecordSender(address sender);

    function doSomethingInteresting() external {
        emit RecordSender(msg.sender);
    }
}

/// @title Library for implementing reantrancy guards.
library ReentrantGuard {
    // Storage slot for the reantrancy guard derived by:
    // keccak256(abi.encode(uint256(keccak256("snowbridge.ReentrancyGuard")) - 1)) & ~bytes32(uint256(0xff))
    bytes32 private constant REENTRANCY_GUARD_STORAGE =
        0x2ba401b34c12923a4850086f5807d30f124be8a0a38cff4286ca87f38422f200;

    /// @dev checks if the reentrancy guard is set and reverts if so, else sets the guard.
    function checkAndSet() internal {
        assembly {
            // Check if flag is set and if true revert because it means the function is currently executing.
            if tload(REENTRANCY_GUARD_STORAGE) { revert(0, 0) }

            // Set the flag to mark the the function is currently executing.
            tstore(REENTRANCY_GUARD_STORAGE, 1)
        }
    }

    /// @dev clears the reentrancy guard.
    function clear() internal {
        assembly {
            // Clear the flag as the function has completed execution.
            tstore(REENTRANCY_GUARD_STORAGE, 0)
        }
    }
}

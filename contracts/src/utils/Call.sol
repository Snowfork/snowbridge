// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: 2023 OpenZeppelin
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.25;

// Derived from OpenZeppelin Contracts (last updated v4.9.0) (utils/Address.sol)
library Call {
    function verifyResult(bool success, bytes memory returndata)
        internal
        pure
        returns (bytes memory)
    {
        if (success) {
            return returndata;
        } else {
            // Look for revert reason and bubble it up if present
            if (returndata.length > 0) {
                // The easiest way to bubble the revert reason is using memory via assembly
                /// @solidity memory-safe-assembly
                assembly {
                    let returndata_size := mload(returndata)
                    revert(add(32, returndata), returndata_size)
                }
            } else {
                revert();
            }
        }
    }

    /**
     * @notice Safely perform a low level call without copying any returndata
     *
     * @param target   Address to call
     * @param data Calldata to pass to the call
     */
    function safeCall(address target, bytes memory data, uint256 value) internal returns (bool) {
        bool success;
        assembly {
            success :=
                call(
                    gas(), // gas
                    target, // recipient
                    value, // ether value
                    add(data, 0x20), // inloc
                    mload(data), // inlen
                    0, // outloc
                    0 // outlen
                )
        }
        return success;
    }
}

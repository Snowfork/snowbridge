// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: 2023 OpenZeppelin
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.23;

// Derived from OpenZeppelin Contracts (last updated v4.9.0) (utils/Address.sol)
library Call {
    function verifyResult(bool success, bytes memory returndata) internal pure returns (bytes memory) {
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

    /// @notice Perform a low level call without copying any returndata
    /// @param _target   Address to call
    /// @param _gas      Amount of gas to pass to the call
    /// @param _value    Amount of value to pass to the call
    /// @param _calldata Calldata to pass to the call
    function safeCall(address _target, uint256 _gas, uint256 _value, bytes memory _calldata) internal returns (bool) {
        bool _success;
        assembly {
            _success :=
                call(
                    _gas, // gas
                    _target, // recipient
                    _value, // ether value
                    add(_calldata, 32), // inloc
                    mload(_calldata), // inlen
                    0, // outloc
                    0 // outlen
                )
        }
        return _success;
    }
}

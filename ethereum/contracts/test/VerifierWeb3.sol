// SPDX-License-Identifier: MIT
pragma solidity >=0.6.2;

contract VerifierWeb3 {

    address public operator;

    constructor(address _operator) public {
        operator = _operator;
    }

    function recover(bytes32 _message, bytes memory _signature)
        public
        view
        returns (bool)
    {
        address signer = verify(ethMessageHash(_message), _signature);
        return operator == signer;
    }

  /**
     * @dev Verify checks if the signer's address matches the operator's address
     */
    function verify(bytes32 h, bytes memory signature)
        internal
        pure
        returns (address)
    {
        bytes32 r;
        bytes32 s;
        uint8 v;

        // Check the signature length
        if (signature.length != 65) {
            return (address(0));
        }

        // Divide the signature in r, s and v variables
        // ecrecover takes the signature parameters, and the only way to get them
        // currently is to use assembly.
        // solium-disable-next-line security/no-inline-assembly
        assembly {
            r := mload(add(signature, 32))
            s := mload(add(signature, 64))
            v := byte(0, mload(add(signature, 96)))
        }

        // Version of signature should be 27 or 28, but 0 and 1 are also possible versions
        if (v < 27) {
            v += 27;
        }

        // If the version is correct return the signer address
        if (v != 27 && v != 28) {
            return (address(0));
        } else {
            // solium-disable-next-line arg-overflow
            return ecrecover(h, v, r, s);
        }
    }

    /**
     * toEthSignedMessageHash
     * @dev prefix a bytes32 value with "\x19Ethereum Signed Message:"
     * and hash the result
     */
    function ethMessageHash(bytes32 message) internal pure returns (bytes32) {
        return
            keccak256(
                abi.encodePacked("\x19Ethereum Signed Message:\n32", message)
            );
    }
}

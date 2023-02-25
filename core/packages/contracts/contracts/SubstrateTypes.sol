// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "./ScaleCodec.sol";

/**
 * @title SCALE encoders for common Substrate types
 */
library SubstrateTypes {
    /**
     * @dev Encodes `MultiAddress::Id`: https://crates.parity.io/sp_runtime/enum.MultiAddress.html#variant.Id
     * @return bytes SCALE-encoded bytes
     */
    // solhint-disable-next-line func-name-mixedcase
    function MultiAddressWithID(bytes32 account) internal pure returns (bytes memory) {
        return bytes.concat(hex"00", account);
    }

    /**
     * @dev Encodes `H160`: https://crates.parity.io/sp_core/struct.H160.html
     * @return bytes SCALE-encoded bytes
     */
    // solhint-disable-next-line func-name-mixedcase
    function H160(address account) internal pure returns (bytes memory) {
        return abi.encodePacked(account);
    }

    /**
     * @dev Encodes `Option::None`: https://doc.rust-lang.org/std/option/enum.Option.html#variant.None
     * @return bytes SCALE-encoded bytes
     */
    // solhint-disable-next-line func-name-mixedcase
    function None() internal pure returns (bytes memory) {
        return hex"00";
    }

    function Bytes(bytes input) internal pure returns (bytes memory) {
        return bytes.concat(ScaleCodec.encodeCompactUint(input.length), input);
    }

    /**
     * @dev Encodes Action::NativeTokens(NativeTokens::Create)
     */
    function NativeTokensCreate(
        bytes memory dest,
        address token,
        bytes memory name,
        bytes memory symbol,
        uint8 decimals
    ) internal pure {
        return
            bytes.concat(
                hex"00",
                hex"00",
                dest,
                abi.encodePacked(token),
                Bytes(name),
                Bytes(symbol),
                decimals
            );
    }
}

// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "./ScaleCodec.sol";
import "./SubstrateTypes.sol";

library NativeTokensTypes {
    /**
     * @dev Encodes Action::NativeTokens(NativeTokens::Mint)
     */
    // solhint-disable-next-line func-name-mixedcase
    function Mint(
        bytes memory dest,
        address token,
        bytes memory recipient,
        uint128 amount
    ) internal pure returns (bytes memory) {
        return
            bytes.concat(
                dest,
                hex"00",
                hex"00",
                abi.encodePacked(token),
                recipient,
                ScaleCodec.encodeU128(amount),
                SubstrateTypes.None()
            );
    }

    /**
     * @dev Encodes Action::NativeTokens(NativeTokens::Create)
     */
    // solhint-disable-next-line func-name-mixedcase
    function Create(
        bytes memory dest,
        address token,
        bytes memory name,
        bytes memory symbol,
        uint8 decimals
    ) internal pure returns (bytes memory) {
        return
            bytes.concat(
                dest,
                hex"00",
                hex"00",
                abi.encodePacked(token),
                SubstrateTypes.VecU8(name),
                SubstrateTypes.VecU8(symbol),
                ScaleCodec.encodeU8(decimals)
            );
    }
}

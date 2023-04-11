// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import {ScaleCodec} from "./ScaleCodec.sol";
import {SubstrateTypes} from "./SubstrateTypes.sol";
import {ParaID} from "./Types.sol";

library NativeTokensTypes {
    /**
     * @dev Encodes Payload::NativeTokens(NativeTokens::Create)
     */
    // solhint-disable-next-line func-name-mixedcase
    function Create(address token, bytes memory name, bytes memory symbol, uint8 decimals)
        internal
        pure
        returns (bytes memory)
    {
        return bytes.concat(
            hex"00",
            hex"00",
            abi.encodePacked(token),
            SubstrateTypes.VecU8(name),
            SubstrateTypes.VecU8(symbol),
            ScaleCodec.encodeU8(decimals)
        );
    }

    /**
     * @dev Encodes Payload::NativeTokens(NativeTokens::Mint)
     */
    // solhint-disable-next-line func-name-mixedcase
    function Mint(address token, ParaID dest, bytes memory recipient, uint128 amount)
        internal
        pure
        returns (bytes memory)
    {
        return bytes.concat(
            hex"00",
            hex"01",
            abi.encodePacked(token),
            SubstrateTypes.OptionParaID(dest),
            recipient,
            ScaleCodec.encodeU128(amount)
        );
    }
}

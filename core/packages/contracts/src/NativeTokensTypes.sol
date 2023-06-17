// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.20;

import {ScaleCodec} from "./ScaleCodec.sol";
import {SubstrateTypes} from "./SubstrateTypes.sol";
import {ParaID} from "./Types.sol";

library NativeTokensTypes {
    /**
     * @dev SCALE-encodes `router_primitives::inbound::VersionedMessage` containing payload
     * `NativeTokensMessage::Create`
     */
    // solhint-disable-next-line func-name-mixedcase
    function Create(
        address origin,
        address token,
        bytes memory name,
        bytes memory symbol,
        uint8 decimals,
        bytes2 createCallIndex,
        bytes2 setMetadataCallIndex
    ) internal view returns (bytes memory) {
        return bytes.concat(
            bytes1(0x00),
            ScaleCodec.encodeU64(uint64(block.chainid)),
            bytes1(0x01),
            bytes1(0x00),
            SubstrateTypes.H160(origin),
            SubstrateTypes.H160(token),
            SubstrateTypes.VecU8(name),
            SubstrateTypes.VecU8(symbol),
            ScaleCodec.encodeU8(decimals),
            createCallIndex,
            setMetadataCallIndex
        );
    }

    /**
     * @dev SCALE-encodes `router_primitives::inbound::VersionedMessage` containing payload
     * `NativeTokensMessage::Mint`
     */
    // solhint-disable-next-line func-name-mixedcase
    function Mint(address origin, address token, ParaID dest, bytes memory recipient, uint128 amount)
        internal
        view
        returns (bytes memory)
    {
        return bytes.concat(
            bytes1(0x00),
            ScaleCodec.encodeU64(uint64(block.chainid)),
            bytes1(0x01),
            bytes1(0x01),
            SubstrateTypes.H160(origin),
            SubstrateTypes.H160(token),
            SubstrateTypes.OptionParaID(dest),
            recipient,
            ScaleCodec.encodeU128(amount)
        );
    }
}

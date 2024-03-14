// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.23;

import {ScaleCodec} from "./utils/ScaleCodec.sol";
import {ParaID} from "./Types.sol";

/**
 * @title SCALE encoders for common Substrate types
 */
library SubstrateTypes {
    error UnsupportedCompactEncoding();

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

    function VecU8(bytes memory input) internal pure returns (bytes memory) {
        return bytes.concat(ScaleCodec.checkedEncodeCompactU32(input.length), input);
    }

    /**
     * @dev Encodes `Option::None`: https://doc.rust-lang.org/std/option/enum.Option.html#variant.None
     * @return bytes SCALE-encoded bytes
     */
    // solhint-disable-next-line func-name-mixedcase
    function None() internal pure returns (bytes memory) {
        return hex"00";
    }

    // solhint-disable-next-line func-name-mixedcase
    function OptionParaID(ParaID v) internal pure returns (bytes memory) {
        if (ParaID.unwrap(v) == 0) {
            return hex"00";
        } else {
            return bytes.concat(bytes1(0x01), ScaleCodec.encodeU32(uint32(ParaID.unwrap(v))));
        }
    }

    /**
     * @dev SCALE-encodes `router_primitives::inbound::VersionedMessage` containing payload
     * `NativeTokensMessage::Create`
     */
    // solhint-disable-next-line func-name-mixedcase
    function RegisterToken(address token, uint128 fee) internal view returns (bytes memory) {
        return bytes.concat(
            bytes1(0x00),
            ScaleCodec.encodeU64(uint64(block.chainid)),
            bytes1(0x00),
            SubstrateTypes.H160(token),
            ScaleCodec.encodeU128(fee)
        );
    }

    /**
     * @dev SCALE-encodes `router_primitives::inbound::VersionedMessage` containing payload
     * `NativeTokensMessage::Mint`
     */
    // destination is AccountID32 address on AssetHub
    function SendTokenToAssetHubAddress32(
        address token,
        address sender,
        bytes32 recipient,
        uint128 xcmFee,
        uint128 amount
    ) internal view returns (bytes memory) {
        return bytes.concat(
            bytes1(0x00),
            ScaleCodec.encodeU64(uint64(block.chainid)),
            bytes1(0x01),
            SubstrateTypes.H160(token),
            SubstrateTypes.H160(sender),
            bytes1(0x00),
            recipient,
            ScaleCodec.encodeU128(amount),
            ScaleCodec.encodeU128(xcmFee)
        );
    }

    // destination is AccountID32 address
    function SendTokenToAddress32(
        address token,
        address sender,
        ParaID paraID,
        bytes32 recipient,
        uint128 xcmFee,
        uint128 destinationXcmFee,
        uint128 amount
    ) internal view returns (bytes memory) {
        return bytes.concat(
            bytes1(0x00),
            ScaleCodec.encodeU64(uint64(block.chainid)),
            bytes1(0x01),
            SubstrateTypes.H160(token),
            SubstrateTypes.H160(sender),
            bytes1(0x01),
            ScaleCodec.encodeU32(uint32(ParaID.unwrap(paraID))),
            recipient,
            ScaleCodec.encodeU128(destinationXcmFee),
            ScaleCodec.encodeU128(amount),
            ScaleCodec.encodeU128(xcmFee)
        );
    }

    // destination is AccountID20 address
    function SendTokenToAddress20(
        address token,
        address sender,
        ParaID paraID,
        bytes20 recipient,
        uint128 xcmFee,
        uint128 destinationXcmFee,
        uint128 amount
    ) internal view returns (bytes memory) {
        return bytes.concat(
            bytes1(0x00),
            ScaleCodec.encodeU64(uint64(block.chainid)),
            bytes1(0x01),
            SubstrateTypes.H160(token),
            SubstrateTypes.H160(sender),
            bytes1(0x02),
            ScaleCodec.encodeU32(uint32(ParaID.unwrap(paraID))),
            recipient,
            ScaleCodec.encodeU128(destinationXcmFee),
            ScaleCodec.encodeU128(amount),
            ScaleCodec.encodeU128(xcmFee)
        );
    }

    // destination is AccountID32 address on AssetHub
    function ClaimTokenToAssetHubAddress32(
        address token,
        address sender,
        bytes32 recipient,
        uint128 xcmFee,
        uint128 amount,
        uint128 feeAmount
    ) internal view returns (bytes memory) {
        return bytes.concat(
            bytes1(0x00),
            ScaleCodec.encodeU64(uint64(block.chainid)),
            bytes1(0x02),
            SubstrateTypes.H160(token),
            SubstrateTypes.H160(sender),
            encodeRecipientOnAssetHub(recipient),
            ScaleCodec.encodeU128(amount),
            ScaleCodec.encodeU128(feeAmount),
            ScaleCodec.encodeU128(xcmFee)
        );
    }

    // destination is AccountID32 address
    function ClaimTokenToAddress32(
        address token,
        address sender,
        ParaID paraID,
        bytes32 recipient,
        uint128 xcmFee,
        uint128 destinationXcmFee,
        uint128 amount,
        uint128 feeAmount
    ) internal view returns (bytes memory) {
        return bytes.concat(
            bytes1(0x00),
            ScaleCodec.encodeU64(uint64(block.chainid)),
            bytes1(0x02),
            SubstrateTypes.H160(token),
            SubstrateTypes.H160(sender),
            encodeRecipientToAddress32OnForeignChain(paraID, recipient),
            ScaleCodec.encodeU128(destinationXcmFee),
            ScaleCodec.encodeU128(amount),
            ScaleCodec.encodeU128(feeAmount),
            ScaleCodec.encodeU128(xcmFee)
        );
    }

    // destination is AccountID20 address
    function ClaimTokenToAddress20(
        address token,
        address sender,
        ParaID paraID,
        bytes20 recipient,
        uint128 xcmFee,
        uint128 destinationXcmFee,
        uint128 amount,
        uint128 feeAmount
    ) internal view returns (bytes memory) {
        return bytes.concat(
            bytes1(0x00),
            ScaleCodec.encodeU64(uint64(block.chainid)),
            bytes1(0x02),
            SubstrateTypes.H160(token),
            SubstrateTypes.H160(sender),
            encodeRecipientToAddress20OnForeignChain(paraID, recipient),
            ScaleCodec.encodeU128(destinationXcmFee),
            ScaleCodec.encodeU128(amount),
            ScaleCodec.encodeU128(xcmFee),
            ScaleCodec.encodeU128(feeAmount)
        );
    }

    function encodeRecipientOnAssetHub(bytes32 recipient) internal pure returns (bytes memory) {
        return bytes.concat(bytes1(0x00), recipient);
    }

    function encodeRecipientToAddress32OnForeignChain(ParaID paraID, bytes32 recipient)
        internal
        pure
        returns (bytes memory)
    {
        return bytes.concat(bytes1(0x01), ScaleCodec.encodeU32(uint32(ParaID.unwrap(paraID))), recipient);
    }

    function encodeRecipientToAddress20OnForeignChain(ParaID paraID, bytes20 recipient)
        internal
        pure
        returns (bytes memory)
    {
        return bytes.concat(bytes1(0x02), ScaleCodec.encodeU32(uint32(ParaID.unwrap(paraID))), recipient);
    }
}

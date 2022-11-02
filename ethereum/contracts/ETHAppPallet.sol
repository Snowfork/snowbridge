// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "./ScaleCodec.sol";
import "./SubstrateTypes.sol";

/**
 * @title SCALE encoders for ETHApp pallet calls
 */
library ETHAppPallet {
    bytes1 public constant PALLET_ID = 0x41;

    bytes1 public constant MINT_CALL = 0x01;
    uint64 public constant MINT_WEIGHT = 100_000_000;

    /**
     * @dev Encode `Pallet::mint`
     * @param sender Sender address
     * @param recipient Recipient address (sr25519)
     * @param amount Amount to mint
     * @return bytes SCALE-encoded call
     * @return uint64 Minimum dispatch weight of pallet call
     */
    function mint(
        address sender,
        bytes32 recipient,
        uint128 amount
    ) internal pure returns (bytes memory, uint64) {
        return (
            bytes.concat(
                PALLET_ID,
                MINT_CALL,
                SubstrateTypes.H160(sender),
                SubstrateTypes.MultiAddressWithID(recipient),
                ScaleCodec.encode128(amount),
                SubstrateTypes.None()
            ),
            MINT_WEIGHT
        );
    }

    /**
     * @dev Encode `Pallet::mint`
     * @param sender Sender address
     * @param recipient Recipient address (sr25519)
     * @param amount Amount to mint
     * @param paraID destination parachain
     * @param fee XCM fees to debit from relayer account
     * @return bytes SCALE-encoded call
     * @return uint64 Minimum dispatch weight of pallet call
     */
    function mintAndForward(
        address sender,
        bytes32 recipient,
        uint128 amount,
        uint32 paraID,
        uint128 fee
    ) internal pure returns (bytes memory, uint64) {
        return (
            bytes.concat(
                PALLET_ID,
                MINT_CALL,
                SubstrateTypes.H160(sender),
                SubstrateTypes.MultiAddressWithID(recipient),
                ScaleCodec.encode128(amount),
                SubstrateTypes.SomeRemotePara(paraID, fee)
            ),
            MINT_WEIGHT
        );
    }
}

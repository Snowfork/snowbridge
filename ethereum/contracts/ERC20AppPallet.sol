// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "./ScaleCodec.sol";
import "./SubstrateTypes.sol";

/**
 * @title SCALE encoders for ERC20App pallet calls
 */
library ERC20AppPallet {
    bytes1 public constant PALLET_ID = 0x42;

    bytes1 public constant MINT_CALL = 0x01;
    uint64 public constant MINT_WEIGHT = 100_000_000;

    bytes1 public constant CREATE_CALL = 0x02;
    uint64 public constant CREATE_WEIGHT = 100_000_000;

    /**
     * @dev Encode `Pallet::mint`
     * @param token Token address
     * @param sender Sender address
     * @param recipient Recipient address (sr25519)
     * @param amount Amount to mint
     * @return bytes SCALE-encoded call
     * @return uint64 Minimum dispatch weight of pallet call
     */
    function mint(
        address token,
        address sender,
        bytes32 recipient,
        uint128 amount
    ) internal pure returns (bytes memory, uint64) {
        return (
            bytes.concat(
                PALLET_ID,
                MINT_CALL,
                SubstrateTypes.H160(token),
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
     * @param token Token address
     * @param sender Sender address
     * @param recipient Recipient address (sr25519)
     * @param amount Amount to mint
     * @param paraID destination parachain
     * @param fee XCM fees to debit from relayer account
     * @return bytes SCALE-encoded call
     * @return uint64 Minimum dispatch weight of pallet call
     */
    function mintAndForward(
        address token,
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
                SubstrateTypes.H160(token),
                SubstrateTypes.H160(sender),
                SubstrateTypes.MultiAddressWithID(recipient),
                ScaleCodec.encode128(amount),
                SubstrateTypes.SomeRemotePara(paraID, fee)
            ),
            MINT_WEIGHT
        );
    }

    /**
     * @dev Encode `Pallet::create`
     * @param token Token address
     * @return bytes SCALE-encoded call
     * @return uint64 Minimum dispatch weight of pallet call
     */
    function create(address token) internal pure returns (bytes memory, uint64) {
        return (bytes.concat(PALLET_ID, CREATE_CALL, SubstrateTypes.H160(token)), CREATE_WEIGHT);
    }
}

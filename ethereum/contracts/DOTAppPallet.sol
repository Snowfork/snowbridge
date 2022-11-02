// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "./ScaleCodec.sol";
import "./SubstrateTypes.sol";

/**
 * @title SCALE encoders for DOTApp pallet calls
 */
library DOTAppPallet {
    bytes1 public constant PALLET_ID = 0x40;

    bytes1 public constant UNLOCK_CALL = 0x01;
    uint64 public constant UNLOCK_WEIGHT = 100_000_000;

    /**
     * @dev Encode `Pallet::unlock`
     * @param sender Sender address
     * @param recipient Recipient address (sr25519)
     * @param amount Amount to unlock
     * @return bytes SCALE-encoded call
     * @return uint64 Minimum dispatch weight of pallet call
     */
    function unlock(
        address sender,
        bytes32 recipient,
        uint256 amount
    ) internal pure returns (bytes memory, uint64) {
        return (
            bytes.concat(
                PALLET_ID,
                UNLOCK_CALL,
                SubstrateTypes.H160(sender),
                SubstrateTypes.MultiAddressWithID(recipient),
                ScaleCodec.encode256(amount)
            ),
            UNLOCK_WEIGHT
        );
    }
}

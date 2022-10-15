// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "./ScaleCodec.sol";

library PalletCalls {
    // Pallet-Call indices
    bytes2 constant DOTAPP_UNLOCK_CALL = 0x4001;
    bytes2 constant ETHERAPP_MINT_CALL = 0x4101;

    function DotApp_unlock(
        address sender,
        bytes32 recipient,
        uint256 amount
    ) internal pure returns (bytes memory) {
        return
            bytes.concat(
                DOTAPP_UNLOCK_CALL, // DotApp.unlock
                abi.encodePacked(sender), // H160
                bytes1(0x00), // MultiAddress::Id
                recipient,
                ScaleCodec.encode256(amount) // U256
            );
    }

    function EtherApp_mint(
        address sender,
        bytes32 recipient,
        uint128 amount,
        uint32 paraID,
        uint128 fee
    ) internal pure returns (bytes memory) {
        bytes memory optionRemotePara;
        if (paraID == 0) {
            // Option::None
            optionRemotePara = bytes.concat(bytes1(0x00));
        } else {
            // Option::Some(RemotePara { para_id: u32, fee: u128 })
            optionRemotePara = bytes.concat(
                bytes1(0x01),
                ScaleCodec.encode32(paraID),
                ScaleCodec.encode128(fee)
            );
        }
        return
            bytes.concat(
                ETHERAPP_MINT_CALL, // EtherApp.mint
                abi.encodePacked(sender), // H160
                bytes1(0x00), // MultiAddress::Id
                recipient,
                ScaleCodec.encode128(amount), // u128
                optionRemotePara
            );
    }
}

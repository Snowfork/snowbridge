// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.7.0;

library ParachainLightClient {
    struct OwnParachainHead {
        bytes32 parentHash;
        uint32 number;
        bytes32 stateRoot;
        bytes32 extrinsicsRoot;
        bytes32 commitment; // TODO check type and position of this element
    }

    struct OwnParachainHeadPartial {
        bytes32 parentHash;
        uint32 number;
        bytes32 stateRoot;
        bytes32 extrinsicsRoot;
    }
}
// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "../ParachainClient.sol";

contract OpaqueProof {
    function dummy(ParachainClient.Proof memory proof) public pure {}
}

// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import {ParachainClient} from "../ParachainClient.sol";

// This contract is a hack which allows us to ABI-encode `ParachainClient.Proof` in the off-chain relayer.
// It is only used to generate client bindings and is never deployed.
contract OpaqueProof {
    // solhint-disable-next-line no-empty-blocks
    function dummy(ParachainClient.Proof memory proof) public pure {}
}

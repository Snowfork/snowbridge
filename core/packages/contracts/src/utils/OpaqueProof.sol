// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.20;

import {ParachainClient} from "../ParachainClient.sol";

// This contract is a hack which allows us to ABI-encode `ParachainClient.Proof` in the off-chain relayer.
// It is only used to generate client bindings and is never deployed.
contract OpaqueProof {
    // solhint-disable-next-line no-empty-blocks
    function dummy(ParachainClient.Proof memory proof) public pure {}
}

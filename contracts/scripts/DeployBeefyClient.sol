// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.23;

import {Script} from "forge-std/Script.sol";
import {BeefyClient} from "../src/BeefyClient.sol";

contract DeployBeefyClient is Script {


    struct Config {
        uint64 startBlock;
        BeefyClient.ValidatorSet current;
        BeefyClient.ValidatorSet next;
        uint256 randaoCommitDelay;
        uint256 randaoCommitExpiration;
        uint256 minimumSignatures;
    }

    function readConfig() internal pure returns (Config memory config) {
        // Checkpoint generated at block 20733663 using the `./beefy-checkpoint.js` script in Polkadot-JS.
        // Block 20733663 is significant as that was when our bridge was initialized on BridgeHub.
        config = Config({
            startBlock: 20733663,
            current: BeefyClient.ValidatorSet({id: 496, length: 297, root: 0xdd04a3a0a4a19180bdae78ecc0c089491d22f5b65b685199d877f20b7fc76434}),
            next: BeefyClient.ValidatorSet({id: 497, length: 297, root: 0xdd04a3a0a4a19180bdae78ecc0c089491d22f5b65b685199d877f20b7fc76434}),
            randaoCommitDelay: 128,
            randaoCommitExpiration: 24,
            minimumSignatures: 17
        });
    }

    function run() public {
        vm.startBroadcast();
        Config memory config = readConfig();

        new BeefyClient(
            config.randaoCommitDelay,
            config.randaoCommitExpiration,
            config.minimumSignatures,
            config.startBlock,
            config.current,
            config.next
        );
    }
}

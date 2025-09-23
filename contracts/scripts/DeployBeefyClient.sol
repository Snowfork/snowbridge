// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.28;

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
        // Checkpoint generated using the script `./beefy-checkpoint.js` script in Polkadot-JS.
        config = Config({
            startBlock: 27895089,
            current: BeefyClient.ValidatorSet({
                id: 3494,
                length: 600,
                root: 0xa9860350770648563c3cc25f2121500db9b858b9fa401d8dfb0ed73f2f1c4ce0
            }),
            next: BeefyClient.ValidatorSet({
                id: 3495,
                length: 600,
                root: 0xa9860350770648563c3cc25f2121500db9b858b9fa401d8dfb0ed73f2f1c4ce0
            }),
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

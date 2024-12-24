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
            startBlock: 21087413,
            current: BeefyClient.ValidatorSet({
                id: 644,
                length: 297,
                root: 0x3db19e57e6a7deaec1204d4fb8295cab4e24f8902f54e70d25f273abfe346ada
            }),
            next: BeefyClient.ValidatorSet({
                id: 645,
                length: 297,
                root: 0x3db19e57e6a7deaec1204d4fb8295cab4e24f8902f54e70d25f273abfe346ada
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

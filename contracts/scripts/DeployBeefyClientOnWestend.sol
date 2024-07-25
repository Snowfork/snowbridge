// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.25;

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
            startBlock: 21828775,
            current: BeefyClient.ValidatorSet({
                id: 6055,
                length: 18,
                root: 0xe04fb483cd0f7af795836fc1883c7b20a3e5e2a618b2248ce3a378901ef0e262
            }),
            next: BeefyClient.ValidatorSet({
                id: 6056,
                length: 18,
                root: 0xe04fb483cd0f7af795836fc1883c7b20a3e5e2a618b2248ce3a378901ef0e262
            }),
            randaoCommitDelay: 0,
            randaoCommitExpiration: 1024,
            minimumSignatures: 12
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

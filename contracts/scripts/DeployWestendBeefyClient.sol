// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.28;

import {Script} from "forge-std/Script.sol";
import {BeefyClient} from "../src/BeefyClient.sol";

contract DeployWestendBeefyClient is Script {
    struct Config {
        uint64 startBlock;
        BeefyClient.ValidatorSet current;
        BeefyClient.ValidatorSet next;
        uint256 randaoCommitDelay;
        uint256 randaoCommitExpiration;
        uint256 minimumSignatures;
        uint256 fiatShamirRequiredSignatures;
    }

    function readConfig() internal pure returns (Config memory config) {
        // Checkpoint generated using the script `./beefy-checkpoint.js` script in Polkadot-JS.
        config = Config({
            startBlock: 29_265_008,
            current: BeefyClient.ValidatorSet({
                id: 18_823,
                length: 20,
                root: 0xff1d13b4dc453f2f88261fbc1ec53922bce47d740489c9022bed06f345395f8c
            }),
            next: BeefyClient.ValidatorSet({
                id: 18_824,
                length: 20,
                root: 0xff1d13b4dc453f2f88261fbc1ec53922bce47d740489c9022bed06f345395f8c
            }),
            randaoCommitDelay: 0,
            randaoCommitExpiration: 1024,
            minimumSignatures: 12,
            fiatShamirRequiredSignatures: 101
        });
    }

    function run() public {
        vm.startBroadcast();
        Config memory config = readConfig();

        new BeefyClient(
            config.randaoCommitDelay,
            config.randaoCommitExpiration,
            config.minimumSignatures,
            config.fiatShamirRequiredSignatures,
            config.startBlock,
            config.current,
            config.next
        );
    }
}

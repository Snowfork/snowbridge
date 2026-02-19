// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.33;

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
        uint256 fiatShamirRequiredSignatures;
    }

    function readConfig() internal view returns (Config memory config) {
        // Checkpoint generated using the script `./beefy-checkpoint.js` script in Polkadot-JS.
        if (
            keccak256(abi.encodePacked(vm.envString("NODE_ENV")))
                == keccak256(abi.encodePacked("polkadot_mainnet"))
        ) {
            return mainnetConfig();
        } else if (
            keccak256(abi.encodePacked(vm.envString("NODE_ENV")))
                == keccak256(abi.encodePacked("westend_sepolia"))
        ) {
            return westendConfig();
        }
    }

    function mainnetConfig() internal pure returns (Config memory config) {
        // Checkpoint generated using the script `./beefy-checkpoint.js` script in Polkadot-JS.
        config = Config({
            startBlock: 27_895_089,
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
            minimumSignatures: 17,
            fiatShamirRequiredSignatures: 101
        });
    }

    function westendConfig() internal pure returns (Config memory config) {
        config = Config({
            startBlock: 29879785,
            current: BeefyClient.ValidatorSet({
                id: 19849,
                length: 20,
                root: 0xff1d13b4dc453f2f88261fbc1ec53922bce47d740489c9022bed06f345395f8c
            }),
            next: BeefyClient.ValidatorSet({
                id: 19850,
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

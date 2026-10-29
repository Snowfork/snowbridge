// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.34;

import {Script} from "forge-std/Script.sol";
import {BeefyClient} from "../src/BeefyClient.sol";
import {console} from "forge-std/console.sol";

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
            keccak256(abi.encodePacked(vm.envString("SNOWBRIDGE_DEPLOY_STAGE")))
                == keccak256(abi.encodePacked("polkadot_mainnet"))
        ) {
            return mainnetConfig();
        } else if (
            keccak256(abi.encodePacked(vm.envString("SNOWBRIDGE_DEPLOY_STAGE")))
                == keccak256(abi.encodePacked("westend_sepolia"))
        ) {
            return westendConfig();
        }
    }

    function mainnetConfig() internal pure returns (Config memory config) {
        config = Config({
            startBlock: 33_643_033,
            current: BeefyClient.ValidatorSet({
                id: 5899,
                length: 600,
                root: 0x0d0fdaefb1c8e7b0ecc2f82742eda356fbaada01190650ea0552fa2fe4185e5f
            }),
            next: BeefyClient.ValidatorSet({
                id: 5900,
                length: 600,
                root: 0x0d0fdaefb1c8e7b0ecc2f82742eda356fbaada01190650ea0552fa2fe4185e5f
            }),
            randaoCommitDelay: 128,
            randaoCommitExpiration: 12,
            minimumSignatures: 20,
            fiatShamirRequiredSignatures: 103
        });
    }

    function westendConfig() internal pure returns (Config memory config) {
        config = Config({
            startBlock: 30_057_085,
            current: BeefyClient.ValidatorSet({
                id: 20_144,
                length: 20,
                root: 0xff1d13b4dc453f2f88261fbc1ec53922bce47d740489c9022bed06f345395f8c
            }),
            next: BeefyClient.ValidatorSet({
                id: 20_145,
                length: 20,
                root: 0xff1d13b4dc453f2f88261fbc1ec53922bce47d740489c9022bed06f345395f8c
            }),
            randaoCommitDelay: 0,
            randaoCommitExpiration: 1024,
            minimumSignatures: 12,
            fiatShamirRequiredSignatures: 111
        });
    }

    function run() public {
        vm.startBroadcast();
        Config memory config = readConfig();

        BeefyClient client = new BeefyClient(
            config.randaoCommitDelay,
            config.randaoCommitExpiration,
            config.minimumSignatures,
            config.fiatShamirRequiredSignatures,
            config.startBlock,
            config.current,
            config.next
        );

        console.log("BeefyClient deployed at:", address(client));
    }
}

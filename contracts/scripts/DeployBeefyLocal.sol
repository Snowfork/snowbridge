// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.25;

import {Script, console2} from "forge-std/Script.sol";
import {stdJson} from "forge-std/StdJson.sol";

import {BeefyClient} from "../src/BeefyClient.sol";
import {HelperConfig} from "./HelperConfig.sol";

contract DeployBeefyClient is Script {
    using stdJson for string;

    function setUp() public {}

    function run() public {
        HelperConfig helperConfig = new HelperConfig("");
        HelperConfig.BeefyClientConfig memory beefyClientConfig = helperConfig.getBeefyClientConfig();

        vm.startBroadcast();

        // BeefyClient
        // Seems `fs_permissions` explicitly configured as absolute path does not work and only allowed from project root
        string memory root = vm.projectRoot();
        string memory beefyCheckpointFile = string.concat(root, "/beefy-state.json");
        string memory beefyCheckpointRaw = vm.readFile(beefyCheckpointFile);
        uint64 startBlock = uint64(beefyCheckpointRaw.readUint(".startBlock"));

        BeefyClient.ValidatorSet memory current = BeefyClient.ValidatorSet(
            uint128(beefyCheckpointRaw.readUint(".current.id")),
            uint128(beefyCheckpointRaw.readUint(".current.length")),
            beefyCheckpointRaw.readBytes32(".current.root")
        );
        BeefyClient.ValidatorSet memory next = BeefyClient.ValidatorSet(
            uint128(beefyCheckpointRaw.readUint(".next.id")),
            uint128(beefyCheckpointRaw.readUint(".next.length")),
            beefyCheckpointRaw.readBytes32(".next.root")
        );

        BeefyClient beefyClient = new BeefyClient(
            beefyClientConfig.randaoCommitDelay,
            beefyClientConfig.randaoCommitExpiration,
            beefyClientConfig.minimumSignatures,
            startBlock,
            current,
            next
        );

        console2.log("BeefyClient: ", address(beefyClient));

        vm.stopBroadcast();
    }
}

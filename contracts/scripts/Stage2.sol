// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.23;

import {Script} from "forge-std/Script.sol";
import {stdJson} from "forge-std/StdJson.sol";

import {WETH9} from "canonical-weth/WETH9.sol";
import {BeefyClient} from "../src/BeefyClient.sol";

import {IGateway} from "../src/interfaces/IGateway.sol";
import {GatewayProxy} from "../src/GatewayProxy.sol";
import {Gateway} from "../src/Gateway.sol";
import {MockGatewayV2} from "../test/mocks/MockGatewayV2.sol";
import {Agent} from "../src/Agent.sol";
import {AgentExecutor} from "../src/AgentExecutor.sol";
import {ChannelID, ParaID, OperatingMode} from "../src/Types.sol";
import {SafeNativeTransfer} from "../src/utils/SafeTransfer.sol";
import {UD60x18, ud60x18} from "prb/math/src/UD60x18.sol";

contract Stage2 is Script {
    using SafeNativeTransfer for address payable;
    using stdJson for string;

    struct Config {
        uint64 startBlock;
        BeefyClient.ValidatorSet current;
        BeefyClient.ValidatorSet next;
        uint256 randaoCommitDelay;
        uint256 randaoCommitExpiration;
        uint256 minimumSignatures;
    }

    function readConfig() internal pure returns (Config memory config) { 
        config = Config({
            startBlock: 0,
            current: BeefyClient.ValidatorSet({
                id: 0,
                length: 0,
                root: 0
            }),
            next: BeefyClient.ValidatorSet({
                id: 0,
                length: 0,
                root: 0
            }),
            randaoCommitDelay: 0,
            randaoCommitExpiration: 0,
            minimumSignatures: 0
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

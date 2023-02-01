// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "forge-std/Script.sol";
import "../../BeefyClient.sol";
import "../../ParachainClient.sol";
import "../../BasicInboundChannel.sol";
import "../../BasicOutboundChannel.sol";
import "forge-std/console.sol";

contract DeployScript is Script {
    function setUp() public {}

    function run() public {
        uint256 privateKey = vm.envUint("PRIVATE_KEY");
        address deployer = vm.rememberKey(privateKey);
        vm.startBroadcast(deployer);
        uint256 randaoCommitDelay = vm.envUint("RANDAO_COMMIT_DELAY");
        uint256 randaoCommitExpiration = vm.envUint("RANDAO_COMMIT_EXP");
        BeefyClient beefyClient = new BeefyClient(randaoCommitDelay, randaoCommitExpiration);
        uint32 paraId = uint32(vm.envUint("PARAID"));
        ParachainClient parachainClient = new ParachainClient(beefyClient, paraId);
        BasicInboundChannel inboundChannel = new BasicInboundChannel(parachainClient);
        console.log("address of inboundChannel is: %s", address(inboundChannel));
        BasicOutboundChannel outboundChannel = new BasicOutboundChannel();
        outboundChannel.initialize(deployer, new address[](0));
        vm.stopBroadcast();
    }
}

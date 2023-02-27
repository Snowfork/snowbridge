// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "forge-std/Script.sol";
import "forge-std/console.sol";

import "../../BasicInboundChannel.sol";
import "../../BasicOutboundChannel.sol";
import "../../BeefyClient.sol";
import "../../ChannelRegistry.sol";
import "../../NativeTokens.sol";
import "../../ParachainClient.sol";

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
        console.log("address of outboundChannel is: %s", address(outboundChannel));

        ChannelRegistry channelRegistry = new ChannelRegistry();
        console.log("address of ChannelRegistry is: %s", address(channelRegistry));

        channelRegistry.updateChannel(0, address(inboundChannel), address(outboundChannel));
        console.log("ChannelRegistry is configured for channel 0.");

        ERC20Vault erc20vault = new ERC20Vault();
        console.log("address of ERC20Vault is: %s", address(erc20vault));

        bytes32 allowOrigin = vm.envBytes32("TOKENS_ALLOWED_ORIGIN");
        console.log("configured tokens allowed origin:");
        console.logBytes32(allowOrigin);
        NativeTokens nativeTokens = new NativeTokens(erc20vault, outboundChannel, allowOrigin);
        outboundChannel.authorizeDefaultOperator(address(nativeTokens));
        erc20vault.transferOwnership(address(nativeTokens));
        nativeTokens.transferOwnership(address(inboundChannel));
        console.log("address of NativeTokens is: %s", address(nativeTokens));

        vm.stopBroadcast();
    }
}

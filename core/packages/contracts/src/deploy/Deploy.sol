// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import "forge-std/Script.sol";
import "forge-std/console.sol";
import "../BeefyClient.sol";
import "../ParachainClient.sol";
import "../InboundChannel.sol";
import "../OutboundChannel.sol";
import "../NativeTokens.sol";
import "../Vault.sol";
import "../SovereignTreasury.sol";
import "../IVault.sol";

contract DeployScript is Script {
    function setUp() public {}

    function run() public {
        uint256 privateKey = vm.envUint("PRIVATE_KEY");
        address deployer = vm.rememberKey(privateKey);
        vm.startBroadcast(deployer);

        // SovereignTreasury
        Vault vault = new Vault();
        SovereignTreasury treasury = new SovereignTreasury(vault);

        // BeefyClient
        uint256 randaoCommitDelay = vm.envUint("RANDAO_COMMIT_DELAY");
        uint256 randaoCommitExpiration = vm.envUint("RANDAO_COMMIT_EXP");
        BeefyClient beefyClient = new BeefyClient(randaoCommitDelay, randaoCommitExpiration);

        // ParachainClient
        uint32 paraId = uint32(vm.envUint("PARAID"));
        ParachainClient parachainClient = new ParachainClient(beefyClient, paraId);

        // InboundChannel
        uint256 relayerReward = vm.envUint("RELAYER_REWARD");
        InboundChannel inboundChannel = new InboundChannel(parachainClient, vault, relayerReward);

        // OutboundChannel
        uint256 relayerFee = vm.envUint("RELAYER_FEE");
        OutboundChannel outboundChannel = new OutboundChannel(vault, relayerFee);

        // NativeTokens
        bytes memory peer = vm.envBytes("TOKENS_ALLOWED_ORIGIN");
        TokenVault tokenVault = new TokenVault();
        NativeTokens nativeTokens = new NativeTokens(tokenVault, outboundChannel, peer);
        tokenVault.transferOwnership(address(nativeTokens));

        // Allow inbound channel to send messages to NativeTokens and SovereignTreasury
        nativeTokens.grantRole(nativeTokens.SENDER_ROLE(), address(inboundChannel));
        treasury.grantRole(treasury.SENDER_ROLE(), address(inboundChannel));

        // Allow InboundChannel and SovereignTreasury to withdraw from vault
        vault.grantRole(vault.WITHDRAW_ROLE(), address(inboundChannel));
        vault.grantRole(vault.WITHDRAW_ROLE(), address(treasury));

        vm.stopBroadcast();
    }
}

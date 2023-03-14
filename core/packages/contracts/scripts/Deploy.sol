// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import "forge-std/Script.sol";
import "forge-std/console.sol";
import "../src/BeefyClient.sol";
import "../src/ParachainClient.sol";
import "../src/InboundChannel.sol";
import "../src/OutboundChannel.sol";
import "../src/NativeTokens.sol";
import "../src/Vault.sol";
import "../src/SovereignTreasury.sol";
import "../src/IVault.sol";

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
        TokenVault tokenVault = new TokenVault();
        NativeTokens nativeTokens = new NativeTokens(
            tokenVault,
            outboundChannel,
            vm.envBytes("STATEMINT_LOCATION"),
            vm.envUint("CREATE_TOKEN_FEE")
        );

        // Allow inbound channel to send messages to NativeTokens and SovereignTreasury
        nativeTokens.grantRole(nativeTokens.SENDER_ROLE(), address(inboundChannel));
        treasury.grantRole(treasury.SENDER_ROLE(), address(inboundChannel));

        // Allow InboundChannel and SovereignTreasury to withdraw from vault
        vault.grantRole(vault.WITHDRAW_ROLE(), address(inboundChannel));
        vault.grantRole(vault.WITHDRAW_ROLE(), address(treasury));

        // Allow NativeTokens to withdraw from TokenVault
        tokenVault.grantRole(vault.WITHDRAW_ROLE(), address(nativeTokens));

        vm.stopBroadcast();
    }
}

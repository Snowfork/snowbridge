// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import {WETH9} from "canonical-weth/WETH9.sol";
import {Script} from "forge-std/Script.sol";
import {BeefyClient} from "../src/BeefyClient.sol";
import {ParachainClient} from "../src/ParachainClient.sol";
import {InboundQueue} from "../src/InboundQueue.sol";
import {IRecipient} from "../src/IRecipient.sol";
import {OutboundQueue} from "../src/OutboundQueue.sol";
import {NativeTokens} from "../src/NativeTokens.sol";
import {TokenVault} from "../src/TokenVault.sol";
import {Vault} from "../src/Vault.sol";
import {IVault} from "../src/IVault.sol";
import {UpgradeProxy} from "../src/UpgradeProxy.sol";
import {SovereignTreasury} from "../src/SovereignTreasury.sol";
import {ParaID} from "../src/Types.sol";

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
        uint32 paraId = uint32(vm.envUint("BRIDGE_HUB_PARAID"));
        ParachainClient parachainClient = new ParachainClient(beefyClient, paraId);

        // InboundQueue
        uint256 relayerReward = vm.envUint("RELAYER_REWARD");
        InboundQueue inboundQueue = new InboundQueue(parachainClient, vault, relayerReward);

        // OutboundQueue
        uint256 relayerFee = vm.envUint("RELAYER_FEE");
        OutboundQueue outboundQueue = new OutboundQueue(vault, relayerFee);

        // NativeTokens
        TokenVault tokenVault = new TokenVault();
        NativeTokens nativeTokens = new NativeTokens(
            tokenVault,
            outboundQueue,
            ParaID.wrap(uint32(vm.envUint("ASSET_HUB_PARAID"))),
            vm.envUint("CREATE_TOKEN_FEE")
        );
        inboundQueue.updateHandler(1, IRecipient(nativeTokens));

        // Deploy WETH for testing
        new WETH9();

        // Upgrades
        UpgradeProxy upgradeProxy = new UpgradeProxy(ParaID.wrap(paraId));

        // Allow inbound queue to send messages to handlers
        nativeTokens.grantRole(nativeTokens.SENDER_ROLE(), address(inboundQueue));
        treasury.grantRole(treasury.SENDER_ROLE(), address(inboundQueue));
        upgradeProxy.grantRole(upgradeProxy.SENDER_ROLE(), address(inboundQueue));

        // Allow handlers to send messages to outbound queue
        outboundQueue.grantRole(outboundQueue.SUBMIT_ROLE(), address(nativeTokens));
        outboundQueue.grantRole(outboundQueue.SUBMIT_ROLE(), address(treasury));
        outboundQueue.grantRole(outboundQueue.SUBMIT_ROLE(), address(upgradeProxy));

        // Allow InboundQueue and SovereignTreasury to withdraw from vault
        vault.grantRole(vault.WITHDRAW_ROLE(), address(inboundQueue));
        vault.grantRole(vault.WITHDRAW_ROLE(), address(treasury));

        // Allow NativeTokens to use TokenVault
        tokenVault.grantRole(tokenVault.WITHDRAW_ROLE(), address(nativeTokens));
        tokenVault.grantRole(tokenVault.DEPOSIT_ROLE(), address(nativeTokens));

        // Move ownership of everything to Upgrades app

        treasury.grantRole(treasury.ADMIN_ROLE(), address(upgradeProxy));
        treasury.revokeRole(treasury.ADMIN_ROLE(), address(this));

        nativeTokens.grantRole(nativeTokens.ADMIN_ROLE(), address(upgradeProxy));
        nativeTokens.revokeRole(nativeTokens.ADMIN_ROLE(), address(this));

        vault.grantRole(vault.ADMIN_ROLE(), address(upgradeProxy));
        vault.revokeRole(vault.ADMIN_ROLE(), address(this));

        tokenVault.grantRole(tokenVault.ADMIN_ROLE(), address(upgradeProxy));
        tokenVault.revokeRole(tokenVault.ADMIN_ROLE(), address(this));

        inboundQueue.grantRole(inboundQueue.ADMIN_ROLE(), address(upgradeProxy));
        inboundQueue.revokeRole(inboundQueue.ADMIN_ROLE(), address(this));

        outboundQueue.grantRole(outboundQueue.ADMIN_ROLE(), address(upgradeProxy));
        outboundQueue.revokeRole(outboundQueue.ADMIN_ROLE(), address(this));

        upgradeProxy.revokeRole(upgradeProxy.ADMIN_ROLE(), address(this));

        vm.stopBroadcast();
    }
}

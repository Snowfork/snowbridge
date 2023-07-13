// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.20;

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
import {UpgradeProxy} from "../src/UpgradeProxy.sol";
import {SovereignTreasury} from "../src/SovereignTreasury.sol";
import {Registry} from "../src/Registry.sol";
import {ParaID} from "../src/Types.sol";

contract DeployScript is Script {
    Registry public registry;
    Vault public vault;
    BeefyClient public beefyClient;
    ParachainClient public parachainClient;
    InboundQueue public inboundQueue;
    OutboundQueue public outboundQueue;
    TokenVault public tokenVault;
    NativeTokens public nativeTokens;
    UpgradeProxy public upgradeProxy;

    function setUp() public {}

    function run() public {
        uint256 privateKey = vm.envUint("PRIVATE_KEY");
        address deployer = vm.rememberKey(privateKey);
        vm.startBroadcast(deployer);

        // Registry
        registry = new Registry();
        registry.grantRole(registry.REGISTER_ROLE(), deployer);

        // Vault
        vault = new Vault();

        // SovereignTreasury
        SovereignTreasury treasury = new SovereignTreasury(registry, vault);
        registry.registerContract(keccak256("SovereignTreasury"), address(treasury));

        // BeefyClient
        uint256 randaoCommitDelay = vm.envUint("RANDAO_COMMIT_DELAY");
        uint256 randaoCommitExpiration = vm.envUint("RANDAO_COMMIT_EXP");
        beefyClient = new BeefyClient(randaoCommitDelay, randaoCommitExpiration);

        // ParachainClient
        uint32 paraId = uint32(vm.envUint("BRIDGE_HUB_PARAID"));
        parachainClient = new ParachainClient(beefyClient, paraId);

        // InboundQueue
        uint256 relayerReward = vm.envUint("RELAYER_REWARD");
        inboundQueue = new InboundQueue(registry, parachainClient, vault, relayerReward);
        registry.registerContract(keccak256("InboundQueue"), address(inboundQueue));

        // OutboundQueue
        uint256 relayerFee = vm.envUint("RELAYER_FEE");
        outboundQueue = new OutboundQueue(registry, vault, relayerFee);
        registry.registerContract(keccak256("OutboundQueue"), address(outboundQueue));

        // NativeTokens
        tokenVault = new TokenVault();
        nativeTokens = new NativeTokens(
            registry,
            tokenVault,
            ParaID.wrap(uint32(vm.envUint("ASSET_HUB_PARAID"))),
            vm.envUint("CREATE_TOKEN_FEE"),
            bytes2(vm.envBytes("CREATE_CALL_INDEX")),
            bytes2(vm.envBytes("SET_METADATA_CALL_INDEX"))
        );
        registry.registerContract(keccak256("NativeTokens"), address(nativeTokens));

        // Deploy WETH for testing
        new WETH9();

        // UpgradeProxy
        upgradeProxy = new UpgradeProxy(registry, ParaID.wrap(paraId));
        registry.registerContract(keccak256("UpgradeProxy"), address(upgradeProxy));

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
        treasury.revokeRole(treasury.ADMIN_ROLE(), deployer);

        nativeTokens.grantRole(nativeTokens.ADMIN_ROLE(), address(upgradeProxy));
        nativeTokens.revokeRole(nativeTokens.ADMIN_ROLE(), deployer);

        vault.grantRole(vault.ADMIN_ROLE(), address(upgradeProxy));
        vault.revokeRole(vault.ADMIN_ROLE(), deployer);

        tokenVault.grantRole(tokenVault.ADMIN_ROLE(), address(upgradeProxy));
        tokenVault.revokeRole(tokenVault.ADMIN_ROLE(), deployer);

        inboundQueue.grantRole(inboundQueue.ADMIN_ROLE(), address(upgradeProxy));
        inboundQueue.revokeRole(inboundQueue.ADMIN_ROLE(), deployer);

        outboundQueue.grantRole(outboundQueue.ADMIN_ROLE(), address(upgradeProxy));
        outboundQueue.revokeRole(outboundQueue.ADMIN_ROLE(), deployer);

        registry.grantRole(outboundQueue.ADMIN_ROLE(), address(upgradeProxy));
        registry.revokeRole(outboundQueue.ADMIN_ROLE(), deployer);

        upgradeProxy.revokeRole(upgradeProxy.ADMIN_ROLE(), deployer);

        // Fund the sovereign account for the BridgeHub parachain. Used to reward relayers
        // of messages originating from BridgeHub
        uint256 initialDeposit = vm.envUint("BRIDGE_HUB_INITIAL_DEPOSIT");
        vault.deposit{value: initialDeposit}(ParaID.wrap(paraId));

        vm.stopBroadcast();
    }
}

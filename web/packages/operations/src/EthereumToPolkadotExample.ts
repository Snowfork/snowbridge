import { Context, environment, toPolkadotV2, assets as assetsV2 } from "@snowbridge/api";
import { assetRegistryFor } from "@snowbridge/registry";
import { Wallet, formatEther } from "ethers";
import { Keyring } from "@polkadot/keyring";
import { cryptoWaitReady } from "@polkadot/util-crypto";

/**
 * Example: Bridge assets from Ethereum to Polkadot AssetHub using Snowbridge
 * This example demonstrates how to transfer ERC20 tokens from Ethereum to Polkadot's AssetHub
 */
async function bridgeEthereumToPolkadot() {
    // Initialize crypto libraries
    await cryptoWaitReady();

    // Configuration - Use 'polkadot_mainnet' for production, 'paseo_sepolia' for testnet
    const environmentName = "polkadot_mainnet"; // or "paseo_sepolia" for testnet
    const snowbridgeEnv = environment.SNOWBRIDGE_ENV[environmentName];

    if (!snowbridgeEnv) {
        throw new Error(`Unknown environment '${environmentName}'`);
    }

    console.log(`Using environment: ${environmentName}`);

    // Setup Ethereum API key
    const ethApiKey = process.env.ETHEREUM_API_KEY || "your-alchemy-api-key";

    // Create registry options from environment
    const registryOptions = assetsV2.fromEnvironment(snowbridgeEnv, ethApiKey);

    // Build the context from registry options  
    const context = new Context({
        environment: environmentName,
        ethereum: {
            ethChainId: snowbridgeEnv.ethChainId,
            ethChains: Object.fromEntries(
                Object.keys(snowbridgeEnv.config.ETHEREUM_CHAINS).map(chainId => [
                    chainId, 
                    snowbridgeEnv.config.ETHEREUM_CHAINS[chainId](ethApiKey)
                ])
            ),
            beacon_url: snowbridgeEnv.config.BEACON_HTTP_API,
        },
        polkadot: {
            assetHubParaId: registryOptions.assetHubParaId,
            bridgeHubParaId: registryOptions.bridgeHubParaId,
            relaychain: registryOptions.relaychain,
            parachains: snowbridgeEnv.config.PARACHAINS,
        },
        appContracts: {
            gateway: registryOptions.gatewayAddress,
            beefy: snowbridgeEnv.config.BEEFY_CONTRACT,
        },
    });

    // Setup accounts
    const ethereumPrivateKey = process.env.ETHEREUM_PRIVATE_KEY || "0x...";
    const polkadotMnemonic = process.env.POLKADOT_MNEMONIC || "//Alice";

    const ethereumWallet = new Wallet(ethereumPrivateKey, context.ethereum());
    const polkadotKeyring = new Keyring({ type: "sr25519" });
    const polkadotAccount = polkadotKeyring.addFromUri(polkadotMnemonic);

    const ethereumAddress = await ethereumWallet.getAddress();
    const polkadotAddress = polkadotAccount.address;

    console.log("Ethereum Address:", ethereumAddress);
    console.log("Polkadot Address:", polkadotAddress);

    // Get the asset registry for the environment
    const registry = assetRegistryFor(environmentName);

    // Token configuration  
    const tokenAddress = "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2"; // WETH on mainnet
    const destinationParaId = 1000; // AssetHub parachain ID
    const transferAmount = BigInt("1000000000000000"); // 0.001 ETH in wei

    try {
        // Step 1: Get delivery fee for the transfer
        console.log("\n🔍 Calculating delivery fee...");
        const deliveryFee = await toPolkadotV2.getDeliveryFee(
            {
                gateway: context.gateway(),
                assetHub: await context.assetHub(),
                destination: await context.parachain(destinationParaId),
            },
            registry,
            tokenAddress,
            destinationParaId
        );

        console.log("Delivery fee:", formatEther(deliveryFee.totalFeeInWei), "ETH");

        // Step 2: Create the transfer transaction
        console.log("\n📝 Creating transfer transaction...");
        const transfer = await toPolkadotV2.createTransfer(
            registry,
            ethereumAddress,
            polkadotAddress,
            tokenAddress,
            destinationParaId,
            transferAmount,
            deliveryFee
        );

        // Step 3: Validate the transfer
        console.log("\n✅ Validating transfer...");
        const validation = await toPolkadotV2.validateTransfer(
            {
                ethereum: context.ethereum(),
                gateway: context.gateway(),
                bridgeHub: await context.bridgeHub(),
                assetHub: await context.assetHub(),
                destParachain: destinationParaId !== 1000 
                    ? await context.parachain(destinationParaId) 
                    : undefined,
            },
            transfer
        );

        // Check for validation errors
        const errors = validation.logs.filter(log => 
            log.kind === toPolkadotV2.ValidationKind.Error
        );

        if (errors.length > 0) {
            console.error("❌ Validation errors:", errors);
            throw new Error("Transfer validation failed");
        }

        console.log("✅ Transfer validation passed");

        // Step 4: Estimate gas costs
        console.log("\n💰 Estimating costs...");
        const { tx, computed: { totalValue } } = transfer;
        const estimatedGas = await context.ethereum().estimateGas(tx);
        const feeData = await context.ethereum().getFeeData();
        const executionFee = (feeData.gasPrice ?? 0n) * estimatedGas;

        console.log("Gas estimate:", estimatedGas.toString());
        console.log("Delivery cost:", formatEther(deliveryFee.totalFeeInWei), "ETH");
        console.log("Execution cost:", formatEther(executionFee), "ETH");
        console.log("Total cost:", formatEther(deliveryFee.totalFeeInWei + executionFee), "ETH");
        console.log("Token amount:", formatEther(transferAmount), "WETH");

        // Step 5: Execute the transfer (remove this check for actual execution)
        const dryRun = process.env.DRY_RUN !== "false";

        if (dryRun) {
            console.log("\n🔍 Dry run - transaction not executed");
            const result = await context.ethereum().call(tx);
            console.log("Dry run result:", result);
        } else {
            console.log("\n🚀 Executing transfer...");

            // Send the transaction
            const response = await ethereumWallet.sendTransaction(tx);
            console.log("Transaction hash:", response.hash);

            // Wait for confirmation
            const receipt = await response.wait(1);
            if (!receipt) {
                throw new Error(`Transaction ${response.hash} not included`);
            }

            console.log("✅ Transaction confirmed in block:", receipt.blockNumber);

            // Get message receipt for tracking
            const message = await toPolkadotV2.getMessageReceipt(receipt);
            if (!message) {
                throw new Error(`Transaction did not emit a message`);
            }

            console.log("📨 Bridge message details:");
            console.log("- Message ID:", message.messageId);
            console.log("- Block number:", message.blockNumber);
            console.log("- Transaction hash:", message.txHash);

            console.log("\n🎉 Transfer initiated successfully!");
            console.log("Your tokens will arrive on Polkadot AssetHub shortly.");
        }

    } catch (error) {
        console.error("❌ Transfer failed:", error);
        throw error;
    } finally {
        // Clean up context
        await context.destroyContext();
    }
}

// Environment setup example
function setupEnvironment() {
    // Set these environment variables:
    process.env.ETHEREUM_API_KEY = "your-alchemy-api-key";
    process.env.ETHEREUM_PRIVATE_KEY = "0x..."; // Your Ethereum private key
    process.env.POLKADOT_MNEMONIC = "your twelve word mnemonic here"; // Your Polkadot mnemonic
    process.env.DRY_RUN = "true"; // Set to "false" to execute actual transfers
}

// Usage example
async function main() {
    try {
        setupEnvironment();
        await bridgeEthereumToPolkadot();
    } catch (error) {
        console.error("Bridge operation failed:", error);
        process.exit(1);
    }
}

// Run the script
main();

export { bridgeEthereumToPolkadot, setupEnvironment };
import { Keyring } from "@polkadot/keyring"
import {
    Context,
    environment,
    toPolkadotV2,
    assetsV2,
    toEthereumV2
} from "@snowbridge/api"
import { WETH9__factory } from "@snowbridge/contract-types"
import { formatEther, formatUnits, Wallet } from "ethers"
import { cryptoWaitReady } from '@polkadot/util-crypto';
import { readFile, writeFile } from "fs/promises";
import { existsSync } from "fs";

function cache<T>(filePath: string, generator: () => T | Promise<T>): Promise<T> {
    return (async () => {
        if (existsSync(filePath)) {
            // Read and parse existing cache file
            const data = await readFile(filePath);
            return JSON.parse(data.toString("utf-8"), (key, value) => {
                if (typeof value === "string" && /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$/.test(value)) {
                    return new Date(value);
                }
                if (typeof value === "string" && /^bigint:\d+$/.test(value)) {
                    return BigInt(value.slice(7));
                }
                return value;
            }) as T;
        }

        // Generate new data and cache it
        const result = await generator();
        const json = JSON.stringify(result, (key, value) => {
            if (typeof value === "bigint") {
                return `bigint:${value.toString()}`;
            }
            return value;
        }, 2);

        await writeFile(filePath, json);
        return result;
    })();
}

const monitor = async () => {
    let env = "local_e2e"
    if (process.env.NODE_ENV !== undefined) {
        env = process.env.NODE_ENV
    }
    const snwobridgeEnv = environment.SNOWBRIDGE_ENV[env]
    if (snwobridgeEnv === undefined) {
        throw Error(`Unknown environment '${env}'`)
    }
    console.log(`Using environment '${env}'`)

    const { name, config, ethChainId } = snwobridgeEnv
    await cryptoWaitReady()

    const ethApikey = process.env.REACT_APP_INFURA_KEY || ""
    const ethChains: { [ethChainId: string]: string } = {}
    Object.keys(config.ETHEREUM_CHAINS)
        .forEach(ethChainId => ethChains[ethChainId.toString()] = config.ETHEREUM_CHAINS[ethChainId](ethApikey))
    const context = new Context({
        environment: name,
        ethereum: {
            ethChainId,
            ethChains,
            beacon_url: config.BEACON_HTTP_API,
        },
        polkadot: {
            assetHubParaId: config.ASSET_HUB_PARAID,
            bridgeHubParaId: config.BRIDGE_HUB_PARAID,
            relaychain: config.RELAY_CHAIN_URL,
            parachains: config.PARACHAINS,
        },
        appContracts: {
            gateway: config.GATEWAY_CONTRACT,
            beefy: config.BEEFY_CONTRACT,
        },
    })


    // Step 0. Build the Asset Registry. The registry contains the list of all token and parachain metadata in order to send tokens.
    // It may take some build but does not change often so it is safe to cache for 12 hours and shipped with your dapp as static data.
    //
    // The registry can be build from a snowbridge environment or snowbridge coutntext.
    //      const registry = await assetsV2.buildRegistry(assetsV2.fromEnvironment(snwobridgeEnv))
    // If your dapp does not use the snowbridge environment or context you can always build it manually by
    // specifying RegistryOptions for only the parachains you care about.


    const registry = await assetsV2.buildRegistry(
        await assetsV2.fromContext(context),
    )
    console.log("Asset Registry:", JSON.stringify(registry, (_, value) => typeof value === "bigint" ? String(value) : value, 2))

    context.destroyContext()
}

monitor()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error("Error:", error)
        process.exit(1)
    })

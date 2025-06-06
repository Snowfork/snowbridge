import { Context, assetsV2, types } from "@snowbridge/api"
import { readFile, writeFile } from "fs/promises"
import { existsSync } from "fs"

function cache<T>(filePath: string, generator: () => T | Promise<T>): Promise<T> {
    return (async () => {
        if (existsSync(filePath)) {
            // Read and parse existing cache file
            const data = await readFile(filePath)
            return JSON.parse(data.toString("utf-8"), (key, value) => {
                if (
                    typeof value === "string" &&
                    /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$/.test(value)
                ) {
                    return new Date(value)
                }
                if (typeof value === "string" && /^bigint:\d+$/.test(value)) {
                    return BigInt(value.slice(7))
                }
                return value
            }) as T
        }

        // Generate new data and cache it
        const result = await generator()
        const json = JSON.stringify(
            result,
            (key, value) => {
                if (typeof value === "bigint") {
                    return `bigint:${value.toString()}`
                }
                return value
            },
            2
        )

        await writeFile(filePath, json)
        return result
    })()
}

// Build the Asset Registry. The registry contains the list of all token and parachain metadata in order to send tokens.
// It may take some build but does not change often so it is safe to cache for 12 hours and shipped with your dapp as static data.
//
// The registry can be build from a snowbridge environment or snowbridge context.
//      const registry = await assetsV2.buildRegistry(assetsV2.fromEnvironment(snwobridgeEnv))
// If your dapp does not use the snowbridge environment or context you can always build it manually by
// specifying RegistryOptions for only the parachains you care about.
export const fetchRegistry = async (
    env: string,
    context: Context
): Promise<types.AssetRegistry> => {
    const registry = await cache(
        `.${env}.registry.json`,
        async () => await assetsV2.buildRegistry(await assetsV2.fromContext(context))
    )
    console.log(
        "Asset Registry:",
        JSON.stringify(
            registry,
            (_, value) => (typeof value === "bigint" ? String(value) : value),
            2
        )
    )
    return registry
}

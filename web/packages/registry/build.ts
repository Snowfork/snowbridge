import "dotenv/config"
import { assetsV2, environment } from "@snowbridge/api"
import { writeFile } from "fs/promises"

async function buildRegistry(env: string, options: assetsV2.RegistryOptions) {
    const registry = await assetsV2.buildRegistry(options)
    const json = JSON.stringify(
        registry,
        (key, value) => {
            if (typeof value === "bigint") {
                return `bigint:${value.toString()}`
            }
            return value
        },
        2,
    )

    const filepath = `src/${env}.registry.json`
    await writeFile(filepath, json)
}

;(async () => {
    let env = "local_e2e"
    if (process.env.NODE_ENV !== undefined) {
        env = process.env.NODE_ENV
    }
    const options = assetsV2.fromEnvironment(environment.SNOWBRIDGE_ENV[env])
    await buildRegistry(env, options)
})()

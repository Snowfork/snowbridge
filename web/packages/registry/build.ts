import { assetsV2 } from "@snowbridge/api"
import { writeFile } from "fs/promises"
import { environmentFor } from "./src"

async function buildRegistry(env: string) {
    const registry = await assetsV2.buildRegistry(environmentFor(env))
    const json = JSON.stringify(
        registry,
        (key, value) => {
            if (typeof value === "bigint") {
                return `bigint:${value.toString()}`
            }
            return value
        },
        2
    )

    const filepath = `src/${env}.registry.json`
    await writeFile(filepath, json)
}

;(async () => {
    let env = "local_e2e"
    if (process.env.NODE_ENV !== undefined) {
        env = process.env.NODE_ENV
    }
    await buildRegistry(env)
})()

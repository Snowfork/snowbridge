import "dotenv/config"
import { environment, assetsV2 } from "@snowbridge/api"
import { writeFile } from "fs/promises"

const run = async () => {
    let env = "local_e2e"
    if (process.env.NODE_ENV !== undefined) {
        env = process.env.NODE_ENV
    }

    const options = assetsV2.fromEnvironment(environment.SNOWBRIDGE_ENV[env])
    const registry = await assetsV2.buildRegistry(options)
    const json = JSON.stringify(
        registry,
        (_, value) => {
            if (typeof value === "bigint") {
                return `bigint:${value.toString()}`
            }
            return value
        },
        2,
    )
    console.log("Asset Registry:", json)
    const filepath = `${env}.registry.json`
    await writeFile(filepath, json)
}

run()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error("Error:", error)
        process.exit(1)
    })

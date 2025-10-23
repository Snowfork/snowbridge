import "dotenv/config"
import { Context, contextConfigFor } from "@snowbridge/api"
import { cryptoWaitReady } from "@polkadot/util-crypto"
import { assetRegistryFor } from "@snowbridge/registry"

async function checkXcmVersion() {
    await cryptoWaitReady()

    let env = "local_e2e"
    if (process.env.NODE_ENV !== undefined) {
        env = process.env.NODE_ENV
    }
    console.log(`Using environment '${env}'`)

    const context = new Context(contextConfigFor(env))
    const registry = assetRegistryFor(env)

    console.log("\nChecking XCM versions for parachains...\n")

    // Get all parachain IDs from the registry
    const parachainIds = Object.keys(registry.parachains)

    for (const paraIdStr of parachainIds) {
        const paraId = parseInt(paraIdStr)
        const parachain = registry.parachains[paraIdStr]

        try {
            console.log(`\n--- ${parachain.info.name} (Para ID: ${paraId}) ---`)

            // Connect to the parachain
            const api = await context.parachain(paraId)

            // Try to create a v5 XCM to check if it's supported
            let supportsV5 = false
            let supportsV4 = false

            try {
                api.registry.createType("XcmVersionedXcm", {
                    v5: [{ clearOrigin: null }],
                })
                supportsV5 = true
            } catch (e) {
                // v5 not supported
            }

            try {
                api.registry.createType("XcmVersionedXcm", {
                    v4: [{ clearOrigin: null }],
                })
                supportsV4 = true
            } catch (e) {
                // v4 not supported
            }

            // Check if initiateTransfer is available in the instruction set
            let supportsInitiateTransfer = false
            try {
                const testXcm = api.registry.createType("XcmVersionedXcm", {
                    v5: [
                        {
                            initiateTransfer: {
                                destination: { parents: 0, interior: "Here" },
                                remote_fees: {
                                    reserveDeposit: { definite: [] },
                                },
                                preserveOrigin: false,
                                assets: [],
                                remoteXcm: [],
                            },
                        },
                    ],
                })
                supportsInitiateTransfer = true
            } catch (e) {
                // initiateTransfer not supported
            }

            console.log(`  XCM v4 support: ${supportsV4 ? "✓ YES" : "✗ NO"}`)
            console.log(`  XCM v5 support: ${supportsV5 ? "✓ YES" : "✗ NO"}`)
            console.log(
                `  initiateTransfer support: ${supportsInitiateTransfer ? "✓ YES" : "✗ NO"}`
            )

            if (supportsV5 && supportsInitiateTransfer) {
                console.log(`  >>> COMPATIBLE with custom XCM script <<<`)
            }

            // Get runtime version for additional info
            const runtimeVersion = api.runtimeVersion
            console.log(`  Runtime: ${runtimeVersion.specName.toString()}`)
            console.log(`  Version: ${runtimeVersion.specVersion.toString()}`)
        } catch (error: any) {
            console.log(`  Error connecting: ${error.message}`)
        }
    }

    console.log("\n\n=== Summary ===")
    console.log("Parachains with XCM v5 + initiateTransfer support are compatible")
    console.log("with the transact_to_hydration.ts script pattern.\n")

    await context.destroyContext()
}

checkXcmVersion()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error("Error:", error)
        process.exit(1)
    })

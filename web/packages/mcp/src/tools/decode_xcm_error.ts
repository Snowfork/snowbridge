import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js"
import { ApiPromise, WsProvider } from "@polkadot/api"
import { hexToU8a } from "@polkadot/util"
import { z } from "zod"

const ASSET_HUB_URL = process.env.SNOWBRIDGE_ASSETHUB_URL ?? ""

let apiPromise: Promise<ApiPromise> | undefined

async function getApi(): Promise<ApiPromise> {
    if (!ASSET_HUB_URL) {
        throw new Error(
            "SNOWBRIDGE_ASSETHUB_URL is not set. Configure it in the MCP server env.",
        )
    }
    if (!apiPromise) {
        const provider = new WsProvider(ASSET_HUB_URL)
        apiPromise = ApiPromise.create({ provider })
    }
    return apiPromise
}

export function registerDecodeXcmError(server: McpServer) {
    // @ts-expect-error TS2589: SDK's dual Zod 3/4 generic resolution overflows the
    // type-instantiation budget. Runtime is fine; Zod validates inputs.
    server.registerTool(
        "decode_xcm_error",
        {
            description:
                "Decode raw XCM error bytes into a human-readable form. Useful when an event " +
                "(e.g. polkadotXcm.Attempted, dryRunCall result) carries a SCALE-encoded XcmError " +
                "or LocalExecutionIncompleteWithError(u8 instruction_idx, XcmError) payload.",
            inputSchema: {
                bytes: z
                    .string()
                    .regex(/^0x[0-9a-fA-F]*$/)
                    .describe("Hex-encoded error bytes, including 0x prefix."),
                kind: z
                    .enum(["XcmError", "LocalExecutionIncompleteWithError"])
                    .default("XcmError")
                    .describe(
                        "Decoder shape. Use LocalExecutionIncompleteWithError when the bytes " +
                            "begin with the variant tag for that error.",
                    ),
            },
        },
        async ({ bytes, kind }) => {
            const api = await getApi()
            const u8a = hexToU8a(bytes)

            if (kind === "LocalExecutionIncompleteWithError") {
                // Strip the 1-byte enum-variant tag, then parse (u8 idx, XcmV5TraitsError).
                const inner = u8a.subarray(1)
                const idx = inner[0]
                const xcmErr = api.registry.createType(
                    "XcmV5TraitsError",
                    inner.subarray(1),
                )
                return {
                    content: [
                        {
                            type: "text",
                            text: `XCM execution failed at instruction #${idx} with: ${xcmErr.toString()}`,
                        },
                    ],
                }
            }

            const xcmErr = api.registry.createType("XcmV5TraitsError", u8a)
            return {
                content: [{ type: "text", text: xcmErr.toString() }],
            }
        },
    )
}

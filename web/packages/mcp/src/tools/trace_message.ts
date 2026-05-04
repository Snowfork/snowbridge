import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js"
import { z } from "zod"
import { fetchTransfer } from "./subsquid"

const SUBSQUID_URL = process.env.SNOWBRIDGE_SUBSQUID_URL ?? ""

// All fields kept .optional() in the schema; required-vs-optional is enforced
// at runtime in the handler. Mixing required + optional Zod fields here trips
// TS2589 "excessively deep type instantiation" in the SDK's dual Zod 3/4
// generic resolution.
const inputSchema: Record<string, z.ZodTypeAny> = {
    id: z
        .string()
        .optional()
        .describe(
            "Required. Any of: nonce (e.g. '264'), messageId (0x...), or source-chain txHash (0x...).",
        ),
    direction: z
        .string()
        .optional()
        .describe(
            "Direction: 'toPolkadot' or 'toEthereum'. Defaults to 'toPolkadot' when omitted.",
        ),
}

export function registerTraceMessage(server: McpServer) {
    // @ts-expect-error TS2589: SDK's dual Zod 3/4 generic resolution overflows the
    // type-instantiation budget for this specific schema shape. Runtime is fine;
    // Zod validates inputs.
    server.registerTool(
        "trace_message",
        {
            description: "Trace cross-chain status of a Snowbridge V2 message.",
            inputSchema,
        },
        async ({ id, direction }) => {
            if (!SUBSQUID_URL) {
                throw new Error(
                    "SNOWBRIDGE_SUBSQUID_URL is not set. Configure it in the MCP server env.",
                )
            }
            if (!id) {
                throw new Error("trace_message requires `id` (nonce, messageId, or txHash).")
            }
            const dir = direction === "toEthereum" ? "toEthereum" : "toPolkadot"
            const transfer = await fetchTransfer(SUBSQUID_URL, dir, id as string)
            if (!transfer) {
                return {
                    content: [{ type: "text", text: `No transfer found for id=${id} (${dir}).` }],
                }
            }
            return {
                content: [{ type: "text", text: JSON.stringify(transfer, null, 2) }],
            }
        },
    )
}

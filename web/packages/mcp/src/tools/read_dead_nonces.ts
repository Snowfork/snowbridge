import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js"
import { readFile } from "fs/promises"
import { z } from "zod"

const DEFAULT_PATH = process.env.SNOWBRIDGE_DEAD_NONCES_PATH ?? ""

interface DeadNonceEntry {
    reason: string
    marked_at: string
}

const inputSchema: Record<string, z.ZodTypeAny> = {
    path: z
        .string()
        .optional()
        .describe(
            "Path to the dead-nonces JSON file. Defaults to SNOWBRIDGE_DEAD_NONCES_PATH env var.",
        ),
    nonce: z
        .number()
        .int()
        .nonnegative()
        .optional()
        .describe("If set, only return the entry for this specific nonce."),
}

export function registerReadDeadNonces(server: McpServer) {
    server.registerTool(
        "read_dead_nonces",
        {
            description:
                "Read the relayer's dead-nonce store — nonces that have failed permanently and " +
                "are skipped on subsequent runs. The store is written by the ethereum-v2 relayer " +
                "(see relayer/relays/error_tracking/dead_nonces.go).",
            inputSchema,
        },
        async ({ path, nonce }) => {
            const filePath = path ?? DEFAULT_PATH
            if (!filePath) {
                throw new Error(
                    "No path supplied and SNOWBRIDGE_DEAD_NONCES_PATH is not set.",
                )
            }

            let raw: string
            try {
                raw = await readFile(filePath, "utf8")
            } catch (e: any) {
                if (e?.code === "ENOENT") {
                    return {
                        content: [
                            {
                                type: "text",
                                text: `No dead-nonce file at ${filePath} (none recorded yet).`,
                            },
                        ],
                    }
                }
                throw e
            }

            const data: Record<string, DeadNonceEntry> = raw.trim() ? JSON.parse(raw) : {}

            if (nonce !== undefined) {
                const entry = data[String(nonce)]
                return {
                    content: [
                        {
                            type: "text",
                            text: entry
                                ? JSON.stringify({ nonce, ...entry }, null, 2)
                                : `Nonce ${nonce} is not in the dead-nonce store.`,
                        },
                    ],
                }
            }

            const summary = Object.entries(data)
                .map(([n, e]) => `nonce=${n} marked_at=${e.marked_at} reason=${e.reason}`)
                .join("\n")
            return {
                content: [
                    {
                        type: "text",
                        text: summary || `${filePath} is empty (no dead nonces recorded).`,
                    },
                ],
            }
        },
    )
}

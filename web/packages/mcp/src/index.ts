#!/usr/bin/env node
import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js"
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js"
import { registerTraceMessage } from "./tools/trace_message"
import { registerDecodeXcmError } from "./tools/decode_xcm_error"
import { registerReadDeadNonces } from "./tools/read_dead_nonces"

async function main() {
    const server = new McpServer({
        name: "snowbridge-mcp",
        version: "0.1.0",
    })

    registerTraceMessage(server)
    registerDecodeXcmError(server)
    registerReadDeadNonces(server)

    const transport = new StdioServerTransport()
    await server.connect(transport)
    process.stderr.write("snowbridge-mcp ready on stdio\n")
}

main().catch((err) => {
    process.stderr.write(`snowbridge-mcp fatal: ${err?.stack ?? err}\n`)
    process.exit(1)
})

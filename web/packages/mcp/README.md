# @snowbridge/mcp

An MCP (Model Context Protocol) server exposing Snowbridge debugging tools. Connect it to an MCP client (Claude Desktop, Claude Code, Cursor, etc.) and the LLM can trace cross-chain messages, decode XCM errors, and read the relayer's dead-nonce store without the user copy-pasting Subscan URLs.

## Tools

| Name | Purpose |
| --- | --- |
| `trace_message` | Look up a Snowbridge V2 transfer by nonce, messageId, or source-chain txHash. Returns the cross-chain status (Eth → BH → AH or reverse). Wraps `historyV2.toPolkadotTransferById` / `toEthereumTransferById`. |
| `decode_xcm_error` | Decode raw XCM error bytes (e.g. from a `polkadotXcm.Attempted` event or a `dryRunCall` result) into a human-readable form. Supports the `LocalExecutionIncompleteWithError(u8 idx, XcmError)` shape. |
| `read_dead_nonces` | Read the relayer's persistent dead-nonce file (see `relayer/relays/error_tracking/dead_nonces.go`). Optionally filter by nonce. |

## Environment

```
SNOWBRIDGE_SUBSQUID_URL        # GraphQL endpoint for trace_message
SNOWBRIDGE_ASSETHUB_URL        # ws endpoint for decode_xcm_error type registry (e.g. wss://polkadot-asset-hub-rpc.polkadot.io)
SNOWBRIDGE_DEAD_NONCES_PATH    # default path for read_dead_nonces (overridable per call)
```

## Install / run

From the monorepo root:

```sh
cd web && pnpm install
cd packages/mcp && pnpm build
```

Run directly:

```sh
SNOWBRIDGE_SUBSQUID_URL=https://... \
SNOWBRIDGE_ASSETHUB_URL=wss://... \
SNOWBRIDGE_DEAD_NONCES_PATH=/var/lib/snowbridge-relayer/dead-nonces.json \
node dist/index.js
```

Or in dev (no build step):

```sh
pnpm dev
```

## Wiring into an MCP client

`~/Library/Application Support/Claude/claude_desktop_config.json` (Claude Desktop):

```json
{
  "mcpServers": {
    "snowbridge": {
      "command": "node",
      "args": ["/abs/path/to/web/packages/mcp/dist/index.js"],
      "env": {
        "SNOWBRIDGE_SUBSQUID_URL": "https://...",
        "SNOWBRIDGE_ASSETHUB_URL": "wss://polkadot-asset-hub-rpc.polkadot.io",
        "SNOWBRIDGE_DEAD_NONCES_PATH": "/var/lib/snowbridge-relayer/dead-nonces.json"
      }
    }
  }
}
```

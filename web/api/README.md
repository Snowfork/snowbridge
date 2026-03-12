# Vercel serverless API

## `/api/monitor`

Runs the Snowbridge monitor (bridge status, channel status, relayer/sovereign balances, indexer status) and sends metrics to CloudWatch. Used by Vercel Cron for periodic runs.

- **Cron**: Every 15 minutes (`*/15 * * * *`). Configure in `vercel.json`; change `schedule` to adjust.
- **Auth**: Set `CRON_SECRET` in Vercel (Project → Settings → Environment Variables). Vercel sends it as `Authorization: Bearer <CRON_SECRET>` when invoking the cron.
- **Env**: Same as the CLI monitor: `NODE_ENV`, RPC URLs (`BEACON_RPC_URL`, `ETHEREUM_RPC_URL_*`, `PARACHAIN_RPC_URL_*`, `RELAY_CHAIN_RPC_URL`), and AWS/CloudWatch vars used by `sendMetrics` in `packages/operations`.

Build must produce `packages/operations/dist/` (e.g. run `pnpm build` from repo root or `web` so that `@snowbridge/operations` is built).

### Build (contract-types + Foundry)

The build depends on `@snowbridge/contract-types`, which runs `forge build` in `contracts/` then typechain. So Foundry must be available in the Vercel build.

- **Root Directory**: Keep as **`web`**. The full repo is cloned, so from `web/packages/contract-types`, `../../../contracts` correctly points at the repo’s `contracts/` folder.
- **Node**: Use **Node 20.x** in Vercel (Project → Settings → General → Node.js Version). Required so the `@foundryup/foundry` binaries work (avoids GLIBC errors).
- **Install / Build**: `vercel.json` uses `installCommand: pnpm install && bash scripts/install-foundry.sh` and `buildCommand: pnpm run build:vercel`. The script prefers `forge` from the `@foundryup/foundry` npm package (installed by `pnpm install`); if that’s not available it runs `foundryup`. Build adds `./.foundry/bin`, `~/.foundry/bin`, and `node_modules/.bin` to PATH so `forge` is found.
- If Foundry still cannot be used (e.g. restricted env), the install script exits successfully and the build continues; contract-types will only run `tsc` if pre-generated `src/` exists.

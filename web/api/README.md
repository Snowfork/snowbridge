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
- **Install / Build**: `vercel.json` sets `installCommand` to install Foundry and `buildCommand` to `pnpm run build:vercel` (adds `~/.foundry/bin` to PATH then runs the normal turbo build). If you override these in the dashboard, use Install: `pnpm install && bash scripts/install-foundry.sh`, Build: `pnpm run build:vercel`.
- If Foundry cannot be installed (e.g. GLIBC on Vercel), you can instead commit the typechain-generated `web/packages/contract-types/src` and change contract-types build to skip `forge` when `forge` is not in PATH (then build is `tsc` only).

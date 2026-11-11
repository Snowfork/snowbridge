// Dry-runs both legs on mainnet without signing.
// Usage: NODE_ENV=polkadot_mainnet npx ts-node src/stables_dry_run.ts <HOLLAR|USDT|USDC> <amount> [USDT|USDC] [ethBeneficiary]
// SUBSTRATE_ACCOUNT_PUBLIC: leg 1 account. STABLES_LEG2_ACCOUNT: leg 2 account (default: quote pool).
import { createApi, stables } from "@snowbridge/api"
import { EthersEthereumProvider } from "@snowbridge/provider-ethers"
import { cryptoWaitReady } from "@polkadot/util-crypto"
import { bridgeInfoFor } from "@snowbridge/registry"
import { formatUnits, parseUnits } from "ethers"

const main = async () => {
    await cryptoWaitReady()
    const env = process.env.NODE_ENV ?? "polkadot_mainnet"
    const info = bridgeInfoFor(env)
    const api = createApi({ info, ethereumProvider: new EthersEthereumProvider() })
    const from = (process.argv[2] ?? "HOLLAR") as stables.HydrationStableSymbol
    const source = stables.HYDRATION_STABLES[from]
    const amount = parseUnits(process.argv[3] ?? "100", source.decimals)
    const to = (process.argv[4] ?? "USDT") as stables.EthereumStableSymbol
    const beneficiary = process.argv[5] ?? "0x0000000000000000000000000000000000000001"
    const account = process.env.SUBSTRATE_ACCOUNT_PUBLIC
    if (!account) throw Error("Set SUBSTRATE_ACCOUNT_PUBLIC")

    const s = api.stables()
    console.log("balances", await s.balances(account))

    console.log(`== Leg 1: ${from} Asset Hub -> Hydration`)
    const leg1 = await s.moveToHydrationTx(account, from, amount)
    const v1 = await s.validateMoveToHydration(leg1)
    console.log("success:", v1.success, v1.logs, v1.data)

    console.log(`== Leg 2: ${from} -> ${to} on Hydration -> Ethereum`)
    const leg2Account =
        process.env.STABLES_LEG2_ACCOUNT ?? stables.stableswapPoolAccountHex(source.quotePoolId)
    const quote = await s.quote(leg2Account, from, to, amount)
    console.log(
        `quote: ${formatUnits(amount, source.decimals)} ${from} -> ${formatUnits(quote.amountOut, quote.to.decimals)} ${to}`,
    )
    const fee = await s.swapAndBridgeFee(to)
    console.log("fee totals:", fee.totals)
    const leg2 = await s.swapAndBridgeTx(leg2Account, beneficiary, quote, 50n, fee)
    console.log(
        "minAmountOut:",
        formatUnits(leg2.minAmountOut, quote.to.decimals),
        "messageId:",
        leg2.messageId,
    )
    const v2 = await s.validateSwapAndBridge(leg2)
    console.log("success:", v2.success, v2.logs, v2.data)
    await api.destroy()
    process.exit(v1.success && v2.success ? 0 : 1)
}
main().catch((e) => {
    console.error(e)
    process.exit(1)
})

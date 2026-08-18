import { createServer } from "node:http"
import { Counter, Gauge, Registry, collectDefaultMetrics } from "prom-client"
import * as status from "@snowbridge/api/dist/status"
import { collectMetrics, sectionOk } from "./monitor"
import * as fisherman from "./fisherman"
import { AllMetrics } from "./metrics"

const Port = parseInt(process.env["METRICS_PORT"] || "9000")
const Host = process.env["METRICS_HOST"] || "0.0.0.0"
const RefreshInterval = parseInt(process.env["MONITOR_REFRESH_INTERVAL_SECONDS"] || "900") * 1000
const TickTimeout = parseInt(process.env["MONITOR_TICK_TIMEOUT_SECONDS"] || "1800") * 1000
const FishermanEnabled = process.env["FISHERMAN_ENABLED"] !== "false"

const register = new Registry()
collectDefaultMetrics({ register })

const gauge = (name: string, help: string, labelNames: string[] = []) =>
    new Gauge({ name, help, labelNames, registers: [register] })

const secondsSinceRefresh = gauge(
    "snowbridge_monitor_seconds_since_last_refresh",
    "Seconds since the last successful refresh, or -1 if it has never refreshed",
)
const lastRefreshTimestamp = gauge(
    "snowbridge_monitor_last_refresh_timestamp_seconds",
    "Unix timestamp of the last successful refresh",
)
const refreshDuration = gauge(
    "snowbridge_monitor_refresh_duration_seconds",
    "Duration of the last successful refresh",
)
const collectionSectionOk = gauge(
    "snowbridge_collection_section_ok",
    "1 if the collection section produced data on the last refresh",
    ["section"],
)
const refreshes = new Counter({
    name: "snowbridge_monitor_refreshes_total",
    help: "Refresh attempts by result",
    labelNames: ["result"],
    registers: [register],
})

const beefyLatency = gauge("snowbridge_beefy_latency_seconds", "To-Ethereum beefy latency")
const latestBeefyBlock = gauge(
    "snowbridge_latest_beefy_block",
    "Latest Polkadot block known to Ethereum",
)
const beaconLatency = gauge("snowbridge_beacon_latency_seconds", "To-Polkadot beacon latency")
const latestBeaconSlot = gauge(
    "snowbridge_latest_beacon_slot",
    "Latest beacon slot known to Polkadot",
)
const operatingMode = gauge("snowbridge_operating_mode", "0 if Normal, 1 if Halted", [
    "direction",
    "component",
])
const channelNonce = gauge("snowbridge_channel_nonce", "Channel nonce", [
    "channel",
    "direction",
    "kind",
])
const channelDeliveryEstimate = gauge(
    "snowbridge_channel_delivery_estimate_seconds",
    "Estimated delivery time over the most recent messages",
    ["channel", "direction"],
)
const channelUndeliveredTimeout = gauge(
    "snowbridge_channel_undelivered_timeout_seconds",
    "Timeout of the oldest undelivered message",
    ["channel", "direction"],
)
const relayerBalance = gauge("snowbridge_relayer_balance", "Relayer balance in base units", [
    "relayer",
    "type",
])
const sovereignBalance = gauge("snowbridge_sovereign_balance", "Sovereign balance in base units", [
    "sovereign",
    "type",
])
const indexerLatency = gauge("snowbridge_indexer_latency_blocks", "Indexer latency in blocks", [
    "chain",
])
const poolLiquidity = gauge("snowbridge_pool_liquidity", "Pool liquidity in base units", [
    "chain",
    "pool",
    "asset",
])

const fishermanLastScannedBlock = gauge(
    "snowbridge_fisherman_last_scanned_block",
    "Last Ethereum block scanned by the fisherman",
)
const fishermanScanLag = gauge(
    "snowbridge_fisherman_scan_lag_blocks",
    "Ethereum chain head minus the last scanned block",
)
const fishermanLastScanTimestamp = gauge(
    "snowbridge_fisherman_last_scan_timestamp_seconds",
    "Unix timestamp of the last successful fisherman scan",
)
const fishermanEquivocations = new Counter({
    name: "snowbridge_fisherman_equivocations_total",
    help: "Equivocations detected by the fisherman",
    labelNames: ["kind"],
    registers: [register],
})

// Reset before repopulating so label sets that disappear stop being exported instead of
// freezing at their last value.
const bridgeGauges = [
    beefyLatency,
    latestBeefyBlock,
    beaconLatency,
    latestBeaconSlot,
    operatingMode,
    channelNonce,
    channelDeliveryEstimate,
    channelUndeliveredTimeout,
    relayerBalance,
    sovereignBalance,
    indexerLatency,
    poolLiquidity,
]

let lastRefresh = 0

const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms))

const updateBridgeMetrics = (metrics: AllMetrics) => {
    for (const g of bridgeGauges) {
        g.reset()
    }

    beefyLatency.set(metrics.bridgeStatus.toEthereum.latencySeconds)
    latestBeefyBlock.set(metrics.bridgeStatus.toEthereum.latestPolkadotBlockOnEthereum)
    beaconLatency.set(metrics.bridgeStatus.toPolkadot.latencySeconds)
    latestBeaconSlot.set(metrics.bridgeStatus.toPolkadot.latestBeaconSlotOnPolkadot)

    const halted = (mode: status.OperatingMode) => (mode === "Halted" ? 1 : 0)
    const toEthereumMode = metrics.bridgeStatus.toEthereum.operatingMode
    const toPolkadotMode = metrics.bridgeStatus.toPolkadot.operatingMode
    operatingMode.set(
        { direction: "to_ethereum", component: "outbound" },
        halted(toEthereumMode.outbound),
    )
    operatingMode.set(
        { direction: "to_polkadot", component: "beacon" },
        halted(toPolkadotMode.beacon),
    )
    operatingMode.set(
        { direction: "to_polkadot", component: "inbound" },
        halted(toPolkadotMode.inbound),
    )
    operatingMode.set(
        { direction: "to_polkadot", component: "outbound" },
        halted(toPolkadotMode.outbound),
    )

    for (const channel of metrics.channels) {
        const name = channel.name ?? "Unknown"
        const directions = [
            { direction: "to_ethereum", info: channel.toEthereum },
            { direction: "to_polkadot", info: channel.toPolkadot },
        ]
        for (const { direction, info } of directions) {
            channelNonce.set({ channel: name, direction, kind: "outbound" }, info.outbound)
            channelNonce.set({ channel: name, direction, kind: "inbound" }, info.inbound)
            if (info.estimatedDeliveryTime) {
                channelDeliveryEstimate.set(
                    { channel: name, direction },
                    info.estimatedDeliveryTime,
                )
            }
            if (info.undeliveredTimeout) {
                channelUndeliveredTimeout.set({ channel: name, direction }, info.undeliveredTimeout)
            }
        }
    }

    for (const relayer of metrics.relayers) {
        relayerBalance.set({ relayer: relayer.name, type: relayer.type }, Number(relayer.balance))
    }
    for (const sovereign of metrics.sovereigns) {
        sovereignBalance.set(
            { sovereign: sovereign.name, type: sovereign.type },
            Number(sovereign.balance),
        )
    }
    for (const indexer of metrics.indexerStatus) {
        indexerLatency.set({ chain: indexer.chain }, Number(indexer.latency))
    }
    for (const pool of metrics.liquidityPools) {
        poolLiquidity.set(
            { chain: pool.chain, pool: pool.name, asset: "DOT" },
            Number(pool.dotBalance),
        )
        poolLiquidity.set(
            { chain: pool.chain, pool: pool.name, asset: "ETH" },
            Number(pool.etherBalance),
        )
    }

    collectionSectionOk.set({ section: "bridge_status" }, sectionOk.bridgeStatus ? 1 : 0)
    collectionSectionOk.set({ section: "channels" }, sectionOk.channels ? 1 : 0)
    collectionSectionOk.set({ section: "balances" }, sectionOk.balances ? 1 : 0)
    collectionSectionOk.set({ section: "indexer" }, sectionOk.indexer ? 1 : 0)
    collectionSectionOk.set({ section: "pools" }, sectionOk.pools ? 1 : 0)
}

const updateFishermanMetrics = (scan: fisherman.FishermanScanResult) => {
    fishermanEquivocations.inc({ kind: "ForkVoting" }, scan.detected.ForkVoting)
    fishermanEquivocations.inc({ kind: "FutureBlockVoting" }, scan.detected.FutureBlockVoting)
    fishermanLastScannedBlock.set(scan.lastScannedBlock)
    fishermanScanLag.set(scan.latestEthereumBlock - scan.lastScannedBlock)
    fishermanLastScanTimestamp.set(Date.now() / 1000)
}

const tick = async () => {
    const started = Date.now()

    const metrics = await collectMetrics()
    updateBridgeMetrics(metrics)

    if (FishermanEnabled) {
        const scan = await fisherman.run(false)
        updateFishermanMetrics(scan)
        collectionSectionOk.set({ section: "fisherman" }, 1)
    }

    lastRefresh = Date.now()
    lastRefreshTimestamp.set(lastRefresh / 1000)
    refreshDuration.set((lastRefresh - started) / 1000)
}

export const runExporter = async (): Promise<void> => {
    if (FishermanEnabled) {
        const checkpoint = await fisherman.loadCheckPoint()
        fishermanEquivocations.inc({ kind: "ForkVoting" }, checkpoint.equivocations.ForkVoting)
        fishermanEquivocations.inc(
            { kind: "FutureBlockVoting" },
            checkpoint.equivocations.FutureBlockVoting,
        )
        fishermanLastScannedBlock.set(checkpoint.lastProcessedBlock)
    }

    const server = createServer(async (req, res) => {
        if (!req.url?.startsWith("/metrics")) {
            res.statusCode = 404
            res.end()
            return
        }
        // Computed here rather than from the timestamp gauge so that clock skew between
        // this container and Prometheus cannot fake or mask a staleness alarm.
        secondsSinceRefresh.set(lastRefresh ? (Date.now() - lastRefresh) / 1000 : -1)
        res.setHeader("Content-Type", register.contentType)
        res.end(await register.metrics())
    })
    server.listen(Port, Host)
    console.log(`Serving metrics on ${Host}:${Port}/metrics`)

    process.on("SIGTERM", () => process.exit(0))
    process.on("SIGINT", () => process.exit(0))

    while (true) {
        try {
            // A hung RPC never rejects, so without this the loop would park forever and
            // keep serving a stale snapshot.
            await Promise.race([
                tick(),
                sleep(TickTimeout).then(() => {
                    throw Error(`Refresh exceeded ${TickTimeout / 1000}s`)
                }),
            ])
            refreshes.inc({ result: "success" })
            console.log("Refresh succeeded.")
        } catch (error) {
            refreshes.inc({ result: "failure" })
            console.error("Refresh failed:", error)
            process.exit(1)
        }
        await sleep(RefreshInterval)
    }
}

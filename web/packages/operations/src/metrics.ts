import { status } from "@snowbridge/api"

import { MonitorRelayer } from "./monitorConfig"

export type LiquidityPoolMetrics = {
    name: string
    chain: string
    dotBalance: bigint
    etherBalance: bigint
}

export type AllMetrics = {
    name: string
    bridgeStatus: status.BridgeStatusInfo
    channels: status.ChannelStatusInfo[]
    sovereigns: status.Sovereign[]
    relayers: MonitorRelayer[]
    indexerStatus: status.IndexerServiceStatusInfo[]
    liquidityPools: LiquidityPoolMetrics[]
}

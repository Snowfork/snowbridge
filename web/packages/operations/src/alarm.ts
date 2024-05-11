import { status, environment } from "@snowbridge/api"
import axios from "axios"

const SLACK_WEBHOOK_URL = process.env["SLACK_WEBHOOK_URL"]

export const AlarmThreshold = {
    MaxBlockLatency: 1200,
    MinBalanceToKeep: 10_000_000_000,
}

export type Sovereign = { name: string; account: string; balance: bigint }

export type AllMetrics = {
    bridgeStatus: status.BridgeStatusInfo
    channels: status.ChannelStatusInfo[]
    sovereigns: Sovereign[]
    relayers: environment.Relayer[]
}

export enum AlarmReason {
    BeefyStale = "BeefyStale",
    BeaconStale = "BeaconStale",
    ToEthereumChannelStale = "ToEthereumChannelStale",
    ToPolkadotChannelStale = "ToPolkadotChannelStale",
    AccountBalanceInsufficient = "AccountBalanceInsufficient",
}

export const sendAlarm = async (metrics: AllMetrics) => {
    let alarm = false
    let alarms = []

    if (
        metrics.bridgeStatus.toEthereum.blockLatency > AlarmThreshold.MaxBlockLatency ||
        metrics.bridgeStatus.toEthereum.latestPolkadotBlockOnEthereum ==
            metrics.bridgeStatus.toEthereum.previousPolkadotBlockOnEthereum
    ) {
        alarm = true
        alarms.push(AlarmReason.BeefyStale)
    }
    if (
        metrics.bridgeStatus.toPolkadot.blockLatency > AlarmThreshold.MaxBlockLatency ||
        metrics.bridgeStatus.toPolkadot.latestEthereumBlockOnPolkadot ==
            metrics.bridgeStatus.toPolkadot.previousEthereumBlockOnPolkadot
    ) {
        alarm = true
        alarms.push(AlarmReason.BeaconStale)
    }
    for (let channel of metrics.channels) {
        if (
            channel.toEthereum.outbound != channel.toEthereum.inbound &&
            channel.toEthereum.inbound == channel.toEthereum.previousInbound
        ) {
            alarm = true
            alarms.push(AlarmReason.ToEthereumChannelStale)
        }
        if (
            channel.toPolkadot.outbound != channel.toPolkadot.inbound &&
            channel.toPolkadot.inbound == channel.toPolkadot.previousInbound
        ) {
            alarm = true
            alarms.push(AlarmReason.ToPolkadotChannelStale)
        }
        break
    }

    for (let relayer of metrics.relayers) {
        if (!relayer.balance || relayer.balance < AlarmThreshold.MinBalanceToKeep) {
            alarm = true
            alarms.push(AlarmReason.AccountBalanceInsufficient)
            break
        }
    }
    const text = JSON.stringify(
        { alarms, metrics },
        (key, value) => (typeof value === "bigint" ? value.toString() : value),
        2
    )
    console.log(text)

    if (alarm) {
        await axios.post(SLACK_WEBHOOK_URL || "", { text })
    }
}

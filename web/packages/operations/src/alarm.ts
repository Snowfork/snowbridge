import { status, environment } from "@snowbridge/api"
import axios from "axios"
import { CloudWatchClient, PutMetricDataCommand } from "@aws-sdk/client-cloudwatch"

const SLACK_WEBHOOK_URL = process.env["SLACK_WEBHOOK_URL"]
const CLOUD_WATCH_NAME_SPACE = "SnowbridgeMetrics"

export const AlarmThreshold = {
    MaxBlockLatency: 2000,
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

export type ChannelKind = "Primary" | "Secondary" | "AssetHub"

export const sendMetrics = async (metrics: AllMetrics) => {
    let client = new CloudWatchClient({})
    let metricData = []
    // Beefy metrics
    metricData.push({
        MetricName: "BeefyLatency",
        Value: metrics.bridgeStatus.toEthereum.blockLatency,
    })
    metricData.push({
        MetricName: "LatestBeefyBlock",
        Value: metrics.bridgeStatus.toEthereum.latestPolkadotBlockOnEthereum,
    })
    metricData.push({
        MetricName: "PreviousBeefyBlock",
        Value: metrics.bridgeStatus.toEthereum.previousPolkadotBlockOnEthereum,
    })
    // Beacon metrics
    metricData.push({
        MetricName: "BeaconLatency",
        Value: metrics.bridgeStatus.toPolkadot.blockLatency,
    })
    metricData.push({
        MetricName: "LatestBeaconBlock",
        Value: metrics.bridgeStatus.toPolkadot.latestEthereumBlockOnPolkadot,
    })
    metricData.push({
        MetricName: "PreviousBeaconBlock",
        Value: metrics.bridgeStatus.toPolkadot.previousEthereumBlockOnPolkadot,
    })
    for (let channel of metrics.channels) {
        metricData.push({
            MetricName: "OutboundNonce",
            Dimensions: [
                {
                    Name: "Direction",
                    Value: "ToEthereum",
                },
            ],
            Value: channel.toEthereum.outbound,
        })
        metricData.push({
            MetricName: "PreviousOutboundNonce",
            Dimensions: [
                {
                    Name: "Direction",
                    Value: "ToEthereum",
                },
            ],
            Value: channel.toEthereum.previousOutbound,
        })
        metricData.push({
            MetricName: "InboundNonce",
            Dimensions: [
                {
                    Name: "Direction",
                    Value: "ToEthereum",
                },
            ],
            Value: channel.toEthereum.inbound,
        })
        metricData.push({
            MetricName: "PreviousInboundNonce",
            Dimensions: [
                {
                    Name: "Direction",
                    Value: "ToEthereum",
                },
            ],
            Value: channel.toEthereum.previousInbound,
        })

        metricData.push({
            MetricName: "OutboundNonce",
            Dimensions: [
                {
                    Name: "Direction",
                    Value: "ToPolkadot",
                },
            ],
            Value: channel.toPolkadot.outbound,
        })
        metricData.push({
            MetricName: "PreviousOutboundNonce",
            Dimensions: [
                {
                    Name: "Direction",
                    Value: "ToPolkadot",
                },
            ],
            Value: channel.toPolkadot.previousOutbound,
        })
        metricData.push({
            MetricName: "InboundNonce",
            Dimensions: [
                {
                    Name: "Direction",
                    Value: "ToPolkadot",
                },
            ],
            Value: channel.toPolkadot.inbound,
        })
        metricData.push({
            MetricName: "PreviousInboundNonce",
            Dimensions: [
                {
                    Name: "Direction",
                    Value: "ToPolkadot",
                },
            ],
            Value: channel.toPolkadot.previousInbound,
        })
    }
    for (let relayer of metrics.relayers) {
        metricData.push({
            MetricName: "BalanceOfRelayer",
            Dimensions: [
                {
                    Name: "Name",
                    Value: relayer.name,
                },
            ],
            Value: Number(relayer.balance),
        })
    }
    for (let sovereign of metrics.sovereigns) {
        metricData.push({
            MetricName: "BalanceOfSovereign",
            Dimensions: [
                {
                    Name: "Name",
                    Value: sovereign.name,
                },
            ],
            Value: Number(sovereign.balance),
        })
    }
    const command = new PutMetricDataCommand({
        MetricData: metricData,
        Namespace: CLOUD_WATCH_NAME_SPACE,
    })
    await client.send(command)
}

export const sendAlarm = async (metrics: AllMetrics) => {
    let alarm = false
    let alarms = []

    if (
        metrics.bridgeStatus.toEthereum.blockLatency > AlarmThreshold.MaxBlockLatency &&
        metrics.bridgeStatus.toEthereum.latestPolkadotBlockOnEthereum ==
            metrics.bridgeStatus.toEthereum.previousPolkadotBlockOnEthereum
    ) {
        alarm = true
        alarms.push(AlarmReason.BeefyStale)
    }
    if (
        metrics.bridgeStatus.toPolkadot.blockLatency > AlarmThreshold.MaxBlockLatency &&
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

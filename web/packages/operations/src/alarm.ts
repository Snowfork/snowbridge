import { status, environment } from "@snowbridge/api"
import axios from "axios"
import {
    CloudWatchClient,
    PutMetricDataCommand,
    PutMetricAlarmCommand,
} from "@aws-sdk/client-cloudwatch"

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
        Dimensions: [
            {
                Name: "Direction",
                Value: "ToEthereum",
            },
        ],
        Value: metrics.bridgeStatus.toEthereum.blockLatency,
    })
    metricData.push({
        MetricName: "LatestBeefyBlock",
        Dimensions: [
            {
                Name: "Direction",
                Value: "ToEthereum",
            },
        ],
        Value: metrics.bridgeStatus.toEthereum.latestPolkadotBlockOnEthereum,
    })
    metricData.push({
        MetricName: "PreviousBeefyBlock",
        Dimensions: [
            {
                Name: "Direction",
                Value: "ToEthereum",
            },
        ],
        Value: metrics.bridgeStatus.toEthereum.previousPolkadotBlockOnEthereum,
    })
    metricData.push({
        MetricName: AlarmReason.BeefyStale.toString(),
        Dimensions: [
            {
                Name: "Direction",
                Value: "ToEthereum",
            },
        ],
        Value: Number(
            metrics.bridgeStatus.toEthereum.blockLatency > AlarmThreshold.MaxBlockLatency &&
                metrics.bridgeStatus.toEthereum.latestPolkadotBlockOnEthereum <=
                    metrics.bridgeStatus.toEthereum.previousPolkadotBlockOnEthereum
        ),
    })
    // Beacon metrics
    metricData.push({
        MetricName: "BeaconLatency",
        Dimensions: [
            {
                Name: "Direction",
                Value: "ToPolkadot",
            },
        ],
        Value: metrics.bridgeStatus.toPolkadot.blockLatency,
    })
    metricData.push({
        MetricName: "LatestBeaconBlock",
        Dimensions: [
            {
                Name: "Direction",
                Value: "ToPolkadot",
            },
        ],
        Value: metrics.bridgeStatus.toPolkadot.latestEthereumBlockOnPolkadot,
    })
    metricData.push({
        MetricName: "PreviousBeaconBlock",
        Dimensions: [
            {
                Name: "Direction",
                Value: "ToPolkadot",
            },
        ],
        Value: metrics.bridgeStatus.toPolkadot.previousEthereumBlockOnPolkadot,
    })
    metricData.push({
        MetricName: AlarmReason.BeaconStale.toString(),
        Dimensions: [
            {
                Name: "Direction",
                Value: "ToPolkadot",
            },
        ],
        Value: Number(
            metrics.bridgeStatus.toPolkadot.blockLatency > AlarmThreshold.MaxBlockLatency &&
                metrics.bridgeStatus.toPolkadot.latestEthereumBlockOnPolkadot <=
                    metrics.bridgeStatus.toPolkadot.previousEthereumBlockOnPolkadot
        ),
    })
    // Channel metrics
    for (let channel of metrics.channels) {
        // To Ethereum
        metricData.push({
            MetricName: "ToEthereumOutboundNonce",
            Dimensions: [
                {
                    Name: "Direction",
                    Value: "ToEthereum",
                },
                {
                    Name: "ChannelName",
                    Value: channel.name,
                },
            ],
            Value: channel.toEthereum.outbound,
        })
        metricData.push({
            MetricName: "ToEthereumPreviousOutboundNonce",
            Dimensions: [
                {
                    Name: "Direction",
                    Value: "ToEthereum",
                },
                {
                    Name: "ChannelName",
                    Value: channel.name,
                },
            ],
            Value: channel.toEthereum.previousOutbound,
        })
        metricData.push({
            MetricName: "ToEthereumInboundNonce",
            Dimensions: [
                {
                    Name: "Direction",
                    Value: "ToEthereum",
                },
                {
                    Name: "ChannelName",
                    Value: channel.name,
                },
            ],
            Value: channel.toEthereum.inbound,
        })
        metricData.push({
            MetricName: "ToEthereumPreviousInboundNonce",
            Dimensions: [
                {
                    Name: "Direction",
                    Value: "ToEthereum",
                },
                {
                    Name: "ChannelName",
                    Value: channel.name,
                },
            ],
            Value: channel.toEthereum.previousInbound,
        })
        metricData.push({
            MetricName: AlarmReason.ToEthereumChannelStale.toString(),
            Dimensions: [
                {
                    Name: "Direction",
                    Value: "ToEthereum",
                },
            ],
            Value: Number(
                channel.toEthereum.outbound < channel.toEthereum.inbound ||
                    (channel.toEthereum.outbound > channel.toEthereum.inbound &&
                        channel.toEthereum.inbound <= channel.toEthereum.previousInbound)
            ),
        })
        // To Polkadot
        metricData.push({
            MetricName: "ToPolkadotOutboundNonce",
            Dimensions: [
                {
                    Name: "Direction",
                    Value: "ToPolkadot",
                },
                {
                    Name: "ChannelName",
                    Value: channel.name,
                },
            ],
            Value: channel.toPolkadot.outbound,
        })
        metricData.push({
            MetricName: "ToPolkadotPreviousOutboundNonce",
            Dimensions: [
                {
                    Name: "Direction",
                    Value: "ToPolkadot",
                },
                {
                    Name: "ChannelName",
                    Value: channel.name,
                },
            ],
            Value: channel.toPolkadot.previousOutbound,
        })
        metricData.push({
            MetricName: "ToPolkadotInboundNonce",
            Dimensions: [
                {
                    Name: "Direction",
                    Value: "ToPolkadot",
                },
                {
                    Name: "ChannelName",
                    Value: channel.name,
                },
            ],
            Value: channel.toPolkadot.inbound,
        })
        metricData.push({
            MetricName: "ToPolkadotPreviousInboundNonce",
            Dimensions: [
                {
                    Name: "Direction",
                    Value: "ToPolkadot",
                },
                {
                    Name: "ChannelName",
                    Value: channel.name,
                },
            ],
            Value: channel.toPolkadot.previousInbound,
        })
        metricData.push({
            MetricName: AlarmReason.ToPolkadotChannelStale.toString(),
            Dimensions: [
                {
                    Name: "Direction",
                    Value: "ToPolkadot",
                },
            ],
            Value: Number(
                channel.toPolkadot.outbound < channel.toPolkadot.inbound ||
                    (channel.toPolkadot.outbound > channel.toPolkadot.inbound &&
                        channel.toPolkadot.inbound <= channel.toPolkadot.previousInbound)
            ),
        })
    }
    for (let relayer of metrics.relayers) {
        metricData.push({
            MetricName: "BalanceOfRelayer",
            Dimensions: [
                {
                    Name: "RelayerName",
                    Value: relayer.name,
                },
            ],
            Value: Number(relayer.balance),
        })
        metricData.push({
            MetricName: AlarmReason.AccountBalanceInsufficient.toString(),
            Value: Number(!relayer.balance || relayer.balance < AlarmThreshold.MinBalanceToKeep),
        })
    }
    for (let sovereign of metrics.sovereigns) {
        metricData.push({
            MetricName: "BalanceOfSovereign",
            Dimensions: [
                {
                    Name: "SovereignName",
                    Value: sovereign.name,
                },
            ],
            Value: Number(sovereign.balance),
        })
        metricData.push({
            MetricName: AlarmReason.AccountBalanceInsufficient.toString(),
            Value: Number(
                !sovereign.balance || sovereign.balance < AlarmThreshold.MinBalanceToKeep
            ),
        })
    }
    const command = new PutMetricDataCommand({
        MetricData: metricData,
        Namespace: CLOUD_WATCH_NAME_SPACE,
    })
    await client.send(command)
}

export const initializeAlarms = async () => {
    let client = new CloudWatchClient({})
    let cloudWatchAlarms = []
    let alarmCommandSharedInput = {
        EvaluationPeriods: 3,
        Namespace: CLOUD_WATCH_NAME_SPACE,
        Period: 60,
        Threshold: 0,
    }
    cloudWatchAlarms.push(
        new PutMetricAlarmCommand({
            AlarmName: AlarmReason.BeefyStale.toString(),
            MetricName: AlarmReason.BeefyStale.toString(),
            AlarmDescription: AlarmReason.BeefyStale.toString(),
            Statistic: "Average",
            ComparisonOperator: "GreaterThanThreshold",
            ...alarmCommandSharedInput,
        })
    )
    cloudWatchAlarms.push(
        new PutMetricAlarmCommand({
            AlarmName: AlarmReason.BeaconStale.toString(),
            MetricName: AlarmReason.BeaconStale.toString(),
            AlarmDescription: AlarmReason.BeaconStale.toString(),
            Statistic: "Average",
            ComparisonOperator: "GreaterThanThreshold",
            ...alarmCommandSharedInput,
        })
    )
    cloudWatchAlarms.push(
        new PutMetricAlarmCommand({
            AlarmName: AlarmReason.ToEthereumChannelStale.toString(),
            MetricName: AlarmReason.ToEthereumChannelStale.toString(),
            AlarmDescription: AlarmReason.ToEthereumChannelStale.toString(),
            Statistic: "Average",
            ComparisonOperator: "GreaterThanThreshold",
            ...alarmCommandSharedInput,
        })
    )
    cloudWatchAlarms.push(
        new PutMetricAlarmCommand({
            AlarmName: AlarmReason.ToPolkadotChannelStale.toString(),
            MetricName: AlarmReason.ToPolkadotChannelStale.toString(),
            AlarmDescription: AlarmReason.ToPolkadotChannelStale.toString(),
            Statistic: "Average",
            ComparisonOperator: "GreaterThanThreshold",
            ...alarmCommandSharedInput,
        })
    )
    cloudWatchAlarms.push(
        new PutMetricAlarmCommand({
            AlarmName: AlarmReason.AccountBalanceInsufficient.toString(),
            MetricName: AlarmReason.AccountBalanceInsufficient.toString(),
            AlarmDescription: AlarmReason.AccountBalanceInsufficient.toString(),
            Statistic: "Average",
            ComparisonOperator: "GreaterThanThreshold",
            ...alarmCommandSharedInput,
        })
    )
    for (let alarm of cloudWatchAlarms) {
        await client.send(alarm)
    }
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

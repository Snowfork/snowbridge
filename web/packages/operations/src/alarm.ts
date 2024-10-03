import { status, environment } from "@snowbridge/api"
import {
    CloudWatchClient,
    PutMetricDataCommand,
    PutMetricAlarmCommand,
} from "@aws-sdk/client-cloudwatch"

const CLOUD_WATCH_NAME_SPACE = "SnowbridgeMetrics"
const BRIDGE_STALE_SNS_TOPIC = process.env["BRIDGE_STALE_SNS_TOPIC"] || ""
const ACCOUNT_BALANCE_SNS_TOPIC = process.env["ACCOUNT_BALANCE_SNS_TOPIC"] || ""

const LatencyDashboard =
    "https://eu-central-1.console.aws.amazon.com/cloudwatch/home?region=eu-central-1#dashboards/dashboard/Latency"
const BalanceDashboard =
    "https://eu-central-1.console.aws.amazon.com/cloudwatch/home?region=eu-central-1#dashboards/dashboard/Balance"

export const sendMetrics = async (metrics: status.AllMetrics) => {
    const { AlarmReason, InsufficientBalanceThreshold } = status
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
    metricData.push({
        MetricName: AlarmReason.BeefyStale.toString(),
        Value: Number(
            metrics.bridgeStatus.toEthereum.blockLatency >
                status.BlockLatencyThreshold.ToEthereum &&
                metrics.bridgeStatus.toEthereum.latestPolkadotBlockOnEthereum <=
                    metrics.bridgeStatus.toEthereum.previousPolkadotBlockOnEthereum
        ),
    })
    // Beacon metrics
    metricData.push({
        MetricName: "BeaconLatency",
        Value: metrics.bridgeStatus.toPolkadot.blockLatency,
    })
    metricData.push({
        MetricName: "LatestBeaconBlock",
        Value: metrics.bridgeStatus.toPolkadot.latestBeaconSlotOnPolkadot,
    })
    metricData.push({
        MetricName: "PreviousBeaconBlock",
        Value: metrics.bridgeStatus.toPolkadot.previousEthereumBlockOnPolkadot,
    })
    metricData.push({
        MetricName: AlarmReason.BeaconStale.toString(),
        Value: Number(
            metrics.bridgeStatus.toPolkadot.blockLatency >
                status.BlockLatencyThreshold.ToPolkadot &&
                metrics.bridgeStatus.toPolkadot.latestBeaconSlotOnPolkadot <=
                    metrics.bridgeStatus.toPolkadot.previousEthereumBlockOnPolkadot
        ),
    })
    // Channel metrics
    for (let channel of metrics.channels) {
        // Only monitor AH channel
        if (channel.name != status.ChannelKind.AssetHub) {
            continue
        }
        // To Ethereum
        metricData.push({
            MetricName: "ToEthereumOutboundNonce",
            Dimensions: [
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
                    Name: "ChannelName",
                    Value: channel.name,
                },
            ],
            Value: channel.toEthereum.previousInbound,
        })
        metricData.push({
            MetricName: AlarmReason.ToEthereumChannelStale.toString(),
            Value: Number(
                channel.toEthereum.outbound < channel.toEthereum.inbound ||
                    (channel.toEthereum.outbound > channel.toEthereum.inbound &&
                        channel.toEthereum.inbound <= channel.toEthereum.previousInbound)
            ),
        })
        metricData.push({
            MetricName: AlarmReason.ToEthereumNoTransfer.toString(),
            Value: Number(
                channel.toEthereum.inbound == channel.toEthereum.previousInbound
            ),
        })
        // To Polkadot
        metricData.push({
            MetricName: "ToPolkadotOutboundNonce",
            Dimensions: [
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
                    Name: "ChannelName",
                    Value: channel.name,
                },
            ],
            Value: channel.toPolkadot.previousInbound,
        })
        metricData.push({
            MetricName: AlarmReason.ToPolkadotChannelStale.toString(),
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
        if (relayer.type == "substrate") {
            metricData.push({
                MetricName: AlarmReason.AccountBalanceInsufficient.toString(),
                Value: Number(
                    !relayer.balance || relayer.balance < InsufficientBalanceThreshold.Substrate
                ),
            })
        }
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
        if (sovereign.type == "substrate") {
            metricData.push({
                MetricName: AlarmReason.AccountBalanceInsufficient.toString(),
                Value: Number(
                    !sovereign.balance || sovereign.balance < InsufficientBalanceThreshold.Substrate
                ),
            })
        }
    }
    const command = new PutMetricDataCommand({
        MetricData: metricData,
        Namespace: CLOUD_WATCH_NAME_SPACE + "-" + metrics.name,
    })
    await client.send(command)
}

export const initializeAlarms = async () => {
    const { AlarmReason } = status
    let env = "local_e2e"
    if (process.env.NODE_ENV !== undefined) {
        env = process.env.NODE_ENV
    }
    const snowbridgeEnv = environment.SNOWBRIDGE_ENV[env]
    if (snowbridgeEnv === undefined) {
        throw Error(`Unknown environment '${env}'`)
    }
    const { name } = snowbridgeEnv

    let client = new CloudWatchClient({})
    let cloudWatchAlarms = []
    let alarmCommandSharedInput = {
        Namespace: CLOUD_WATCH_NAME_SPACE + "-" + name,
        Threshold: 0,
        TreatMissingData: "breaching",
    }

    // Alarm for stale bridge
    cloudWatchAlarms.push(
        new PutMetricAlarmCommand({
            AlarmName: AlarmReason.BeefyStale.toString() + "-" + name,
            MetricName: AlarmReason.BeefyStale.toString(),
            AlarmDescription: LatencyDashboard,
            Statistic: "Average",
            ComparisonOperator: "GreaterThanThreshold",
            AlarmActions: [BRIDGE_STALE_SNS_TOPIC],
            EvaluationPeriods: 5,
            Period: 3600,
            ...alarmCommandSharedInput,
        })
    )
    cloudWatchAlarms.push(
        new PutMetricAlarmCommand({
            AlarmName: AlarmReason.BeaconStale.toString() + "-" + name,
            MetricName: AlarmReason.BeaconStale.toString(),
            AlarmDescription: LatencyDashboard,
            Statistic: "Average",
            ComparisonOperator: "GreaterThanThreshold",
            AlarmActions: [BRIDGE_STALE_SNS_TOPIC],
            EvaluationPeriods: 3,
            Period: 1800,
            ...alarmCommandSharedInput,
        })
    )
    cloudWatchAlarms.push(
        new PutMetricAlarmCommand({
            AlarmName: AlarmReason.ToEthereumChannelStale.toString() + "-" + name,
            MetricName: AlarmReason.ToEthereumChannelStale.toString(),
            AlarmDescription: LatencyDashboard,
            Statistic: "Average",
            ComparisonOperator: "GreaterThanThreshold",
            AlarmActions: [BRIDGE_STALE_SNS_TOPIC],
            EvaluationPeriods: 5,
            Period: 3600,
            ...alarmCommandSharedInput,
        })
    )
    cloudWatchAlarms.push(
        new PutMetricAlarmCommand({
            AlarmName: AlarmReason.ToPolkadotChannelStale.toString() + "-" + name,
            MetricName: AlarmReason.ToPolkadotChannelStale.toString(),
            AlarmDescription: LatencyDashboard,
            Statistic: "Average",
            ComparisonOperator: "GreaterThanThreshold",
            AlarmActions: [BRIDGE_STALE_SNS_TOPIC],
            EvaluationPeriods: 3,
            Period: 1800,
            ...alarmCommandSharedInput,
        })
    )
    for (let alarm of cloudWatchAlarms) {
        await client.send(alarm)
    }

    // Alarm for account balance insufficient
    let accountBalanceAlarm = new PutMetricAlarmCommand({
        AlarmName: AlarmReason.AccountBalanceInsufficient.toString() + "-" + name,
        MetricName: AlarmReason.AccountBalanceInsufficient.toString(),
        AlarmDescription: BalanceDashboard,
        Statistic: "Average",
        ComparisonOperator: "GreaterThanThreshold",
        AlarmActions: [ACCOUNT_BALANCE_SNS_TOPIC],
        EvaluationPeriods: 1,
        Period: 1800,
        ...alarmCommandSharedInput,
    })
    await client.send(accountBalanceAlarm)

    console.log("Initialize alarm rules success.")
}

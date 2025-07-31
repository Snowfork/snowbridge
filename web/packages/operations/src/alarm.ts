import { status, environment } from "@snowbridge/api"
import {
    CloudWatchClient,
    PutMetricDataCommand,
    PutMetricAlarmCommand,
} from "@aws-sdk/client-cloudwatch"

const CLOUD_WATCH_NAME_SPACE = "SnowbridgeMetrics"
const BRIDGE_STALE_SNS_TOPIC = process.env["BRIDGE_STALE_SNS_TOPIC"] || ""
const BRIDGE_ATTACKED_SNS_TOPIC = process.env["BRIDGE_ATTACKED_SNS_TOPIC"] || ""
const ACCOUNT_BALANCE_SNS_TOPIC = process.env["ACCOUNT_BALANCE_SNS_TOPIC"] || ""

const LatencyDashboard =
    process.env["LATENCY_DASHBOARD_URL"] ||
    "https://eu-central-1.console.aws.amazon.com/cloudwatch/home?region=eu-central-1#dashboards/dashboard/Latency"
const BalanceDashboard =
    process.env["BALANCE_DASHBOARD_URL"] ||
    "https://eu-central-1.console.aws.amazon.com/cloudwatch/home?region=eu-central-1#dashboards/dashboard/Balance"

export enum AlarmReason {
    BeefyStale = "BeefyStale",
    BeaconStale = "BeaconStale",
    ToEthereumChannelStale = "ToEthereumChannelStale",
    ToPolkadotChannelStale = "ToPolkadotChannelStale",
    RelayAccountBalanceInsufficient = "RelayAccountBalanceInsufficient",
    SovereignAccountBalanceInsufficient = "SovereignAccountBalanceInsufficient",
    IndexServiceStale = "IndexServiceStale",
    HeartbeatLost = "HeartbeatLost",
}

export const InsufficientBalanceThreshold = {
    // Minimum as 100 DOT
    Substrate: process.env["SubstrateBalanceThreshold"]
        ? parseInt(process.env["SubstrateBalanceThreshold"])
        : 1_000_000_000_000,
    // Minimum as 0.3 Ether
    Ethereum: process.env["EthereumBalanceThreshold"]
        ? parseInt(process.env["EthereumBalanceThreshold"])
        : 300_000_000_000_000_000,
}

// This configuration is for setting up a CloudWatch alarm.
// EvaluationPeriods: The number of most recent periods (data points) to evaluate when determining the alarm state.
// DatapointsToAlarm: The number of data points within the evaluation periods that must breach the threshold to trigger the alarm.
// For more details, see: https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/AlarmThatSendsEmail.html
export const AlarmEvaluationConfiguration = {
    ToEthereumStale: {
        EvaluationPeriods: process.env["ToEthereumEvaluationPeriods"]
            ? parseInt(process.env["ToEthereumEvaluationPeriods"])
            : 8,
        DatapointsToAlarm: process.env["ToEthereumDatapointsToAlarm"]
            ? parseInt(process.env["ToEthereumDatapointsToAlarm"])
            : 6,
    },
    ToPolkadotStale: {
        EvaluationPeriods: process.env["ToPolkadotEvaluationPeriods"]
            ? parseInt(process.env["ToPolkadotEvaluationPeriods"])
            : 8,
        DatapointsToAlarm: process.env["ToPolkadotDatapointsToAlarm"]
            ? parseInt(process.env["ToPolkadotDatapointsToAlarm"])
            : 6,
    },
}

export const IndexerLatencyThreshold = process.env["IndexerLatencyThreshold"]
    ? parseInt(process.env["IndexerLatencyThreshold"])
    : 150

export const ScanInterval = process.env["SCAN_INTERVAL"]
    ? parseInt(process.env["SCAN_INTERVAL"])
    : 900

export const sendMetrics = async (metrics: status.AllMetrics) => {
    let client = new CloudWatchClient({})
    let metricData = []
    // Heartbeat metrics
    metricData.push({
        MetricName: "Heartbeat",
        Value: 1,
    })
    // Beefy metrics
    metricData.push({
        MetricName: "BeefyLatency",
        Value: metrics.bridgeStatus.toEthereum.latencySeconds,
    })
    metricData.push({
        MetricName: "LatestBeefyBlock",
        Value: metrics.bridgeStatus.toEthereum.latestPolkadotBlockOnEthereum,
    })
    // Beacon metrics
    metricData.push({
        MetricName: "BeaconLatency",
        Value: metrics.bridgeStatus.toPolkadot.latencySeconds,
    })
    metricData.push({
        MetricName: "LatestBeaconSlot",
        Value: metrics.bridgeStatus.toPolkadot.latestBeaconSlotOnPolkadot,
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
            MetricName: "ToEthereumInboundNonce",
            Dimensions: [
                {
                    Name: "ChannelName",
                    Value: channel.name,
                },
            ],
            Value: channel.toEthereum.inbound,
        })
        if (channel.toEthereum.estimatedDeliveryTime) {
            metricData.push({
                MetricName: "ToEthereumDeliveryEstimate",
                Dimensions: [
                    {
                        Name: "ChannelName",
                        Value: channel.name,
                    },
                ],
                Value: channel.toEthereum.estimatedDeliveryTime,
            })
        }
        if (channel.toEthereum.undeliveredTimeout) {
            metricData.push({
                MetricName: "ToEthereumUndeliveredTimeout",
                Dimensions: [
                    {
                        Name: "ChannelName",
                        Value: channel.name,
                    },
                ],
                Value: channel.toEthereum.undeliveredTimeout,
            })
        }
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
            MetricName: "ToPolkadotInboundNonce",
            Dimensions: [
                {
                    Name: "ChannelName",
                    Value: channel.name,
                },
            ],
            Value: channel.toPolkadot.inbound,
        })
        if (channel.toPolkadot.estimatedDeliveryTime) {
            metricData.push({
                MetricName: "ToPolkadotDeliveryEstimate",
                Dimensions: [
                    {
                        Name: "ChannelName",
                        Value: channel.name,
                    },
                ],
                Value: channel.toPolkadot.estimatedDeliveryTime,
            })
        }
        if (channel.toPolkadot.undeliveredTimeout) {
            metricData.push({
                MetricName: "ToPolkadotUndeliveredTimeout",
                Dimensions: [
                    {
                        Name: "ChannelName",
                        Value: channel.name,
                    },
                ],
                Value: channel.toPolkadot.undeliveredTimeout,
            })
        }
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
    }
    for (let status of metrics.indexerStatus) {
        metricData.push({
            MetricName: "IndexerLatency",
            Dimensions: [
                {
                    Name: "ChainName",
                    Value: status.chain,
                },
            ],
            Value: Number(status.latency),
        })
    }
    const command = new PutMetricDataCommand({
        MetricData: metricData,
        Namespace: CLOUD_WATCH_NAME_SPACE + "-" + metrics.name,
    })
    await client.send(command)
}

export const initializeAlarms = async () => {
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
    let alarmCommandSharedInput: any = {
        Namespace: CLOUD_WATCH_NAME_SPACE + "-" + name,
        TreatMissingData: "notBreaching",
        Period: ScanInterval,
        Statistic: "Average",
        ComparisonOperator: "GreaterThanThreshold",
    }

    // Beefy stale
    cloudWatchAlarms.push(
        new PutMetricAlarmCommand({
            AlarmName: AlarmReason.BeefyStale.toString() + "-" + name,
            MetricName: "BeefyLatency",
            AlarmDescription: LatencyDashboard,
            AlarmActions: [BRIDGE_STALE_SNS_TOPIC],
            EvaluationPeriods: AlarmEvaluationConfiguration.ToEthereumStale.EvaluationPeriods,
            DatapointsToAlarm: AlarmEvaluationConfiguration.ToEthereumStale.DatapointsToAlarm,
            ...alarmCommandSharedInput,
            Threshold: 3600 * 4, // 1 epoch = 4 hours
        })
    )
    // Beacon stale
    cloudWatchAlarms.push(
        new PutMetricAlarmCommand({
            AlarmName: AlarmReason.BeaconStale.toString() + "-" + name,
            MetricName: "BeaconLatency",
            AlarmDescription: LatencyDashboard,
            AlarmActions: [BRIDGE_STALE_SNS_TOPIC],
            EvaluationPeriods: AlarmEvaluationConfiguration.ToPolkadotStale.EvaluationPeriods,
            DatapointsToAlarm: AlarmEvaluationConfiguration.ToPolkadotStale.DatapointsToAlarm,
            ...alarmCommandSharedInput,
            Threshold: 3 * 32 * 12, // 3 epochs = 3 * 6.4 mins ~= 20 mins
        })
    )

    // To Ethereum channel stale
    cloudWatchAlarms.push(
        new PutMetricAlarmCommand({
            AlarmName: AlarmReason.ToEthereumChannelStale.toString() + "-" + name,
            MetricName: "ToEthereumUndeliveredTimeout",
            AlarmDescription: LatencyDashboard,
            AlarmActions: [BRIDGE_STALE_SNS_TOPIC],
            EvaluationPeriods: AlarmEvaluationConfiguration.ToEthereumStale.EvaluationPeriods,
            DatapointsToAlarm: AlarmEvaluationConfiguration.ToEthereumStale.DatapointsToAlarm,
            ...alarmCommandSharedInput,
            Threshold: 5400, // 1.5 hours at most
        })
    )

    // To Polkadot channel stale
    cloudWatchAlarms.push(
        new PutMetricAlarmCommand({
            AlarmName: AlarmReason.ToPolkadotChannelStale.toString() + "-" + name,
            MetricName: "ToPolkadotUndeliveredTimeout",
            AlarmDescription: LatencyDashboard,
            AlarmActions: [BRIDGE_STALE_SNS_TOPIC],
            EvaluationPeriods: AlarmEvaluationConfiguration.ToPolkadotStale.EvaluationPeriods,
            DatapointsToAlarm: AlarmEvaluationConfiguration.ToPolkadotStale.DatapointsToAlarm,
            ...alarmCommandSharedInput,
            Threshold: 1800, // 0.5 hour
        })
    )

    for (let alarm of cloudWatchAlarms) {
        await client.send(alarm)
    }

    // Insufficient balance in the relay account
    let relayAccountBalanceAlarm = new PutMetricAlarmCommand({
        AlarmName: AlarmReason.RelayAccountBalanceInsufficient.toString() + "-" + name,
        MetricName: "BalanceOfRelayer",
        AlarmDescription: BalanceDashboard,
        AlarmActions: [ACCOUNT_BALANCE_SNS_TOPIC],
        EvaluationPeriods: 6,
        ...alarmCommandSharedInput,
        Threshold: InsufficientBalanceThreshold.Substrate,
    })
    await client.send(relayAccountBalanceAlarm)

    // Insufficient balance in the sovereign account
    let sovereignAccountBalanceAlarm = new PutMetricAlarmCommand({
        AlarmName: AlarmReason.SovereignAccountBalanceInsufficient.toString() + "-" + name,
        MetricName: "BalanceOfSovereign",
        AlarmDescription: BalanceDashboard,
        AlarmActions: [ACCOUNT_BALANCE_SNS_TOPIC],
        EvaluationPeriods: 6,
        ...alarmCommandSharedInput,
        Threshold: InsufficientBalanceThreshold.Substrate,
    })
    await client.send(sovereignAccountBalanceAlarm)

    // Indexer service stale
    let indexerAlarm = new PutMetricAlarmCommand({
        AlarmName: AlarmReason.IndexServiceStale.toString() + "-" + name,
        MetricName: "IndexerLatency",
        AlarmDescription: AlarmReason.IndexServiceStale.toString(),
        ComparisonOperator: "GreaterThanThreshold",
        AlarmActions: [BRIDGE_STALE_SNS_TOPIC],
        EvaluationPeriods: 6,
        ...alarmCommandSharedInput,
        Threshold: IndexerLatencyThreshold,
    })
    await client.send(indexerAlarm)

    // Heartbeat lost
    let heartbeartAlarm = new PutMetricAlarmCommand({
        AlarmName: AlarmReason.HeartbeatLost.toString() + "-" + name,
        MetricName: "Heartbeat",
        AlarmDescription: AlarmReason.HeartbeatLost.toString(),
        AlarmActions: [BRIDGE_STALE_SNS_TOPIC],
        EvaluationPeriods: 6,
        ...alarmCommandSharedInput,
        Threshold: 1,
        TreatMissingData: "breaching",
    })
    await client.send(heartbeartAlarm)
}

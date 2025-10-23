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
    process.env["LATENCY_DASHBOARD_URL"] ||
    "https://eu-central-1.console.aws.amazon.com/cloudwatch/home?region=eu-central-1#dashboards/dashboard/Latency?start=PT168H&end=null"
const BalanceDashboard =
    process.env["BALANCE_DASHBOARD_URL"] ||
    "https://eu-central-1.console.aws.amazon.com/cloudwatch/home?region=eu-central-1#dashboards/dashboard/Balance?start=PT168H&end=null"

export enum AlarmReason {
    BeefyStale = "BeefyStale",
    BeaconStale = "BeaconStale",
    ToEthereumChannelStale = "ToEthereumChannelStale",
    ToPolkadotChannelStale = "ToPolkadotChannelStale",
    RelayAccountBalanceInsufficient = "RelayAccountBalanceInsufficient",
    SovereignAccountBalanceInsufficient = "SovereignAccountBalanceInsufficient",
    IndexServiceStale = "IndexServiceStale",
    HeartbeatLost = "HeartbeatLost",
    ToPolkadotV2Stale = "ToPolkadotV2Stale",
    ToEthereumV2Stale = "ToEthereumV2Stale",
    FutureBlockVoting = "FutureBlockVoting",
    ForkVoting = "ForkVoting",
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
    EvaluationPeriods: process.env["EvaluationPeriods"]
        ? parseInt(process.env["EvaluationPeriods"])
        : 4,
    DatapointsToAlarm: process.env["DatapointsToAlarm"]
        ? parseInt(process.env["DatapointsToAlarm"])
        : 3,
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
    // V2 metrics
    if (metrics.v2Status?.toEthereum.estimatedDeliveryTime) {
        metricData.push({
            MetricName: "ToEthereumV2DeliveryEstimate",
            Value: metrics.v2Status?.toEthereum.estimatedDeliveryTime,
        })
    }
    if (metrics.v2Status?.toPolkadot.estimatedDeliveryTime) {
        metricData.push({
            MetricName: "ToPolkadotV2DeliveryEstimate",
            Value: metrics.v2Status?.toPolkadot.estimatedDeliveryTime,
        })
    }
    if (metrics.v2Status?.toEthereum.undeliveredTimeout) {
        metricData.push({
            MetricName: "ToEthereumV2UndeliveredTimeout",
            Value: metrics.v2Status?.toEthereum.undeliveredTimeout,
        })
    }
    if (metrics.v2Status?.toPolkadot.undeliveredTimeout) {
        metricData.push({
            MetricName: "ToPolkadotV2UndeliveredTimeout",
            Value: metrics.v2Status?.toPolkadot.undeliveredTimeout,
        })
    }
    const command = new PutMetricDataCommand({
        MetricData: metricData,
        Namespace: CLOUD_WATCH_NAME_SPACE + "-" + metrics.name,
    })
    console.log("Sent metrics:", JSON.stringify(metricData, null, 2))
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
        EvaluationPeriods: AlarmEvaluationConfiguration.EvaluationPeriods,
        DatapointsToAlarm: AlarmEvaluationConfiguration.DatapointsToAlarm,
    }

    // For alarms that need to trigger when an absolute value breaches a threshold.
    // For this case we dont want to wait for 3/4 datapoints in a 15 minute window(45 min)
    // as it will take a minimum of 45 minutes before we are alerted. We choose 5 minutes.
    // We use maximum statistic because we can about the maximum value breaching within a
    // time window.
    // e.g. bridge latency greater than x seconds.
    // e.g. nonce difference greater than x messages.
    let absoluteValueBreachingAlarmConfig: any = {
        Namespace: CLOUD_WATCH_NAME_SPACE + "-" + name,
        TreatMissingData: "notBreaching",
        Period: 60*5,
        Statistic: "Maximum",
        ComparisonOperator: "GreaterThanThreshold",
        EvaluationPeriods: 1,
        DatapointsToAlarm: 1,
    }

    // Beefy stale
    cloudWatchAlarms.push(
        new PutMetricAlarmCommand({
            AlarmName: AlarmReason.BeefyStale.toString() + "-" + name,
            MetricName: "BeefyLatency",
            AlarmDescription: LatencyDashboard,
            AlarmActions: [BRIDGE_STALE_SNS_TOPIC],
            ...alarmCommandSharedInput,
            Threshold: 3600 * 4, // 1 epoch = 4 hours
        }),
    )
    // Beacon stale
    cloudWatchAlarms.push(
        new PutMetricAlarmCommand({
            AlarmName: AlarmReason.BeaconStale.toString() + "-" + name,
            MetricName: "BeaconLatency",
            AlarmDescription: LatencyDashboard,
            AlarmActions: [BRIDGE_STALE_SNS_TOPIC],
            ...alarmCommandSharedInput,
            Threshold: 1500, // 3 epochs = 3 * 6.4 mins ~= 20 mins
        }),
    )

    // To Ethereum channel stale
    cloudWatchAlarms.push(
        new PutMetricAlarmCommand({
            AlarmName: AlarmReason.ToEthereumChannelStale.toString() + "-" + name,
            MetricName: "ToEthereumUndeliveredTimeout",
            Dimensions: [
                {
                    Name: "ChannelName",
                    Value: "AssetHub",
                },
            ],
            AlarmDescription: LatencyDashboard,
            AlarmActions: [BRIDGE_STALE_SNS_TOPIC],
            ...absoluteValueBreachingAlarmConfig,
            Threshold: 5400, // 1.5 hours at most
        }),
    )

    // To Polkadot channel stale
    cloudWatchAlarms.push(
        new PutMetricAlarmCommand({
            AlarmName: AlarmReason.ToPolkadotChannelStale.toString() + "-" + name,
            MetricName: "ToPolkadotUndeliveredTimeout",
            Dimensions: [
                {
                    Name: "ChannelName",
                    Value: "AssetHub",
                },
            ],
            AlarmDescription: LatencyDashboard,
            AlarmActions: [BRIDGE_STALE_SNS_TOPIC],
            ...absoluteValueBreachingAlarmConfig,
            Threshold: 1800, // 0.5 hour
        }),
    )

    // Insufficient balance in the relay account
    for (const relayName of ["beacon", "execution-assethub"]) {
        let relayAccountBalanceAlarm = new PutMetricAlarmCommand({
            AlarmName:
                AlarmReason.RelayAccountBalanceInsufficient.toString() +
                "-" +
                name +
                "-" +
                relayName,
            MetricName: "BalanceOfRelayer",
            Dimensions: [
                {
                    Name: "RelayerName",
                    Value: relayName,
                },
            ],
            AlarmDescription: BalanceDashboard,
            AlarmActions: [ACCOUNT_BALANCE_SNS_TOPIC],
            ...alarmCommandSharedInput,
            ComparisonOperator: "LessThanThreshold",
            Threshold: InsufficientBalanceThreshold.Substrate,
        })
        cloudWatchAlarms.push(relayAccountBalanceAlarm)
    }

    // Insufficient balance in the sovereign account
    let sovereignAccountBalanceAlarm = new PutMetricAlarmCommand({
        AlarmName: AlarmReason.SovereignAccountBalanceInsufficient.toString() + "-" + name,
        MetricName: "BalanceOfSovereign",
        Dimensions: [
            {
                Name: "SovereignName",
                Value: "AssetHub",
            },
        ],
        AlarmDescription: BalanceDashboard,
        AlarmActions: [ACCOUNT_BALANCE_SNS_TOPIC],
        ...alarmCommandSharedInput,
        ComparisonOperator: "LessThanThreshold",
        Threshold: InsufficientBalanceThreshold.Substrate,
    })
    cloudWatchAlarms.push(sovereignAccountBalanceAlarm)

    // Indexer service stale
    for (const chain of [
        "assethub",
        "bridgehub",
        "ethereum",
        "kusama_assethub",
        "hydration",
        "neuroweb",
        "mythos",
    ]) {
        let indexerAlarm = new PutMetricAlarmCommand({
            AlarmName: AlarmReason.IndexServiceStale.toString() + "-" + name + "-" + chain,
            MetricName: "IndexerLatency",
            Dimensions: [
                {
                    Name: "ChainName",
                    Value: chain,
                },
            ],
            AlarmDescription: AlarmReason.IndexServiceStale.toString(),
            AlarmActions: [BRIDGE_STALE_SNS_TOPIC],
            ...alarmCommandSharedInput,
            Threshold: IndexerLatencyThreshold,
        })
        cloudWatchAlarms.push(indexerAlarm)
    }

    // Heartbeat lost
    let heartbeartAlarm = new PutMetricAlarmCommand({
        AlarmName: AlarmReason.HeartbeatLost.toString() + "-" + name,
        MetricName: "Heartbeat",
        AlarmDescription: AlarmReason.HeartbeatLost.toString(),
        AlarmActions: [BRIDGE_STALE_SNS_TOPIC],
        ...alarmCommandSharedInput,
        ComparisonOperator: "LessThanThreshold",
        Threshold: 1,
        TreatMissingData: "breaching",
    })
    cloudWatchAlarms.push(heartbeartAlarm)

    // To Ethereum V2 stale
    cloudWatchAlarms.push(
        new PutMetricAlarmCommand({
            AlarmName: AlarmReason.ToEthereumV2Stale.toString() + "-" + name,
            MetricName: "ToEthereumV2UndeliveredTimeout",
            AlarmDescription: LatencyDashboard,
            AlarmActions: [BRIDGE_STALE_SNS_TOPIC],
            ...alarmCommandSharedInput,
            Threshold: 5400, // 1.5 hours at most
        }),
    )

    // To Polkadot V2 stale
    cloudWatchAlarms.push(
        new PutMetricAlarmCommand({
            AlarmName: AlarmReason.ToPolkadotV2Stale.toString() + "-" + name,
            MetricName: "ToPolkadotV2UndeliveredTimeout",
            AlarmDescription: LatencyDashboard,
            AlarmActions: [BRIDGE_STALE_SNS_TOPIC],
            ...alarmCommandSharedInput,
            Threshold: 1800, // 0.5 hour
        }),
    )

    // Fisherman FutureBlockVoting equivocation alarm
    cloudWatchAlarms.push(
        new PutMetricAlarmCommand({
            AlarmName: AlarmReason.FutureBlockVoting.toString() + "-" + name,
            MetricName: AlarmReason.FutureBlockVoting.toString(),
            AlarmDescription: AlarmReason.FutureBlockVoting.toString(),
            AlarmActions: [BRIDGE_STALE_SNS_TOPIC],
            ...alarmCommandSharedInput,
            Period: 120,
            EvaluationPeriods: 1,
            DatapointsToAlarm: 1,
            Threshold: 0,
        }),
    )
    // Fisherman ForkVoting equivocation alarm
    cloudWatchAlarms.push(
        new PutMetricAlarmCommand({
            AlarmName: AlarmReason.ForkVoting.toString() + "-" + name,
            MetricName: AlarmReason.ForkVoting.toString(),
            AlarmDescription: AlarmReason.ForkVoting.toString(),
            AlarmActions: [BRIDGE_STALE_SNS_TOPIC],
            ...alarmCommandSharedInput,
            Period: 120,
            EvaluationPeriods: 1,
            DatapointsToAlarm: 1,
            Threshold: 0,
        }),
    )

    // Send all alarms
    for (let alarm of cloudWatchAlarms) {
        await client.send(alarm)
    }
}

const sendFishermanAlarm = async (nameSpace: string, reason: AlarmReason, blockNumber: number) => {
    let client = new CloudWatchClient({})
    let metricData = [] // Fisherman metrics
    metricData.push({
        MetricName: reason.toString(),
        Value: blockNumber,
    })
    const command = new PutMetricDataCommand({
        MetricData: metricData,
        Namespace: CLOUD_WATCH_NAME_SPACE + "-" + nameSpace,
    })
    console.log("Sent fisherman alarm:", JSON.stringify(metricData, null, 2))
    await client.send(command)
}

export const sendForkVotingAlarm = async (nameSpace: string, blockNumber: number) => {
    await sendFishermanAlarm(nameSpace, AlarmReason.ForkVoting, blockNumber)
}

export const sendFutureBlockVotingAlarm = async (nameSpace: string, blockNumber: number) => {
    await sendFishermanAlarm(nameSpace, AlarmReason.FutureBlockVoting, blockNumber)
}

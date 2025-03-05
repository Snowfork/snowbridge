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
    AccountBalanceInsufficient = "AccountBalanceInsufficient",
    ToEthereumNoTransfer = "ToEthereumNoTransfer",
    ToPolkadotNoTransfer = "ToPolkadotNoTransfer",
    ToEthereumChannelAttacked = "ToEthereumChannelAttacked",
    ToPolkadotChannelAttacked = "ToPolkadotChannelAttacked",
    IndexServiceStale = "IndexServiceStale",
}

export const InsufficientBalanceThreshold = {
    // Minimum as 300 DOT
    Substrate: process.env["SubstrateBalanceThreshold"]
        ? parseInt(process.env["SubstrateBalanceThreshold"])
        : 3_000_000_000_000,
    // Minimum as 0.3 Ether
    Ethereum: process.env["EthereumBalanceThreshold"]
        ? parseInt(process.env["EthereumBalanceThreshold"])
        : 300_000_000_000_000_000,
}

export const BlockLatencyThreshold = {
    // Syncing beefy finality update every 4 hours(1200 ethereum blocks), leave some buffer here
    ToEthereum: process.env["BlockLatencyToEthereum"]
        ? parseInt(process.env["BlockLatencyToEthereum"])
        : 1800,
    // Syncing beacon finality update every 6.4 minutes(64 substrate blocks), leave some buffer here
    ToPolkadot: process.env["BlockLatencyToPolkadot"]
        ? parseInt(process.env["BlockLatencyToPolkadot"])
        : 120,
}

export const AlarmEvaluationConfiguration = {
    ToEthereumStale: {
        EvaluationPeriods: process.env["ToEthereumEvaluationPeriods"]
            ? parseInt(process.env["ToEthereumEvaluationPeriods"])
            : 12,
        DatapointsToAlarm: process.env["ToEthereumDatapointsToAlarm"]
            ? parseInt(process.env["ToEthereumDatapointsToAlarm"])
            : 10,
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

export const sendMetrics = async (metrics: status.AllMetrics) => {
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
            metrics.bridgeStatus.toEthereum.blockLatency > BlockLatencyThreshold.ToEthereum
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
            metrics.bridgeStatus.toPolkadot.blockLatency > BlockLatencyThreshold.ToPolkadot
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
            MetricName: "ToEthereumUndelivered",
            Dimensions: [
                {
                    Name: "ChannelName",
                    Value: channel.name,
                },
            ],
            Value: channel.toEthereum.outbound - channel.toEthereum.inbound,
        })
        metricData.push({
            MetricName: AlarmReason.ToEthereumChannelStale.toString(),
            Value: Number(
                channel.toEthereum.outbound > channel.toEthereum.inbound &&
                    channel.toEthereum.inbound == channel.toEthereum.previousInbound
            ),
        })
        metricData.push({
            MetricName: AlarmReason.ToEthereumChannelAttacked.toString(),
            Value: Number(channel.toEthereum.outbound < channel.toEthereum.inbound),
        })
        metricData.push({
            MetricName: AlarmReason.ToEthereumNoTransfer.toString(),
            Value: Number(channel.toEthereum.inbound == channel.toEthereum.previousInbound),
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
            MetricName: "ToPolkadotUndelivered",
            Dimensions: [
                {
                    Name: "ChannelName",
                    Value: channel.name,
                },
            ],
            Value: channel.toPolkadot.outbound - channel.toPolkadot.inbound,
        })
        metricData.push({
            MetricName: AlarmReason.ToPolkadotChannelStale.toString(),
            Value: Number(
                channel.toPolkadot.outbound > channel.toPolkadot.inbound &&
                    channel.toPolkadot.inbound == channel.toPolkadot.previousInbound
            ),
        })
        metricData.push({
            MetricName: AlarmReason.ToPolkadotChannelAttacked.toString(),
            Value: Number(channel.toPolkadot.outbound < channel.toPolkadot.inbound),
        })
        metricData.push({
            MetricName: AlarmReason.ToPolkadotNoTransfer.toString(),
            Value: Number(channel.toPolkadot.inbound == channel.toPolkadot.previousInbound),
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
    let indexerStale = false
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
        indexerStale = status.latency > IndexerLatencyThreshold
        if (indexerStale) {
            break
        }
    }
    metricData.push({
        MetricName: AlarmReason.IndexServiceStale.toString(),
        Value: Number(indexerStale),
    })
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
        Threshold: 0,
    }
    if (name == "polkadot_mainnet") {
        alarmCommandSharedInput.TreatMissingData = "breaching"
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
            EvaluationPeriods: AlarmEvaluationConfiguration.ToEthereumStale.EvaluationPeriods,
            Period: 1800,
            DatapointsToAlarm: AlarmEvaluationConfiguration.ToEthereumStale.DatapointsToAlarm,
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
            EvaluationPeriods: AlarmEvaluationConfiguration.ToPolkadotStale.EvaluationPeriods,
            Period: 1800,
            DatapointsToAlarm: AlarmEvaluationConfiguration.ToPolkadotStale.DatapointsToAlarm,
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
            EvaluationPeriods: AlarmEvaluationConfiguration.ToEthereumStale.EvaluationPeriods,
            Period: 1800,
            DatapointsToAlarm: AlarmEvaluationConfiguration.ToEthereumStale.DatapointsToAlarm,
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
            EvaluationPeriods: AlarmEvaluationConfiguration.ToPolkadotStale.EvaluationPeriods,
            Period: 1800,
            DatapointsToAlarm: AlarmEvaluationConfiguration.ToPolkadotStale.DatapointsToAlarm,
            ...alarmCommandSharedInput,
        })
    )
    cloudWatchAlarms.push(
        new PutMetricAlarmCommand({
            AlarmName: AlarmReason.ToEthereumChannelAttacked.toString() + "-" + name,
            MetricName: AlarmReason.ToEthereumChannelAttacked.toString(),
            AlarmDescription: LatencyDashboard,
            Statistic: "Average",
            ComparisonOperator: "GreaterThanThreshold",
            AlarmActions: [BRIDGE_ATTACKED_SNS_TOPIC],
            EvaluationPeriods: 1,
            Period: 1800,
            ...alarmCommandSharedInput,
        })
    )
    cloudWatchAlarms.push(
        new PutMetricAlarmCommand({
            AlarmName: AlarmReason.ToPolkadotChannelAttacked.toString() + "-" + name,
            MetricName: AlarmReason.ToPolkadotChannelAttacked.toString(),
            AlarmDescription: LatencyDashboard,
            Statistic: "Average",
            ComparisonOperator: "GreaterThanThreshold",
            AlarmActions: [BRIDGE_ATTACKED_SNS_TOPIC],
            EvaluationPeriods: 1,
            Period: 1800,
            ...alarmCommandSharedInput,
        })
    )
    // For westend alarm when there is no transfer(i.e. nonce not increased) for more than 1 day
    if (name == "westend_sepolia") {
        cloudWatchAlarms.push(
            new PutMetricAlarmCommand({
                AlarmName: AlarmReason.ToEthereumNoTransfer.toString() + "-" + name,
                MetricName: AlarmReason.ToEthereumNoTransfer.toString(),
                AlarmDescription: LatencyDashboard,
                Statistic: "Average",
                ComparisonOperator: "GreaterThanThreshold",
                AlarmActions: [BRIDGE_STALE_SNS_TOPIC],
                EvaluationPeriods: 3,
                Period: 21600,
                ...alarmCommandSharedInput,
            })
        )
        cloudWatchAlarms.push(
            new PutMetricAlarmCommand({
                AlarmName: AlarmReason.ToPolkadotNoTransfer.toString() + "-" + name,
                MetricName: AlarmReason.ToPolkadotNoTransfer.toString(),
                AlarmDescription: LatencyDashboard,
                Statistic: "Average",
                ComparisonOperator: "GreaterThanThreshold",
                AlarmActions: [BRIDGE_STALE_SNS_TOPIC],
                EvaluationPeriods: 3,
                Period: 21600,
                ...alarmCommandSharedInput,
            })
        )
    }

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
        EvaluationPeriods: 2,
        Period: 1800,
        ...alarmCommandSharedInput,
    })
    await client.send(accountBalanceAlarm)

    // Alarm for indexer service
    let indexerAlarm = new PutMetricAlarmCommand({
        AlarmName: AlarmReason.IndexServiceStale.toString() + "-" + name,
        MetricName: AlarmReason.IndexServiceStale.toString(),
        AlarmDescription: AlarmReason.IndexServiceStale.toString(),
        Statistic: "Average",
        ComparisonOperator: "GreaterThanThreshold",
        AlarmActions: [BRIDGE_STALE_SNS_TOPIC],
        EvaluationPeriods: 3,
        Period: 1800,
        ...alarmCommandSharedInput,
    })
    await client.send(indexerAlarm)
}

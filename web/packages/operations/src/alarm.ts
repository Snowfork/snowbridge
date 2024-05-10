import { status } from "@snowbridge/api"
import axios from "axios"

const MaxBlockLatency = 1200
const SLACK_WEBHOOK_URL = process.env["SLACK_WEBHOOK_URL"] || ""

export type AllMetrics = {
    bridgeStatus: status.BridgeStatusInfo
    primaryChannelInfo: status.ChannelStatusInfo
    assetHubChannelInfo: status.ChannelStatusInfo
    assetHubSovereignBalance: string
    assetHubAgentBalance: string
}

export enum AlarmReason {
    BeefyStale,
    BeaconStale,
    ToEthereumAssetHubChannelStale,
    ToPolkadotAssetHubChannelStale,
}

export const sendAlarm = async (metrics: AllMetrics) => {
    let alarm = false
    let reasons = []

    if (
        metrics.bridgeStatus.toEthereum.blockLatency > MaxBlockLatency ||
        metrics.bridgeStatus.toEthereum.latestPolkadotBlockOnEthereum ==
            metrics.bridgeStatus.toEthereum.previousPolkadotBlockOnEthereum
    ) {
        alarm = true
        reasons.push(AlarmReason.BeefyStale)
    }
    if (
        metrics.bridgeStatus.toPolkadot.blockLatency > MaxBlockLatency ||
        metrics.bridgeStatus.toPolkadot.latestEthereumBlockOnPolkadot ==
            metrics.bridgeStatus.toPolkadot.previousEthereumBlockOnPolkadot
    ) {
        alarm = true
        reasons.push(AlarmReason.BeaconStale)
    }
    if (
        metrics.assetHubChannelInfo.toEthereum.outbound !=
            metrics.assetHubChannelInfo.toEthereum.inbound &&
        metrics.assetHubChannelInfo.toEthereum.inbound ==
            metrics.assetHubChannelInfo.toEthereum.previousInbound
    ) {
        alarm = true
        reasons.push(AlarmReason.ToEthereumAssetHubChannelStale)
    }
    if (
        metrics.assetHubChannelInfo.toPolkadot.outbound !=
            metrics.assetHubChannelInfo.toPolkadot.inbound &&
        metrics.assetHubChannelInfo.toPolkadot.inbound ==
            metrics.assetHubChannelInfo.toPolkadot.previousInbound
    ) {
        alarm = true
        reasons.push(AlarmReason.ToPolkadotAssetHubChannelStale)
    }

    if (alarm) {
        const text = JSON.stringify({ reasons, metrics }, null, 2)
        console.log(text)
        await axios.post(SLACK_WEBHOOK_URL, { text })
    }
}

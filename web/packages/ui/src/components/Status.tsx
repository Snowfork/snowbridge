import { u8aToHex } from '@polkadot/util'
import { blake2AsU8a, encodeAddress } from "@polkadot/util-crypto"
import { status, utils } from "@snowbridge/api"
import { ChannelStatusInfo } from '@snowbridge/api/dist/status'
import { useEffect,  useRef,  useState } from "react"
import { AccountInfo, AppProps, BridgeStatus } from "./Common"

const PRIMARY_GOVERNANCE_CHANNEL_ID = '0x0000000000000000000000000000000000000000000000000000000000000001'
const SECONDARY_GOVERNANCE_CHANNEL_ID = '0x0000000000000000000000000000000000000000000000000000000000000002'
const REFRESH_INTERVAL = 300;
const REFRESH_CHECK = 10;
const ACCEPTABLE_BRIDGE_LATENCY = 28800 /* 8 hours */

interface StatusProps extends AppProps {
    diagnostic: boolean,
    bridgeStatus?: BridgeStatus,
    setBridgeStatus: React.Dispatch<React.SetStateAction<BridgeStatus | undefined>>
    updateDate: Date
    setUpdateDate: React.Dispatch<React.SetStateAction<Date>>
}

const formatTime = (time: number): string => {
    let hours = Math.floor(time / 3600)
    let minutes = Math.floor((time % 3600) / 60)
    let seconds = Math.floor(time % 60)
    let fmt = ""
    if (hours > 0) fmt += `${hours}h `;
    if (minutes > 0) fmt += `${minutes}m `;
    fmt += `${seconds}s`;
    return fmt
}

interface AccountProps {
    account: AccountInfo
    decimals: number
}
const Account = ({ account, decimals }: AccountProps): JSX.Element => {
    let balance = account.balance.toString()
    let unit = ""
    switch (account.type) {
        case 'ethereum':
            balance = (Number(account.balance) / Number(1_000_000_000_000_000_000n)).toFixed(decimals)
            unit = "ETH"
            break;
        case 'substrate':
            // TODO: Format decimals 10 for polkadot
            balance = (Number(account.balance) / Number(1_000_000_000_000n)).toFixed(decimals)
            unit = "DOT"
            break;
        default:
    }
    return (<div className='panel accountData'>
        <p>{account.name}</p>
        <p>{balance + ' ' + unit}</p>
        <p>{account.account}</p>
    </div>)
}


interface ChannelProps {
    channelName: string
    status: ChannelStatusInfo
}
const Channel = ({ channelName, status }: ChannelProps): JSX.Element => {
    return (<div className='panel channelData'>
        <h4>{channelName}</h4>
        <div></div>
        <div></div>
        <div></div>
        <p>Inbound</p>
        <p>Outbound</p>
        <p>To Ethereum Nonce</p>
        <p>{status.toEthereum.inbound}</p>
        <p>{status.toEthereum.outbound}</p>
        <p>To Polkadot Nonce</p>
        <p>{status.toPolkadot.inbound}</p>
        <p>{status.toPolkadot.outbound}</p>
        <p>To Polkadot Operating Mode</p>
        <p>{status.toPolkadot.operatingMode.outbound}</p>
    </div>)
}

export const Status = (props: StatusProps): JSX.Element => {
    const { context, config, bridgeStatus, setBridgeStatus, setUpdateDate, updateDate } = props
    const [refreshInProgress, setRefreshInProgress] = useState<boolean>(false)
    const [timeNow, setTimeNow] = useState<Date>(new Date())
    const refreshTime = useRef<Date>(updateDate)
    const [error, setError] = useState<string>('')

    const refreshStatus = async () => {
        setRefreshInProgress(true)
        if (context === undefined) {
            setError('Context not connected.')
            setRefreshInProgress(false)
            return
        }

        const bridgeStatusInfo = await status.bridgeStatusInfo(context)
        const assethub = await status.channelStatusInfo(context, utils.paraIdToChannelId(config.ASSET_HUB_PARAID))
        const primaryGov = await status.channelStatusInfo(context, PRIMARY_GOVERNANCE_CHANNEL_ID)
        const secondaryGov = await status.channelStatusInfo(context, SECONDARY_GOVERNANCE_CHANNEL_ID)

        const accounts: AccountInfo[] = []
        const assetHubSovereignAddress = utils.paraIdToSovereignAccount("sibl", config.ASSET_HUB_PARAID)
        const assetHubSovereignBalance = BigInt(((await context.polkadot.api.bridgeHub.query.system.account(assetHubSovereignAddress)).toPrimitive() as any).data.free)
        accounts.push({ name: "Asset Hub Sovereign", type: "substrate", account: encodeAddress(assetHubSovereignAddress), balance: assetHubSovereignBalance })

        const assetHubAgentAddress = await context.ethereum.contracts.gateway.agentOf(
            utils.paraIdToAgentId(context.polkadot.api.bridgeHub.registry, config.ASSET_HUB_PARAID)
        )
        const assetHubAgentBalance = (await context.ethereum.api.getBalance(assetHubAgentAddress))
        accounts.push({ name: "Asset Hub Agent", type: "ethereum", account: assetHubAgentAddress, balance: assetHubAgentBalance })

        const bridgeHubAgentId = u8aToHex(blake2AsU8a("0x00", 256))
        let bridgeHubAgentAddress = await context.ethereum.contracts.gateway.agentOf(bridgeHubAgentId)
        let bridgeHubAgentBalance = await context.ethereum.api.getBalance(bridgeHubAgentAddress)
        accounts.push({ name: "Bridge Hub Agent", type: "ethereum", account: bridgeHubAgentAddress, balance: bridgeHubAgentBalance })

        const relayers: AccountInfo[] = []
        for (const relayer of config.RELAYERS) {
            let balance = 0n
            switch (relayer.type) {
                case "ethereum":
                    balance = await context.ethereum.api.getBalance(relayer.account)
                    break
                case "substrate":
                    balance = BigInt(((await context.polkadot.api.bridgeHub.query.system.account(relayer.account)).toPrimitive() as any).data.free)
                    break
            }
            relayers.push({ name: relayer.name, account: relayer.account, balance: balance, type: relayer.type })
        }

        setBridgeStatus({
            statusInfo: bridgeStatusInfo,
            assetHubChannel: assethub,
            channelStatusInfos: [
                { name: "Asset Hub", status: assethub },
                { name: "Primary Governance", status: primaryGov },
                { name: "Secondary Governance", status: secondaryGov },
            ],
            relayers,
            accounts,
        })
        const nextUpdate = new Date()
        nextUpdate.setTime(new Date().getTime() + REFRESH_INTERVAL * 1000)
        setUpdateDate(nextUpdate)
        refreshTime.current = nextUpdate;
        setRefreshInProgress(false)
    }

    useEffect(() => {
        const intervalId = setInterval(() => {
            const now = new Date()
            setTimeNow(now)
            if (refreshTime.current <= now) {
                refreshStatus();
            }
        }, REFRESH_CHECK * 1000);

        return () => { clearInterval(intervalId); }
    }, [context]);

    if (context === undefined) {
        return (<div>Loading...</div>)
    }

    if (error !== '') {
        return (<div style={{ color: "red" }}>{error}</div>)
    }
    if (bridgeStatus === undefined) {
        return (<div>Loading...</div>)
    }
    let extra: JSX.Element = <div hidden></div>
    if (props.diagnostic) {
        extra = (
            <div>
                <h2>Detailed Diagnostic</h2>
                <h3>To Ethereum</h3>
                <div className='panel statusGeneral'>
                    <p>Outbound Messages:</p><p>{bridgeStatus?.statusInfo.toEthereum.operatingMode.outbound}</p>
                    <p>Latest Relay Chain Block:</p><p>{bridgeStatus?.statusInfo.toEthereum.latestPolkaotBlock}</p>
                    <p>Latest Relay Chain Block in Beefy Client:</p><p>{bridgeStatus?.statusInfo.toEthereum.latestPolkadotBlockOnEthereum}</p>
                    <p>BEEFY client latency (blocks):</p><p>{bridgeStatus?.statusInfo.toEthereum.blockLatency}</p>
                </div >
                <h3>To Polkadot</h3>
                <div className='panel statusGeneral'>
                    <p>Beacon Client:</p><p>{bridgeStatus?.statusInfo.toPolkadot.operatingMode.beacon}</p>
                    <p>Inbound Messages:</p><p>{bridgeStatus?.statusInfo.toPolkadot.operatingMode.inbound}</p>
                    <p>Outbound Messages:</p><p>{bridgeStatus?.statusInfo.toPolkadot.operatingMode.outbound}</p>
                    <p>Latest Ethereum Block:</p><p>{bridgeStatus?.statusInfo.toPolkadot.latestEthereumBlock}</p>
                    <p>Latest Ethereum Block in Beacon Client:</p><p>{bridgeStatus?.statusInfo.toPolkadot.latestEthereumBlockOnPolkadot}</p>
                    <p>Beacon client latency (blocks):</p><p>{bridgeStatus?.statusInfo.toPolkadot.blockLatency}</p>
                </div >
                <h3>Channels</h3>
                {bridgeStatus.channelStatusInfos.map((channel, i) => (<Channel key={i} channelName={channel.name} status={channel.status}></Channel>))}
                <h3>Relayers</h3>
                {bridgeStatus.relayers.map((relayer, i) => (<Account key={i} decimals={2} account={relayer}></Account>))}
                <h3>Accounts</h3>
                {bridgeStatus.accounts.map((account, i) => (<Account key={i} decimals={2} account={account}></Account>))}
                <p></p>
            </div>)
    }

    const toPolkadot = {
        lightClientLatencyIsAcceptable: bridgeStatus.statusInfo.toPolkadot.latencySeconds < ACCEPTABLE_BRIDGE_LATENCY,
        bridgeOperational: bridgeStatus.statusInfo.toPolkadot.operatingMode.outbound === 'Normal' && bridgeStatus.statusInfo.toPolkadot.operatingMode.beacon === 'Normal',
        channelOperational: bridgeStatus.assetHubChannel.toPolkadot.operatingMode.outbound === 'Normal',
    }
    let toPolkadotOperatingMode =
        !toPolkadot.bridgeOperational || !toPolkadot.channelOperational ? "Halted"
            : !toPolkadot.lightClientLatencyIsAcceptable ? "Delayed"
                : "Normal"
    let toPolkadotStyle = { color: toPolkadotOperatingMode === "Normal" ? "green" : "red" }

    const toEthereum = {
        bridgeOperational: bridgeStatus.statusInfo.toEthereum.operatingMode.outbound === 'Normal',
        lightClientLatencyIsAcceptable: bridgeStatus.statusInfo.toEthereum.latencySeconds < ACCEPTABLE_BRIDGE_LATENCY,
    }
    let toEthereumOperatingMode =
        !toEthereum.bridgeOperational ? "Halted"
            : !toEthereum.lightClientLatencyIsAcceptable ? "Delayed"
                : "Normal"
    let toEthereumStyle = { color: toEthereumOperatingMode === "Normal" ? "green" : "red" }

    return (<div>
        <h2>Bridge Status</h2>
        <h3>To Polkadot</h3>
        <div className='panel statusGeneral' style={toPolkadotStyle}>
            <p>Operating Mode</p><p>{toPolkadotOperatingMode}</p>
            <p>BEEFY client latency (time):</p><p>{formatTime(bridgeStatus?.statusInfo.toEthereum.latencySeconds)}</p>
        </div>
        <h3>To Ethereum</h3>
        <div className='panel statusGeneral' style={toEthereumStyle}>
            <p>Operating Mode</p><p>{toEthereumOperatingMode}</p>
            <p>Beacon client latency (time):</p><p>{formatTime(bridgeStatus?.statusInfo.toPolkadot.latencySeconds)}</p>
        </div>
        {extra}
        <p>Refreshing in {formatTime((updateDate.getTime() - timeNow.getTime())/1000)} seconds.</p>
        <button disabled={refreshInProgress} onClick={refreshStatus}>Refresh Now</button>
    </div>)
}
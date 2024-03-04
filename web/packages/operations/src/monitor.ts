
import { contextFactory, status, utils } from '@snowbridge/api'

const PRIMARY_GOVERNANCE_CHANNEL_ID = '0x0000000000000000000000000000000000000000000000000000000000000001'
const SECONDARY_GOVERNANCE_CHANNEL_ID = '0x0000000000000000000000000000000000000000000000000000000000000002'

let config = {
    ETHEREUM_WS_API: 'ws://127.0.0.1:8546',
    RELAY_CHAIN_WS_URL: 'ws://127.0.0.1:9944',
    ASSET_HUB_WS_URL: 'ws://127.0.0.1:12144',
    BRIDGE_HUB_WS_URL: 'ws://127.0.0.1:11144',
    GATEWAY_CONTRACT: '0xEDa338E4dC46038493b885327842fD3E301CaB39',
    BEEFY_CONTRACT: '0x992B9df075935E522EC7950F37eC8557e86f6fdb',
    ASSET_HUB_PARAID: 1000,
}
if (process.env.NODE_ENV === 'production') {
    config = {
        ETHEREUM_WS_API: `wss://sepolia.infura.io/ws/v3/${process.env.REACT_APP_INFURA_KEY}`,
        RELAY_CHAIN_WS_URL: 'wss://rococo-rpc.polkadot.io',
        ASSET_HUB_WS_URL: 'wss://rococo-asset-hub-rpc.polkadot.io',
        BRIDGE_HUB_WS_URL: 'wss://rococo-bridge-hub-rpc.polkadot.io',
        GATEWAY_CONTRACT: '0x5b4909ce6ca82d2ce23bd46738953c7959e710cd',
        BEEFY_CONTRACT: '0x27e5e17ac995d3d720c311e1e9560e28f5855fb1',
        ASSET_HUB_PARAID: 1000,
    }
}

const monitor = async () => {
    const context = await contextFactory({
        ethereum: { url: config.ETHEREUM_WS_API },
        polkadot: {
            url: {
                bridgeHub: config.BRIDGE_HUB_WS_URL,
                assetHub: config.ASSET_HUB_WS_URL,
                relaychain: config.RELAY_CHAIN_WS_URL,
            },
        },
        appContracts: {
            gateway: config.GATEWAY_CONTRACT,
            beefy: config.BEEFY_CONTRACT,
        },
    })

    const bridegStatus = await status.bridgeStatusInfo(context)
    console.log('Bridge Status:', bridegStatus)
    const assethub = await status.channelStatusInfo(context, utils.paraIdToChannelId(config.ASSET_HUB_PARAID))
    console.log('Asset Hub Channel:', assethub)
    const primaryGov = await status.channelStatusInfo(context, PRIMARY_GOVERNANCE_CHANNEL_ID)
    console.log('Primary Governance Channel:', primaryGov)
    const secondaryGov = await status.channelStatusInfo(context, SECONDARY_GOVERNANCE_CHANNEL_ID)
    console.log('Secondary Governance Channel:', secondaryGov)
}


monitor()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error(error)
        process.exit(1)
    })

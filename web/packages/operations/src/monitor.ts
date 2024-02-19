
import { contextFactory, toPolkadot, status } from '@snowbridge/api'
import { Wallet } from 'ethers'

const ETHEREUM_WS_API = 'ws://127.0.0.1:8546'
const RELAY_CHAIN_WS_URL = 'ws://127.0.0.1:9944'
const ASSET_HUB_WS_URL = 'ws://127.0.0.1:12144'
const BRIDGE_HUB_WS_URL = 'ws://127.0.0.1:11144'
const GATEWAY_CONTRACT = '0xEDa338E4dC46038493b885327842fD3E301CaB39'
const BEEFY_CONTRACT = '0x992B9df075935E522EC7950F37eC8557e86f6fdb'
const WETH_CONTRACT = '0x87d1f7fdfEe7f651FaBc8bFCB6E086C278b77A7d'
const ASSET_HUB_CHANNEL_ID = '0xc173fac324158e77fb5840738a1a541f633cbec8884c6a601c567d2b376a0539'
const PRIMARY_GOVERNANCE_CHANNEL_ID = '0x0000000000000000000000000000000000000000000000000000000000000001'
const SECONDARY_GOVERNANCE_CHANNEL_ID = '0x0000000000000000000000000000000000000000000000000000000000000002'

const monitor = async () => {
    const context = await contextFactory({
        ethereum: { url: ETHEREUM_WS_API },
        polkadot: {
            url: {
                bridgeHub: BRIDGE_HUB_WS_URL,
                assetHub: ASSET_HUB_WS_URL,
                relaychain: RELAY_CHAIN_WS_URL,
            },
        },
        appContracts: {
            gateway: GATEWAY_CONTRACT,
            beefy: BEEFY_CONTRACT,
        },
    })

    const bridegStatus = await status.bridgeStatusInfo(context)
    console.log('Bridge Status:', bridegStatus)
    const assethub = await status.channelStatusInfo(context, ASSET_HUB_CHANNEL_ID)
    console.log('Asset Hub Channel:', assethub)
    const primaryGov = await status.channelStatusInfo(context, PRIMARY_GOVERNANCE_CHANNEL_ID)
    console.log('Primary Governance Channel:', primaryGov)
    const secondaryGov = await status.channelStatusInfo(context, SECONDARY_GOVERNANCE_CHANNEL_ID)
    console.log('Secondary Governance Channel:', secondaryGov)

    const signer = new Wallet('0x5e002a1af63fd31f1c25258f3082dc889762664cb8f218d86da85dff8b07b342', context.ethereum.api)
    const plan = await toPolkadot.validateSend(context, signer, '5CiPPseXPECbkjWCa6MnjNokrgYjMqmKndv2rSnekmSK2DjL', WETH_CONTRACT, 2000, BigInt(1000), BigInt(4_000_000_000))
    console.log('Plan:', plan)
    const result = await toPolkadot.send(context, signer, plan)
    console.log('Execute:', result)
    for await (const update of toPolkadot.trackSendProgress(context, result)) {
        console.log(update)
    }
}


monitor()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error(error)
        process.exit(1)
    })

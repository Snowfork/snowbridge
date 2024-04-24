
import { contextFactory, destroyContext, toEthereum, toPolkadot } from '@snowbridge/api'
import { Keyring } from '@polkadot/keyring'
import { Wallet } from 'ethers'

const BEACON_HTTP_API = 'http://127.0.0.1:9596'
const ETHEREUM_WS_API = 'ws://127.0.0.1:8546'
const RELAY_CHAIN_WS_URL = 'ws://127.0.0.1:9944'
const ASSET_HUB_WS_URL = 'ws://127.0.0.1:12144'
const PENPAL_WS_URL = 'ws://127.0.0.1:13144'
const BRIDGE_HUB_WS_URL = 'ws://127.0.0.1:11144'
const GATEWAY_CONTRACT = '0xEDa338E4dC46038493b885327842fD3E301CaB39'
const BEEFY_CONTRACT = '0x992B9df075935E522EC7950F37eC8557e86f6fdb'
const WETH_CONTRACT = '0x87d1f7fdfEe7f651FaBc8bFCB6E086C278b77A7d'

const monitor = async () => {
    const context = await contextFactory({
        ethereum: { execution_url: ETHEREUM_WS_API, beacon_url: BEACON_HTTP_API },
        polkadot: {
            url: {
                bridgeHub: BRIDGE_HUB_WS_URL,
                assetHub: ASSET_HUB_WS_URL,
                relaychain: RELAY_CHAIN_WS_URL,
                parachains: [PENPAL_WS_URL]
            },
        },
        appContracts: {
            gateway: GATEWAY_CONTRACT,
            beefy: BEEFY_CONTRACT,
        },
    })
    const polkadot_keyring = new Keyring({ type: 'sr25519' })

    const ETHEREUM_ACCOUNT = new Wallet('0x5e002a1af63fd31f1c25258f3082dc889762664cb8f218d86da85dff8b07b342', context.ethereum.api)
    const ETHEREUM_ACCOUNT_PUBLIC = await ETHEREUM_ACCOUNT.getAddress()
    const POLKADOT_ACCOUNT = polkadot_keyring.addFromUri('//Ferdie')
    const POLKADOT_ACCOUNT_PUBLIC = POLKADOT_ACCOUNT.address

    const amount = 10n

    const POLL_INTERVAL_MS = 10_000

    console.log('# Ethereum to Asset Hub')
    {
        const plan = await toPolkadot.validateSend(context, ETHEREUM_ACCOUNT, POLKADOT_ACCOUNT_PUBLIC, WETH_CONTRACT, 1000, amount, BigInt(0))
        console.log('Plan:', plan)
        let result = await toPolkadot.send(context, ETHEREUM_ACCOUNT, plan)
        console.log('Execute:', result)
        while (true) {
            const { status } = (await toPolkadot.trackSendProgressPolling(context, result))
            if (status !== "pending") { break }
            await new Promise(r => setTimeout(r, POLL_INTERVAL_MS));
        }
        console.log('Complete:', result)
    }

    console.log('# Asset Hub to Ethereum')
    {
        const plan = await toEthereum.validateSend(context, POLKADOT_ACCOUNT, 1000, ETHEREUM_ACCOUNT_PUBLIC, WETH_CONTRACT, amount)
        console.log('Plan:', plan)
        const result = await toEthereum.send(context, POLKADOT_ACCOUNT, plan)
        console.log('Execute:', result)
        while (true) {
            const { status } = (await toEthereum.trackSendProgressPolling(context, result))
            if (status !== "pending") { break }
            await new Promise(r => setTimeout(r, POLL_INTERVAL_MS));
        }
        console.log('Complete:', result)
    }

    console.log('# Ethereum to Penpal')
    {
        const plan = await toPolkadot.validateSend(context, ETHEREUM_ACCOUNT, POLKADOT_ACCOUNT_PUBLIC, WETH_CONTRACT, 2000, amount, BigInt(4_000_000_000))
        console.log('Plan:', plan)
        let result = await toPolkadot.send(context, ETHEREUM_ACCOUNT, plan)
        console.log('Execute:', result)
        while (true) {
            const { status } = (await toPolkadot.trackSendProgressPolling(context, result))
            if (status !== "pending") { break }
            await new Promise(r => setTimeout(r, POLL_INTERVAL_MS));
        }
        console.log('Complete:', result)
    }

    console.log('# Penpal to Ethereum')
    {
        const plan = await toEthereum.validateSend(context, POLKADOT_ACCOUNT, 2000, ETHEREUM_ACCOUNT_PUBLIC, WETH_CONTRACT, amount)
        console.log('Plan:', plan)
        const result = await toEthereum.send(context, POLKADOT_ACCOUNT, plan)
        console.log('Execute:', result)
        while (true) {
            const { status } = (await toEthereum.trackSendProgressPolling(context, result))
            if (status !== "pending") { break }
            await new Promise(r => setTimeout(r, POLL_INTERVAL_MS));
        }
        console.log('Complete:', result)
    }

    await destroyContext(context)
}

monitor()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error('Error:', error)
        process.exit(1)
    })

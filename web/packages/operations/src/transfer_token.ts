
import { contextFactory, toEthereum, toPolkadot } from '@snowbridge/api'
import { Keyring } from '@polkadot/keyring'
import { Wallet } from 'ethers'

const ETHEREUM_WS_API = 'ws://127.0.0.1:8546'
const RELAY_CHAIN_WS_URL = 'ws://127.0.0.1:9944'
const ASSET_HUB_WS_URL = 'ws://127.0.0.1:12144'
const BRIDGE_HUB_WS_URL = 'ws://127.0.0.1:11144'
const GATEWAY_CONTRACT = '0xEDa338E4dC46038493b885327842fD3E301CaB39'
const BEEFY_CONTRACT = '0x992B9df075935E522EC7950F37eC8557e86f6fdb'
const WETH_CONTRACT = '0x87d1f7fdfEe7f651FaBc8bFCB6E086C278b77A7d'

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
    const polkadot_keyring = new Keyring({ type: 'sr25519' });

    const ETHEREUM_ACCOUNT = new Wallet('0x5e002a1af63fd31f1c25258f3082dc889762664cb8f218d86da85dff8b07b342', context.ethereum.api)
    const ETHEREUM_ACCOUNT_PUBLIC = await ETHEREUM_ACCOUNT.getAddress()
    const POLKADOT_ACCOUNT = polkadot_keyring.addFromUri('//Ferdie');
    const POLKADOT_ACCOUNT_PUBLIC = POLKADOT_ACCOUNT.address
    console.log(POLKADOT_ACCOUNT_PUBLIC)

    const amount = 1000n

    // To Polkadot (Asset Hub)
    {
        const plan = await toPolkadot.validateSend(context, ETHEREUM_ACCOUNT, POLKADOT_ACCOUNT_PUBLIC, WETH_CONTRACT, 1000, amount, BigInt(0))
        console.log('Plan:', plan)
        const result = await toPolkadot.send(context, ETHEREUM_ACCOUNT, plan)
        console.log('Execute:', result)
        for await (const update of toPolkadot.trackSendProgress(context, result)) {
            console.log(update)
        }
    }
    // To Polkadot (Penpal)
    {
        const plan = await toPolkadot.validateSend(context, ETHEREUM_ACCOUNT, POLKADOT_ACCOUNT_PUBLIC, WETH_CONTRACT, 2000, amount, BigInt(4_000_000_000))
        console.log('Plan:', plan)
        const result = await toPolkadot.send(context, ETHEREUM_ACCOUNT, plan)
        console.log('Execute:', result)
        for await (const update of toPolkadot.trackSendProgress(context, result)) {
            console.log(update)
        }
    }
    // To Ethereum
    {
        const plan = await toEthereum.validateSend(context, POLKADOT_ACCOUNT, ETHEREUM_ACCOUNT_PUBLIC, WETH_CONTRACT, 1n);
        console.log('Plan:', plan)
        const result = await toEthereum.send(context, POLKADOT_ACCOUNT, plan);
        console.log('Execute:', result)
        for await (const update of toEthereum.trackSendProgress(context, result)) {
            console.log(update)
        }
    }
}

monitor()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error(error)
        process.exit(1)
    })

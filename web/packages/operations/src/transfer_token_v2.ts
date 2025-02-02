import { Keyring } from "@polkadot/keyring"
import {
    Context,
    environment,
    toPolkadotV2,
    assetsV2
} from "@snowbridge/api"
import { WETH9__factory } from "@snowbridge/contract-types"
import { formatEther, Wallet } from "ethers"
import { cryptoWaitReady } from '@polkadot/util-crypto';

const monitor = async () => {
    let env = "local_e2e"
    if (process.env.NODE_ENV !== undefined) {
        env = process.env.NODE_ENV
    }
    const snwobridgeEnv = environment.SNOWBRIDGE_ENV[env]
    if (snwobridgeEnv === undefined) {
        throw Error(`Unknown environment '${env}'`)
    }
    console.log(`Using environment '${env}'`)

    const { ethChainId, config } = snwobridgeEnv
    await cryptoWaitReady()

    const context = new Context({
        ethereum: {
            execution_url: config.ETHEREUM_API(process.env.REACT_APP_INFURA_KEY || ""),
            beacon_url: config.BEACON_HTTP_API,
        },
        polkadot: {
            assetHubParaId: config.ASSET_HUB_PARAID,
            bridgeHubParaId: config.BRIDGE_HUB_PARAID,
            relaychain: config.RELAY_CHAIN_URL,
            parachains: config.PARACHAINS,
        },
        appContracts: {
            gateway: config.GATEWAY_CONTRACT,
            beefy: config.BEEFY_CONTRACT,
        },
    })


    // Contains the list of all token and parachain metadata in order to send tokens.
    // It may take some build but does not change often so it is safe to cache for 12 hours.
    //const registry = await assetsV2.buildRegistry(assetsV2.fromEnvironment(snwobridgeEnv))
    const registry = await assetsV2.buildRegistry(await assetsV2.fromContext(context))

    console.log("Asset Registry:", JSON.stringify(registry, (_, value) => typeof value === "bigint" ? String(value) : value, 2))

    const polkadot_keyring = new Keyring({ type: "sr25519" })

    const ETHEREUM_ACCOUNT = new Wallet(
        "0x5e002a1af63fd31f1c25258f3082dc889762664cb8f218d86da85dff8b07b342",
        context.ethereum()
    )
    const ETHEREUM_ACCOUNT_PUBLIC = await ETHEREUM_ACCOUNT.getAddress()
    const POLKADOT_ACCOUNT = polkadot_keyring.addFromUri("//Ferdie")
    const POLKADOT_ACCOUNT_PUBLIC = POLKADOT_ACCOUNT.address

    const amount = 10n

    const POLL_INTERVAL_MS = 10_000
    const WETH_CONTRACT = snwobridgeEnv.locations[0].erc20tokensReceivable.find(
        (t) => t.id === "WETH"
    )!.address

    //console.log("# Deposit and Approve WETH")
    //{
    //    const weth9 = WETH9__factory.connect(WETH_CONTRACT, ETHEREUM_ACCOUNT)
    //    const depositResult = await weth9.deposit({ value: amount })
    //    const depositReceipt = await depositResult.wait()

    //    const approveResult = await weth9.approve(config.GATEWAY_CONTRACT, amount * 2n)
    //    const approveReceipt = await approveResult.wait()

    //    console.log('deposit tx', depositReceipt?.hash, 'approve tx', approveReceipt?.hash)
    //}

    console.log("Ethereum to Asset Hub")
    {
        const destinationChainId = 1000
        const deliveryFee = await toPolkadotV2.getDeliveryFee(context.gateway(), registry, WETH_CONTRACT, destinationChainId)
        const transfer = await toPolkadotV2.createTransfer(
            registry,
            ETHEREUM_ACCOUNT_PUBLIC,
            POLKADOT_ACCOUNT_PUBLIC,
            WETH_CONTRACT,
            destinationChainId,
            amount,
            deliveryFee,
        );
        const { tx, computed: { totalValue }} = transfer
        const estimatedGas = await context.ethereum().estimateGas(tx)
        const feeData = await context.ethereum().getFeeData()
        const executionFee = (feeData.gasPrice ?? 0n) * estimatedGas
        console.log('tx:', tx)
        console.log('feeData:', feeData.toJSON())
        console.log('gas:', estimatedGas)
        console.log('delivery cost:', formatEther(deliveryFee))
        console.log('execution cost:', formatEther(executionFee))
        console.log('total cost:', formatEther(deliveryFee + executionFee))
        console.log('ether sent:', formatEther(totalValue - deliveryFee))
        console.log('dry run:', await context.ethereum().call(tx))

        const validation = await toPolkadotV2.validateTransfer({
            ethereum: context.ethereum(),
            gateway: context.gateway(),
            bridgeHub: await context.bridgeHub(),
            assetHub: await context.assetHub(),
        }, transfer)
        console.log('validation result', validation)
    }

    context.destroyContext()
}

monitor()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error("Error:", error)
        process.exit(1)
    })

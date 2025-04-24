import { Keyring } from "@polkadot/keyring"
import { Context, environment, toPolkadotV2 } from "@snowbridge/api"
import { formatEther, Wallet } from "ethers"
import { cryptoWaitReady } from "@polkadot/util-crypto"
import { fetchRegistry } from "./registry"

export const transferToPolkadot = async (
    destinationChainId: number,
    symbol: string,
    amount: bigint
) => {
    let env = "local_e2e"
    if (process.env.NODE_ENV !== undefined) {
        env = process.env.NODE_ENV
    }
    const snwobridgeEnv = environment.SNOWBRIDGE_ENV[env]
    if (snwobridgeEnv === undefined) {
        throw Error(`Unknown environment '${env}'`)
    }
    console.log(`Using environment '${env}'`)

    const { name, config, ethChainId } = snwobridgeEnv
    await cryptoWaitReady()

    const ethApikey = process.env.REACT_APP_INFURA_KEY || ""
    const ethChains: { [ethChainId: string]: string } = {}
    Object.keys(config.ETHEREUM_CHAINS).forEach(
        (ethChainId) =>
            (ethChains[ethChainId.toString()] = config.ETHEREUM_CHAINS[ethChainId](ethApikey))
    )
    const context = new Context({
        environment: name,
        ethereum: {
            ethChainId,
            ethChains,
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

    const polkadot_keyring = new Keyring({ type: "sr25519" })

    const ETHEREUM_ACCOUNT = new Wallet(
        process.env.ETHEREUM_KEY ?? "your key goes here",
        context.ethereum()
    )
    const ETHEREUM_ACCOUNT_PUBLIC = await ETHEREUM_ACCOUNT.getAddress()
    const POLKADOT_ACCOUNT = polkadot_keyring.addFromUri(
        process.env.SUBSTRATE_KEY ?? "your key goes here"
    )
    const POLKADOT_ACCOUNT_PUBLIC = POLKADOT_ACCOUNT.address

    console.log("eth", ETHEREUM_ACCOUNT_PUBLIC, "sub", POLKADOT_ACCOUNT_PUBLIC)

    const registry = await fetchRegistry(env, context)

    const assets = registry.ethereumChains[registry.ethChainId].assets
    const TOKEN_CONTRACT = Object.keys(assets)
        .map((t) => assets[t])
        .find((asset) => asset.symbol.toLowerCase().startsWith(symbol.toLowerCase()))?.token

    console.log("# Ethereum to Asset Hub")
    {
        // Step 1. Get the delivery fee for the transaction
        const fee = await toPolkadotV2.getDeliveryFee(
            {
                gateway: context.gateway(),
                assetHub: await context.assetHub(),
                destination: await context.parachain(destinationChainId),
            },
            registry,
            // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
            TOKEN_CONTRACT!,
            destinationChainId
        )

        // Step 2. Create a transfer tx
        const transfer = await toPolkadotV2.createTransfer(
            registry,
            ETHEREUM_ACCOUNT_PUBLIC,
            POLKADOT_ACCOUNT_PUBLIC,
            // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
            TOKEN_CONTRACT!,
            destinationChainId,
            amount,
            fee
        )

        // Step 3. Validate the transaction.
        const validation = await toPolkadotV2.validateTransfer(
            {
                ethereum: context.ethereum(),
                gateway: context.gateway(),
                bridgeHub: await context.bridgeHub(),
                assetHub: await context.assetHub(),
                destParachain:
                    destinationChainId !== 1000
                        ? await context.parachain(destinationChainId)
                        : undefined,
            },
            transfer
        )
        console.log("validation result", validation)

        // Step 4. Check validation logs for errors
        if (validation.logs.find((l) => l.kind == toPolkadotV2.ValidationKind.Error)) {
            throw Error(`validation has one of more errors.`)
        }

        // Step 5. Estimate the cost of the execution cost of the transaction
        const {
            tx,
            computed: { totalValue },
        } = transfer
        const estimatedGas = await context.ethereum().estimateGas(tx)
        const feeData = await context.ethereum().getFeeData()
        const executionFee = (feeData.gasPrice ?? 0n) * estimatedGas

        console.log("tx:", tx)
        console.log("feeData:", feeData.toJSON())
        console.log("gas:", estimatedGas)
        console.log("delivery cost:", formatEther(fee.totalFeeInWei))
        console.log("execution cost:", formatEther(executionFee))
        console.log("total cost:", formatEther(fee.totalFeeInWei + executionFee))
        console.log("ether sent:", formatEther(totalValue - fee.totalFeeInWei))
        console.log("dry run:", await context.ethereum().call(tx))

        if (process.env["DRY_RUN"] != "true") {
            // Step 6. Submit the transaction
            const response = await ETHEREUM_ACCOUNT.sendTransaction(tx)
            const receipt = await response.wait(1)
            if (!receipt) {
                throw Error(`Transaction ${response.hash} not included.`)
            }

            // Step 7. Get the message reciept for tracking purposes
            const message = await toPolkadotV2.getMessageReceipt(receipt)
            if (!message) {
                throw Error(`Transaction ${receipt.hash} did not emit a message.`)
            }
            console.log("Success message", message.messageId)
        }
    }
    await context.destroyContext()
}

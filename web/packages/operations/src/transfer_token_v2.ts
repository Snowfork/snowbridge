import { Keyring } from "@polkadot/keyring"
import {
    Context,
    environment,
    toPolkadotV2,
    assetsV2,
    toEthereumV2
} from "@snowbridge/api"
import { WETH9__factory } from "@snowbridge/contract-types"
import { formatEther, formatUnits, Wallet } from "ethers"
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

    const { config } = snwobridgeEnv
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

    let overrides = {}
    if(env === "polkadot_mainnet") {
        // Add override for mythos token and add precompile for moonbeam
        overrides = {
            precompiles: { "2004": "0x000000000000000000000000000000000000081A" },
            destinationFeeOverrides: {
                "3369": 500_000_000n
            },
            assetOverrides: {
                "3369": [
                    {
                        token: "0xba41ddf06b7ffd89d1267b5a93bfef2424eb2003".toLowerCase(),
                        name: "Mythos",
                        minimumBalance: 10_000_000_000_000_000n,
                        symbol: "MYTH",
                        decimals: 18,
                        isSufficient: true,
                    }
                ]
            }
        }
    }

    // Step 0. Build the Asset Registry. The registry contains the list of all token and parachain metadata in order to send tokens.
    // It may take some build but does not change often so it is safe to cache for 12 hours and shipped with your dapp as static data.
    //
    // The registry can be build from a snowbridge environment or snowbridge coutntext.
    //      const registry = await assetsV2.buildRegistry(assetsV2.fromEnvironment(snwobridgeEnv))
    // If your dapp does not use the snowbridge environment or context you can always build it manually by
    // specifying RegistryOptions for only the parachains you care about.
    const registry = await assetsV2.buildRegistry({
        ...await assetsV2.fromContext(context),
        ...overrides
    })

    console.log("Asset Registry:", JSON.stringify(registry, (_, value) => typeof value === "bigint" ? String(value) : value, 2))

    const polkadot_keyring = new Keyring({ type: "sr25519" })

    const ETHEREUM_ACCOUNT = new Wallet(
        process.env.ETH_PRIVATE_KEY ?? "your key goes here",
        context.ethereum()
    )
    const ETHEREUM_ACCOUNT_PUBLIC = await ETHEREUM_ACCOUNT.getAddress()
    const POLKADOT_ACCOUNT = polkadot_keyring.addFromUri("//Ferdie")
    const POLKADOT_ACCOUNT_PUBLIC = POLKADOT_ACCOUNT.address

    const amount = 10n

    const WETH_CONTRACT = snwobridgeEnv.locations[0].erc20tokensReceivable.find(
        (t) => t.id === "WETH"
    )!.address

    console.log("# Deposit and Approve WETH")
    {
        const weth9 = WETH9__factory.connect(WETH_CONTRACT, ETHEREUM_ACCOUNT)
        const depositResult = await weth9.deposit({ value: amount * 2n })
        const depositReceipt = await depositResult.wait()

        const approveResult = await weth9.approve(config.GATEWAY_CONTRACT, amount * 2n)
        const approveReceipt = await approveResult.wait()

        console.log('deposit tx', depositReceipt?.hash, 'approve tx', approveReceipt?.hash)
    }

    console.log("Ethereum to Asset Hub")
    {
        const destinationChainId: number = 1000
        // Step 1. Get the delivery fee for the transaction
        const fee = await toPolkadotV2.getDeliveryFee(context.gateway(), registry, WETH_CONTRACT, destinationChainId)

        // Step 2. Create a transfer tx
        const transfer = await toPolkadotV2.createTransfer(
            registry,
            ETHEREUM_ACCOUNT_PUBLIC,
            POLKADOT_ACCOUNT_PUBLIC,
            WETH_CONTRACT,
            destinationChainId,
            amount,
            fee,
        );

        // Step 3. Validate the transaction.
        const validation = await toPolkadotV2.validateTransfer({
            ethereum: context.ethereum(),
            gateway: context.gateway(),
            bridgeHub: await context.bridgeHub(),
            assetHub: await context.assetHub(),
            destParachain: (destinationChainId !== 1000) ? await context.parachain(destinationChainId) : undefined
        }, transfer)
        console.log('validation result', validation)

        // Step 4. Check validation logs for errors
        if (validation.logs.find(l => l.kind == toPolkadotV2.ValidationKind.Error)) {
            throw Error(`validation has one of more errors.`)
        }

        // Step 5. Estimate the cost of the execution cost of the transaction
        const { tx, computed: { totalValue } } = transfer
        const estimatedGas = await context.ethereum().estimateGas(tx)
        const feeData = await context.ethereum().getFeeData()
        const executionFee = (feeData.gasPrice ?? 0n) * estimatedGas

        console.log('tx:', tx)
        console.log('feeData:', feeData.toJSON())
        console.log('gas:', estimatedGas)
        console.log('delivery cost:', formatEther(fee.deliveryFeeInWei))
        console.log('execution cost:', formatEther(executionFee))
        console.log('total cost:', formatEther(fee.deliveryFeeInWei + executionFee))
        console.log('ether sent:', formatEther(totalValue - fee.deliveryFeeInWei))
        console.log('dry run:', await context.ethereum().call(tx))

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
        console.log('Success message', message.messageId);
    }

    console.log("Asset Hub to Ethereum")
    {
        const sourceParaId = 1000
        // Step 1. Get the delivery fee for the transaction
        const fee = await toEthereumV2.getDeliveryFee(await context.assetHub(), registry)

        // Step 2. Create a transfer tx
        const transfer = await toEthereumV2.createTransfer(
            await context.parachain(sourceParaId),
            registry,
            POLKADOT_ACCOUNT_PUBLIC,
            ETHEREUM_ACCOUNT_PUBLIC,
            WETH_CONTRACT,
            amount,
            fee,
        );

        // Step 3. Estimate the cost of the execution cost of the transaction
        console.log('call: ', transfer.tx.inner.toHex())
        console.log('utx: ', transfer.tx.toHex())
        const feePayment = (await transfer.tx.paymentInfo(POLKADOT_ACCOUNT, { withSignedTransaction: true })).toPrimitive() as any
        console.log(`execution fee (${transfer.computed.sourceParachain.info.tokenSymbols}):`,
            formatUnits(feePayment.partialFee, transfer.computed.sourceParachain.info.tokenDecimals))
        console.log(`delivery fee (${registry.parachains[registry.assetHubParaId].info.tokenSymbols}): `,
            formatUnits(fee.totalFeeInDot, transfer.computed.sourceParachain.info.tokenDecimals))
        console.log('dryRun: ', (
            await transfer.tx.dryRun(
                POLKADOT_ACCOUNT,
                { withSignedTransaction: true }
            )
        ).toHuman()
        )

        // Step 4. Validate the transaction.
        const validation = await toEthereumV2.validateTransfer({
            sourceParachain: await context.parachain(sourceParaId),
            assetHub: await context.assetHub(),
            gateway: context.gateway(),
            bridgeHub: await context.bridgeHub(),
        }, transfer)
        console.log('validation result', validation)

        // Step 5. Check validation logs for errors
        if (validation.logs.find(l => l.kind == toPolkadotV2.ValidationKind.Error)) {
            throw Error(`validation has one of more errors.`)
        }

        // Step 6. Submit transaction and get receipt for tracking
        const response = await toEthereumV2.signAndSend(
            await context.parachain(sourceParaId),
            transfer,
            POLKADOT_ACCOUNT,
            { withSignedTransaction: true }
        )
        if (!response) {
           throw Error(`Transaction ${response} not included.`)
        }
        console.log('Success message', response.messageId)
    }

    console.log("Ethereum to Penpal")
    {
        const destinationChainId: number = 2000
        // Step 1. Get the delivery fee for the transaction
        const fee = await toPolkadotV2.getDeliveryFee(context.gateway(), registry, WETH_CONTRACT, destinationChainId)

        // Step 2. Create a transfer tx
        const transfer = await toPolkadotV2.createTransfer(
            registry,
            ETHEREUM_ACCOUNT_PUBLIC,
            POLKADOT_ACCOUNT_PUBLIC,
            WETH_CONTRACT,
            destinationChainId,
            amount,
            fee,
        );

        // Step 3. Validate the transaction.
        const validation = await toPolkadotV2.validateTransfer({
            ethereum: context.ethereum(),
            gateway: context.gateway(),
            bridgeHub: await context.bridgeHub(),
            assetHub: await context.assetHub(),
            destParachain: (destinationChainId !== 1000) ? await context.parachain(destinationChainId) : undefined
        }, transfer)
        console.log('validation result', validation)

        // Step 4. Check validation logs for errors
        if (validation.logs.find(l => l.kind == toPolkadotV2.ValidationKind.Error)) {
            throw Error(`validation has one of more errors.`)
        }

        // Step 5. Estimate the cost of the execution cost of the transaction
        const { tx, computed: { totalValue } } = transfer
        const estimatedGas = await context.ethereum().estimateGas(tx)
        const feeData = await context.ethereum().getFeeData()
        const executionFee = (feeData.gasPrice ?? 0n) * estimatedGas

        console.log('tx:', tx)
        console.log('feeData:', feeData.toJSON())
        console.log('gas:', estimatedGas)
        console.log('delivery cost:', formatEther(fee.deliveryFeeInWei))
        console.log('execution cost:', formatEther(executionFee))
        console.log('total cost:', formatEther(fee.deliveryFeeInWei + executionFee))
        console.log('ether sent:', formatEther(totalValue - fee.deliveryFeeInWei))
        console.log('dry run:', await context.ethereum().call(tx))

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
        console.log('Success message', message.messageId)
    }

    console.log("Penpal to Ethereum")
    {
        const sourceParaId = 2000
        // Step 1. Get the delivery fee for the transaction
        const fee = await toEthereumV2.getDeliveryFee(await context.assetHub(), registry)

        // Step 2. Create a transfer tx
        const transfer = await toEthereumV2.createTransfer(
            await context.parachain(sourceParaId),
            registry,
            POLKADOT_ACCOUNT_PUBLIC,
            ETHEREUM_ACCOUNT_PUBLIC,
            WETH_CONTRACT,
            amount,
            fee,
        );

        // Step 3. Estimate the cost of the execution cost of the transaction
        console.log('call: ', transfer.tx.inner.toHex())
        console.log('utx: ', transfer.tx.toHex())
        const feePayment = (await transfer.tx.paymentInfo(POLKADOT_ACCOUNT, { withSignedTransaction: true })).toPrimitive() as any
        console.log(`execution fee (${transfer.computed.sourceParachain.info.tokenSymbols}):`,
            formatUnits(feePayment.partialFee, transfer.computed.sourceParachain.info.tokenDecimals))
        console.log(`delivery fee (${registry.parachains[registry.assetHubParaId].info.tokenSymbols}): `,
            formatUnits(fee.totalFeeInDot, transfer.computed.sourceParachain.info.tokenDecimals))
        console.log('dryRun: ', (
            await transfer.tx.dryRun(
                POLKADOT_ACCOUNT,
                { withSignedTransaction: true }
            )
        ).toHuman()
        )

        // Step 4. Validate the transaction.
        const validation = await toEthereumV2.validateTransfer({
            sourceParachain: await context.parachain(sourceParaId),
            assetHub: await context.assetHub(),
            gateway: context.gateway(),
            bridgeHub: await context.bridgeHub(),
        }, transfer)
        console.log('validation result', validation)

        // Step 5. Check validation logs for errors
        if (validation.logs.find(l => l.kind == toPolkadotV2.ValidationKind.Error)) {
            throw Error(`validation has one of more errors.`)
        }

        // Step 6. Submit transaction and get receipt for tracking
        const response = await toEthereumV2.signAndSend(
            await context.parachain(sourceParaId),
            transfer,
            POLKADOT_ACCOUNT,
            { withSignedTransaction: true }
        )
        if (!response) {
           throw Error(`Transaction ${response} not included.`)
        }
        console.log('Success message', response.messageId)
    }

    context.destroyContext()
}

monitor()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error("Error:", error)
        process.exit(1)
    })

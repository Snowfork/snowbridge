import { Keyring } from "@polkadot/keyring"
import { Context, toPolkadotSnowbridgeV2, toPolkadotV2 } from "@snowbridge/api"
import { cryptoWaitReady } from "@polkadot/util-crypto"
import { formatEther, Wallet } from "ethers"
import { assetRegistryFor, environmentFor } from "@snowbridge/registry"
import { IERC20__factory } from "@snowbridge/contract-types"
import { ETHER_TOKEN_ADDRESS } from "@snowbridge/api/dist/assets_v2"

export const transferToPolkadot = async (
    l2ChainId: number,
    destParaId: number,
    symbol: string,
    amount: bigint,
) => {
    await cryptoWaitReady()

    let env = "local_e2e"
    if (process.env.NODE_ENV !== undefined) {
        env = process.env.NODE_ENV
    }
    console.log(`Using environment '${env}'`)

    const context = new Context(environmentFor(env))

    const polkadot_keyring = new Keyring({ type: "sr25519" })

    const ETHEREUM_ACCOUNT = new Wallet(
        process.env.ETHEREUM_KEY ?? "Your Key Goes Here",
        context.ethChain(l2ChainId),
    )
    const ETHEREUM_ACCOUNT_PUBLIC = await ETHEREUM_ACCOUNT.getAddress()
    const POLKADOT_ACCOUNT = polkadot_keyring.addFromUri(process.env.SUBSTRATE_KEY ?? "//Ferdie")
    const POLKADOT_ACCOUNT_PUBLIC = POLKADOT_ACCOUNT.address

    console.log("eth", ETHEREUM_ACCOUNT_PUBLIC, "sub", POLKADOT_ACCOUNT_PUBLIC)

    const registry = assetRegistryFor(env)

    const assets = registry.ethereumChains[l2ChainId].assets
    const TOKEN_CONTRACT = Object.keys(assets)
        .map((t) => assets[t])
        .find((asset) => asset.symbol.toLowerCase().startsWith(symbol.toLowerCase()))?.token
    if (!TOKEN_CONTRACT) {
        console.error("no token contract exists, check it and rebuild asset registry.")
        throw Error(`No token found for ${symbol}`)
    }

    console.log("TOKEN_CONTRACT", TOKEN_CONTRACT)

    if (TOKEN_CONTRACT != ETHER_TOKEN_ADDRESS) {
        console.log("# Approve")
        const erc20 = IERC20__factory.connect(TOKEN_CONTRACT, ETHEREUM_ACCOUNT)
        const l2AdapterAddress = await context.l2Adapter(l2ChainId).getAddress()
        const [balance, allowance] = await Promise.all([
            erc20.balanceOf(ETHEREUM_ACCOUNT_PUBLIC),
            erc20.allowance(ETHEREUM_ACCOUNT_PUBLIC, l2AdapterAddress),
        ])

        if (allowance <= amount) {
            // Step 1: Reset allowance to 0 (required by this ERC20 implementation)
            console.log("Resetting allowance to 0...")
            const resetTx = await erc20.approve(l2AdapterAddress, 0n)
            await resetTx.wait()

            // Step 2: Set new allowance (higher than transfer amount for gateway fees)
            const approveAmount = amount * 10n // 10x buffer
            console.log("Setting new allowance to", approveAmount.toString())
            const approveTx = await erc20.approve(l2AdapterAddress, approveAmount)
            await approveTx.wait()

            const newAllowance = await erc20.allowance(ETHEREUM_ACCOUNT_PUBLIC, l2AdapterAddress)
            console.log("newAllowance", newAllowance.toString())
        }
    }

    console.log("Ethereum to Polkadot")
    {
        // Step 0. Create a transfer implementation
        const transferImpl = toPolkadotSnowbridgeV2.createL2TransferImplementation(
            l2ChainId,
            destParaId,
            registry,
            TOKEN_CONTRACT,
        )
        // Step 1. Get the delivery fee for the transaction
        let fee = await transferImpl.getDeliveryFee(
            context,
            registry,
            l2ChainId,
            TOKEN_CONTRACT,
            amount,
            destParaId,
        )

        console.log("fee: ", fee)
        // Step 2. Create a transfer tx
        const transfer = await transferImpl.createTransfer(
            context,
            registry,
            l2ChainId,
            TOKEN_CONTRACT,
            amount,
            destParaId,
            ETHEREUM_ACCOUNT_PUBLIC,
            POLKADOT_ACCOUNT_PUBLIC,
            fee,
        )

        // Step 3. Validate the transaction.
        const validation = await transferImpl.validateTransfer(context, transfer)
        console.log("validation result", validation)

        // Step 4. Check validation logs for errors
        if (validation.logs.find((l) => l.kind == toPolkadotV2.ValidationKind.Error)) {
            throw Error(`validation has one of more errors.` + JSON.stringify(validation.logs))
        }

        // Step 5. Estimate the cost of the execution cost of the transaction
        const {
            tx,
            computed: { totalValue },
        } = transfer
        const estimatedGas = await context.ethChain(l2ChainId).estimateGas(tx)
        const feeData = await context.ethChain(l2ChainId).getFeeData()
        const executionFee = (feeData.gasPrice ?? 0n) * estimatedGas

        console.log("tx:", tx)
        console.log("feeData:", feeData.toJSON())
        console.log("gas:", estimatedGas)
        console.log("relayer fee:", formatEther(fee.relayerFee))
        console.log("execution cost:", formatEther(executionFee))
        console.log("total cost:", formatEther(fee.totalFeeInWei + executionFee))
        console.log("ether sent:", formatEther(totalValue - fee.totalFeeInWei))
        console.log("dry run:", await context.ethereum().call(tx))

        if (process.env["DRY_RUN"] != "true") {
            console.log("sending tx")
            // Step 5. Submit the transaction
            const response = await ETHEREUM_ACCOUNT.sendTransaction(tx)
            console.log("sent transaction")
            const receipt = await response.wait(1)
            console.log("got receipt")
            if (!receipt || receipt.status != 1) {
                throw Error(`Transaction ${response.hash} not included.`)
            }

            console.log(
                `Success messages:
                block number: ${receipt.blockNumber}
                tx hash: ${receipt.hash}`,
            )
        }
    }
    await context.destroyContext()
}

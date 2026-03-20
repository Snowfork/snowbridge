import { Keyring } from "@polkadot/keyring"
import { assetsV2, createApi } from "@snowbridge/api"
import { TransferStatus } from "@snowbridge/api/dist/history_v2"
import { EthersEthereumProvider } from "@snowbridge/provider-ethers"
import { formatEther, Wallet } from "ethers"
import { cryptoWaitReady } from "@polkadot/util-crypto"
import { setTimeout } from "timers/promises"
import { bridgeInfoFor } from "@snowbridge/registry"
;(async () => {
    // Initialize polkadot-js crypto
    await cryptoWaitReady()

    // Get the registry of parachains and assets.
    const env = "polkadot_mainnet"
    const info = bridgeInfoFor(env)
    const { registry } = info

    // Initialize the context which establishes and pool connections
    const api = createApi({ info, ethereumProvider: new EthersEthereumProvider() })
    const context = api.context

    // Initialize ethereum wallet.
    const ETHEREUM_ACCOUNT = new Wallet(
        process.env.ETHEREUM_KEY ?? "Your Key Goes Here",
        context.ethereum(),
    )
    const ETHEREUM_ACCOUNT_PUBLIC = await ETHEREUM_ACCOUNT.getAddress()

    // Initialize substrate wallet.
    const polkadot_keyring = new Keyring({ type: "sr25519" })
    const POLKADOT_ACCOUNT = polkadot_keyring.addFromUri(
        process.env.SUBSTRATE_KEY ?? "Your Key Goes Here",
    )
    const POLKADOT_ACCOUNT_PUBLIC = POLKADOT_ACCOUNT.address

    console.log("eth", ETHEREUM_ACCOUNT_PUBLIC, "sub", POLKADOT_ACCOUNT_PUBLIC)

    // Select the token you want to send. In this case we use Ether. The registry contains the list of tokens.
    const TOKEN_CONTRACT = assetsV2.ETHER_TOKEN_ADDRESS

    // Select the destination parachain. In this case it is Asset Hub.
    const DESTINATION_PARACHAIN = 1000

    console.log("# Ethereum to Asset Hub")
    const transferImpl = api.sender(
        { kind: "ethereum", id: registry.ethChainId },
        { kind: "polkadot", id: DESTINATION_PARACHAIN },
    )
    // Step 1. Get the delivery fee for the transaction
    const fee = await transferImpl.fee(TOKEN_CONTRACT)

    // Step 2. Create a transfer tx.
    const amount = 15_000_000_000_000n // 0.000015 ETH
    const transfer = await transferImpl.tx(
        ETHEREUM_ACCOUNT_PUBLIC, // Source account
        POLKADOT_ACCOUNT_PUBLIC, // Destination account
        TOKEN_CONTRACT, // The erc20 token contract address
        amount, // Transfer Amount
        fee, // The delivery fee
    )

    // Step 3. Validate the transaction by dry-running on source and destination.
    const validation = await transferImpl.validate(transfer)
    console.log("validation result", validation)

    // Step 4. Check validation logs for dry errors
    if (!validation.success) {
        console.error(validation.logs)
        throw Error(`validation has one of more errors.`)
    }

    // Estimate the cost of the execution cost of the transaction
    const {
        tx,
        computed: { totalValue },
    } = transfer

    // Viewing gas and tx cost
    console.log("tx:", tx)
    console.log("Gas price quoted:", validation.data.feeInfo?.feeData)
    console.log("Transaction Gas Cost:", validation.data.feeInfo?.estimatedGas)

    console.log("Delivery Fee:", formatEther(fee.totalFeeInWei))
    console.log("Execution Fee:", formatEther(validation.data.feeInfo?.executionFee ?? 0n))
    console.log(
        "Total cost:",
        formatEther(fee.totalFeeInWei + (validation.data.feeInfo?.executionFee ?? 0n)),
    )
    console.log("Ether sent:", formatEther(totalValue - fee.totalFeeInWei))

    // Step 5. Submit the transaction
    const response = await ETHEREUM_ACCOUNT.sendTransaction(tx)
    const receipt = await response.wait(1)
    if (!receipt) {
        throw Error(`Transaction ${response.hash} not included.`)
    }

    // Step 6. Get the message receipt for tracking purposes
    const message = await transferImpl.messageId(receipt)
    if (!message) {
        throw Error(`Transaction ${receipt.hash} did not emit a message.`)
    }
    const messageId = transfer.computed.topic
    console.log(
        `Success message with message id: ${messageId}
                nonce: ${message.nonce}
                block number: ${message.blockNumber}
                tx hash: ${message.txHash}`,
    )

    // Step 7. Poll for message completion
    while (true) {
        const status = await api.txStatus(messageId)
        if (status !== undefined && status.status !== TransferStatus.Pending) {
            console.dir(status, { depth: 100 })
            console.log("tx complete:", TransferStatus[status.status])
            break
        }
        console.dir(status, { depth: 100 })
        console.log("waiting for tx to be completed...")
        await setTimeout(60_000) // Wait 60 seconds between requests
    }

    // Clean up all open connections
    await api.destroy()
})()

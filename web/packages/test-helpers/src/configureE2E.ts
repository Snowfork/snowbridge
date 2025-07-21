import { ApiPromise, WsProvider, Keyring } from "@polkadot/api"
import { cryptoWaitReady } from "@polkadot/util-crypto"

const InitialFund = 100_000_000_000_000n
const SudoPubKey =
    process.env["sudo_pubkey"] ||
    "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"
const sudoAccount = "//Alice"

interface TransactionConfig {
    recipient: string
    senderParaId?: number
}

const sendBatchTransactions = async (wsPort: number, txs: TransactionConfig[]) => {
    // Connect to node
    let api = await ApiPromise.create({
        provider: new WsProvider("ws://127.0.0.1:" + wsPort.toString()),
    })
    api = await api.isReady

    // Initialize Keyring and add an account (Replace with your private key or use mnemonic)
    const keyring = new Keyring({ type: "sr25519" })
    const sender = keyring.addFromUri(sudoAccount)
    await cryptoWaitReady()

    // Check if 'balances' is available in the API
    if (!api.tx.balances || !api.tx.balances.transferAllowDeath) {
        throw new Error("Balances module is not available in this network.")
    }

    // Define recipient addresses and amounts (replace with real addresses)
    // Create transactions
    const transactions = txs.map(({ recipient, senderParaId }) =>
        senderParaId
            ? api.tx.hrmp.forceOpenHrmpChannel(senderParaId, parseInt(recipient), 8, 512)
            : api.tx.balances.transferAllowDeath(recipient, InitialFund)
    )

    console.log("Transactions: ", transactions)

    // Create a batch transaction
    const batchTx = api.tx.utility.batchAll(transactions)
    const finalTx = txs.some((tx) => tx.senderParaId)
        ? (console.log("Sending sudo transaction..."), api.tx.sudo.sudo(batchTx))
        : (console.log("Sending batch transaction..."), batchTx)

    // Sign and send the batch transaction
    finalTx.signAndSend(sender, ({ status }) => {
        if (status.isInBlock) {
            console.log(`✅ Transaction included in block: ${status.asInBlock}`)
        }
    })
}

const sendBatchTransactionsOnPenpal = async () => {
    // Connect to node
    const api = await ApiPromise.create({ provider: new WsProvider("ws://127.0.0.1:13144") })

    // Initialize Keyring and add an account (Replace with your private key or use mnemonic)
    const keyring = new Keyring({ type: "sr25519" })
    const sender = keyring.addFromUri(sudoAccount)

    const versionedLocation = api.createType("XcmVersionedLocation", {
        v4: {
            parents: 1,
            interior: {
                x1: [
                    {
                        accountId32: {
                            network: null,
                            id: SudoPubKey,
                        },
                    },
                ],
            },
        },
    })

    // Define recipient addresses and amounts (replace with real addresses)
    const transactions = [
        //Account for AH sovereign
        api.tx.balances.transferAllowDeath(
            "5Eg2fntNprdN3FgH4sfEaaZhYtddZQSQUqvYJ1f2mLtinVhV",
            InitialFund
        ),
        //Checking account
        api.tx.balances.transferAllowDeath(
            "5EYCAe5ijiYgWYWi1fs8Xz1td1djEtJVVnNfzvDRP4VtLL7Y",
            InitialFund
        ),
        api.tx.polkadotXcm.addAuthorizedAlias(versionedLocation, null),
    ]

    // Create a batch transaction
    const batchTx = api.tx.utility.batchAll(transactions)

    console.log("Sending batch transaction...")

    // Sign and send the batch transaction
    batchTx.signAndSend(sender, ({ status }) => {
        if (status.isInBlock) {
            console.log(`✅ Transaction included in block: ${status.asInBlock}`)
        }
    })
}

const sleep = async (ms: number) => {
    return new Promise((resolve) => setTimeout(resolve, ms))
}

const main = async () => {
    // HRMP channel opening
    await sendBatchTransactions(9944, [
        { recipient: "1002", senderParaId: 1000 },
        { recipient: "1000", senderParaId: 1002 },
        { recipient: "2000", senderParaId: 1000 },
        { recipient: "1000", senderParaId: 2000 },
    ])
    // BridgeHub funding
    await sendBatchTransactions(11144, [
        { recipient: "5Eg2fntNprdN3FgH4sfEaaZhYtddZQSQUqvYJ1f2mLtinVhV" },
        { recipient: "5GWFwdZb6JyU46e6ZiLxjGxogAHe8SenX76btfq8vGNAaq8c" },
        { recipient: "5DF6KbMTBPGQN6ScjqXzdB2ngk5wi3wXvubpQVUZezNfM6aV" },
    ])
    // AssetHub funding
    await sendBatchTransactions(12144, [
        { recipient: "5Eg2fntJ27qsari4FGrGhrMqKFDRnkNSR6UshkZYBGXmSuC8" },
        { recipient: "5GjRnmh5o3usSYzVmsxBWzHEpvJyHK4tKNPhjpUR3ASrruBy" },
    ])
    // Relaychain funding
    await sendBatchTransactions(9944, [
        { recipient: "5DF6KbMTBPGQN6ScjqXzdB2ngk5wi3wXvubpQVUZezNfM6aV" },
    ])
    await sendBatchTransactionsOnPenpal()
}

// Run the script
main()
    .then(async () => {
        await sleep(3000) // Wait for transactions to be processed
        console.log("All transactions sent successfully.")
        process.exit(0)
    })
    .catch(console.error)

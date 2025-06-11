import { ApiPromise, WsProvider, Keyring } from "@polkadot/api"
import { cryptoWaitReady } from "@polkadot/util-crypto"

const InitialFund = 100_000_000_000_000n


const sendBatchTransactions = async (wsPort: number, txs: string[]) => {
    // Connect to node
    let api = await ApiPromise.create({
        provider: new WsProvider("ws://127.0.0.1:" + wsPort.toString()),
    })
    api = await api.isReady

    // Initialize Keyring and add an account (Replace with your private key or use mnemonic)
    const keyring = new Keyring({ type: "sr25519" })
    const sender = keyring.addFromUri("//Alice")
    await cryptoWaitReady()

    // Check if 'balances' is available in the API
    if (!api.tx.balances || !api.tx.balances.transferAllowDeath) {
        throw new Error("Balances module is not available in this network.")
    }

    // Define recipient addresses and amounts (replace with real addresses)
    // Create transactions
    const transactions = txs.map((recipient) =>
        api.tx.balances.transferAllowDeath(recipient, InitialFund)
    )

    // Create a batch transaction
    const batchTx = api.tx.utility.batchAll(transactions)

    console.log("Sending batch transaction...")

    // Sign and send the batch transaction
    const unsub = await batchTx.signAndSend(sender, ({ status }) => {
        if (status.isInBlock) {
            console.log(`✅ Transaction included in block: ${status.asInBlock}`)
        } else if (status.isFinalized) {
            console.log(`🎉 Transaction finalized in block: ${status.asFinalized}`)
            unsub()
        }
    })
}

const buildHrmpChannels = async () => {
    // Connect to node
    let api = await ApiPromise.create({ provider: new WsProvider("ws://127.0.0.1:9944") })
    api = await api.isReady

    // Initialize Keyring and add an account (Replace with your private key or use mnemonic)
    const keyring = new Keyring({ type: "sr25519" })
    const sender = keyring.addFromUri("//Alice")
    await cryptoWaitReady()

    const transactions = [
        api.tx.hrmp.forceOpenHrmpChannel(1000, 1002, 8, 512),
        api.tx.hrmp.forceOpenHrmpChannel(1002, 1000, 8, 512),
        api.tx.hrmp.forceOpenHrmpChannel(1000, 2000, 8, 512),
        api.tx.hrmp.forceOpenHrmpChannel(2000, 1000, 8, 512),
    ]

    // Create a batch transaction
    const batchTx = api.tx.utility.batchAll(transactions)
    const sudoTx = api.tx.sudo.sudo(batchTx)

    console.log("Sending sudo transaction...")

    // Sign and send the batch transaction
    const unsub = await sudoTx.signAndSend(sender, ({ status }) => {
        if (status.isInBlock) {
            console.log(`✅ Transaction included in block: ${status.asInBlock}`)
        } else if (status.isFinalized) {
            console.log(`🎉 Transaction finalized in block: ${status.asFinalized}`)
            unsub()
        }
    })
}

const main = async () => {
    await buildHrmpChannels()
    // bridgehub funding
    await sendBatchTransactions(11144, [
        "5Eg2fntNprdN3FgH4sfEaaZhYtddZQSQUqvYJ1f2mLtinVhV",
        "5GWFwdZb6JyU46e6ZiLxjGxogAHe8SenX76btfq8vGNAaq8c",
        "5DF6KbMTBPGQN6ScjqXzdB2ngk5wi3wXvubpQVUZezNfM6aV",
    ])
    // assethub funding
    await sendBatchTransactions(12144, [
        "5Eg2fntJ27qsari4FGrGhrMqKFDRnkNSR6UshkZYBGXmSuC8",
        "5GjRnmh5o3usSYzVmsxBWzHEpvJyHK4tKNPhjpUR3ASrruBy",
    ])
    // relaychain funding
    await sendBatchTransactions(9944, ["5DF6KbMTBPGQN6ScjqXzdB2ngk5wi3wXvubpQVUZezNfM6aV"])
}

// Run the script
main()
    .then(() => {
        console.log("initial fund finished")
        process.exit(0)
    })
    .catch(console.error)

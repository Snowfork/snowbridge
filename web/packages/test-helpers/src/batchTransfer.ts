import { ApiPromise, WsProvider, Keyring } from "@polkadot/api"
import { cryptoWaitReady } from "@polkadot/util-crypto"

const InitialFund = 100_000_000_000_000n

const sendBatchTransactionsOnBridgehub = async () => {
    // Connect to node
    let api = await ApiPromise.create({ provider: new WsProvider("ws://127.0.0.1:11144") })
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
    const transactions = [
        //Account for assethub (Sibling parachain 1000)
        api.tx.balances.transferAllowDeath(
            "5Eg2fntNprdN3FgH4sfEaaZhYtddZQSQUqvYJ1f2mLtinVhV",
            InitialFund
        ),
        //BeaconRelay
        api.tx.balances.transferAllowDeath(
            "5GWFwdZb6JyU46e6ZiLxjGxogAHe8SenX76btfq8vGNAaq8c",
            InitialFund
        ),
        //ExecutionRelayAssetHub
        api.tx.balances.transferAllowDeath(
            "5DF6KbMTBPGQN6ScjqXzdB2ngk5wi3wXvubpQVUZezNfM6aV",
            InitialFund
        ),
    ]

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

const sendBatchTransactionsOnAssethub = async () => {
    // Connect to node
    const api = await ApiPromise.create({ provider: new WsProvider("ws://127.0.0.1:12144") })

    // Initialize Keyring and add an account (Replace with your private key or use mnemonic)
    const keyring = new Keyring({ type: "sr25519" })
    const sender = keyring.addFromUri("//Alice")

    // Define recipient addresses and amounts (replace with real addresses)
    const transactions = [
        //Account for penpal (Sibling parachain 2000)
        api.tx.balances.transferAllowDeath(
            "5Eg2fntJ27qsari4FGrGhrMqKFDRnkNSR6UshkZYBGXmSuC8",
            InitialFund
        ),
        //Account for snowbridge sovereign
        api.tx.balances.transferAllowDeath(
            "5GjRnmh5o3usSYzVmsxBWzHEpvJyHK4tKNPhjpUR3ASrruBy",
            InitialFund
        ),
    ]

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

const main = async () => {
    await sendBatchTransactionsOnBridgehub()
    await sendBatchTransactionsOnAssethub()
}

// Run the script
main()
    .then(() => {
        console.log("initial fund finshed")
        process.exit(0)
    })
    .catch(console.error)

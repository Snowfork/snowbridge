import { ApiPromise, WsProvider, Keyring } from "@polkadot/api"
import { cryptoWaitReady } from "@polkadot/util-crypto"

const InitialFund = 100_000_000_000_000n
const SudoPubKey =
    process.env["sudo_pubkey"] ||
    "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"
const sudoAccount = "//Alice"

const authorizedAliasLocation = {
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
}

interface TransactionConfig {
    recipient?: string
    senderParaId?: number
    authorizedAlias?: any
}

const sendBatchTransactions = async (
    wsPort: number,
    txs: TransactionConfig[],
    senderAddress?: string,
    noSending?: boolean
) => {
    // Connect to node
    let api = await ApiPromise.create({
        provider: new WsProvider("ws://127.0.0.1:" + wsPort.toString()),
    })
    api = await api.isReady

    // Initialize Keyring and add an account (Replace with your private key or use mnemonic)
    const keyring = new Keyring({ type: "sr25519" })
    let sender
    if (senderAddress) {
        console.log("Using provided sender account:", senderAddress)
        sender = keyring.addFromUri(senderAddress)
    } else {
        sender = keyring.addFromUri(sudoAccount)
    }
    // Check if 'balances' is available in the API
    if (!api.tx.balances || !api.tx.balances.transferAllowDeath) {
        throw new Error("Balances module is not available in this network.")
    }

    // Define recipient addresses and amounts (replace with real addresses)
    // Create transactions
    const transactions = txs.map(({ recipient, senderParaId, authorizedAlias }) =>
        recipient
            ? senderParaId
                ? api.tx.hrmp.forceOpenHrmpChannel(senderParaId, parseInt(recipient), 8, 512)
                : api.tx.balances.transferAllowDeath(recipient, InitialFund)
            : (function () {
                  const versionedLocation = api.createType("XcmVersionedLocation", authorizedAlias)
                  return api.tx.polkadotXcm.addAuthorizedAlias(versionedLocation, null)
              })()
    )
    // Create a batch transaction
    const batchTx = api.tx.utility.batchAll(transactions)
    const finalTx = txs.some((tx) => tx.senderParaId)
        ? (console.log("Building sudo transaction..."), api.tx.sudo.sudo(batchTx))
        : (console.log("Building batch transaction..."), batchTx)

    if (noSending) {
        console.log("Transaction not sent due to noSending flag.")
        return Promise.resolve({ sender, tx: finalTx })
    }
    // Sign and send the batch transaction
    finalTx.signAndSend(sender, ({ status }) => {
        if (status.isInBlock) {
            console.log(`✅ Transaction included in block: ${status.asInBlock}`)
        }
    })
}

const main = async () => {
    await cryptoWaitReady()
    // Relaychain funding, use a different account and leave sudo account for sudo txs
    console.log("sending Relay txs")
    await sendBatchTransactions(
        9944,
        [{ recipient: "5DF6KbMTBPGQN6ScjqXzdB2ngk5wi3wXvubpQVUZezNfM6aV" }],
        "//Bob"
    )
    // BridgeHub funding
    console.log("sending BridgeHub txs")
    await sendBatchTransactions(11144, [
        { recipient: "5Eg2fntNprdN3FgH4sfEaaZhYtddZQSQUqvYJ1f2mLtinVhV" },
        { recipient: "5GWFwdZb6JyU46e6ZiLxjGxogAHe8SenX76btfq8vGNAaq8c" },
        { recipient: "5DF6KbMTBPGQN6ScjqXzdB2ngk5wi3wXvubpQVUZezNfM6aV" },
    ])
    // AssetHub funding
    console.log("sending AssetHub txs")
    await sendBatchTransactions(12144, [
        { recipient: "5Eg2fntJ27qsari4FGrGhrMqKFDRnkNSR6UshkZYBGXmSuC8" },
        { recipient: "5GjRnmh5o3usSYzVmsxBWzHEpvJyHK4tKNPhjpUR3ASrruBy" },
        { recipient: "5EYCAe5ijiYgWYWi1fs8Xz1td1djEtJVVnNfzvDRP4VtLL7Y" },
        { authorizedAlias: authorizedAliasLocation },
    ])
    console.log("sending Penpal txs")
    // Penpal funding
    await sendBatchTransactions(13144, [
        { recipient: "5Eg2fntNprdN3FgH4sfEaaZhYtddZQSQUqvYJ1f2mLtinVhV" },
        { recipient: "5EYCAe5ijiYgWYWi1fs8Xz1td1djEtJVVnNfzvDRP4VtLL7Y" },
        { authorizedAlias: authorizedAliasLocation },
    ])
    // HRMP channel opening
    console.log("sending HRMP channel opening txs")
    return (await sendBatchTransactions(
        9944,
        [
            { recipient: "1002", senderParaId: 1000 },
            { recipient: "1000", senderParaId: 1002 },
            { recipient: "2000", senderParaId: 1000 },
            { recipient: "1000", senderParaId: 2000 },
        ],
        sudoAccount,
        true
    ))!
}

// Run the script
main()
    .then(async ({ tx, sender }) => {
        console.log("Waiting for the last transaction to be included...")
        const unsub = await tx?.signAndSend(sender, ({ status }) => {
            if (status.isInBlock) {
                console.log(
                    `✅ Final Transaction included in block: ${status.asInBlock}, all transactions sent successfully.`
                )
                unsub!()
                process.exit(0)
            }
        })
    })
    .catch(console.error)

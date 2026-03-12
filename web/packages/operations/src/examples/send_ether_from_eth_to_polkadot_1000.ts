import { Keyring } from "@polkadot/keyring"
import { assetsV2, createApi, TransferStatus } from "@snowbridge/api"
import { EthersEthereumProvider } from "@snowbridge/provider-ethers"
import { polkadot_mainnet } from "@snowbridge/registry"
import { getDefaultProvider, Wallet } from "ethers"
import { cryptoWaitReady } from "@polkadot/util-crypto"
import { setTimeout } from "timers/promises"
;(async () => {
    await cryptoWaitReady()
    // Wallet Setup
    const ETHEREUM_ACCOUNT = new Wallet(
        process.env.ETHEREUM_KEY ?? "Your Key Goes Here",
        getDefaultProvider("mainnet"),
    )
    const ETHEREUM_ACCOUNT_PUBLIC = await ETHEREUM_ACCOUNT.getAddress()

    const polkadotKeyring = new Keyring({ type: "sr25519" })
    const POLKADOT_ACCOUNT = polkadotKeyring.addFromUri(
        process.env.SUBSTRATE_KEY ?? "Your Key Goes Here",
    )
    const POLKADOT_ACCOUNT_PUBLIC = POLKADOT_ACCOUNT.address

    console.log("eth", ETHEREUM_ACCOUNT_PUBLIC, "sub", POLKADOT_ACCOUNT_PUBLIC)

    // 1. Initialize API
    const api = createApi({
        info: polkadot_mainnet,
        ethereumProvider: new EthersEthereumProvider(),
    })

    // 2. Get a send builder
    const {
        chains: { ethereum, assetHub },
    } = polkadot_mainnet
    const sender = api.sender(ethereum, assetHub)

    // 3. Get Fee
    const fee = await sender.fee(assetsV2.ETHER_TOKEN_ADDRESS)

    // 4. Build Tx
    const createdTransfer = await sender.rawTx(
        ETHEREUM_ACCOUNT_PUBLIC,
        POLKADOT_ACCOUNT_PUBLIC,
        assetsV2.ETHER_TOKEN_ADDRESS, // Ether address
        15_000_000_000_000n, // 0.000015 ETH,
        fee,
    )

    // 5. Dry-run Tx
    const validation = await sender.validate(createdTransfer)
    console.log("validation result", validation)

    // 6. Check errors
    if (!validation.success) {
        console.error(validation.logs)
        throw Error(`validation has one of more errors.`)
    }

    // 8. Send Tx wil wallet
    const response = await ETHEREUM_ACCOUNT.sendTransaction(createdTransfer.tx)
    const receipt = await response.wait(1)
    if (!receipt) {
        throw Error(`Transaction ${response.hash} not included.`)
    }

    // 9. Get message id
    const message = await sender.messageId(receipt)
    if (!message) {
        throw Error(`Transaction ${receipt.hash} did not emit a message.`)
    }
    const messageId = createdTransfer.computed.topic
    console.log(
        `Success message with message id: ${messageId}
                nonce: ${message.nonce}
                block number: ${message.blockNumber}
                tx hash: ${message.txHash}`,
    )

    // 10. Poll Tx Status
    while (true) {
        const status = await api.txStatus(messageId)
        if (status !== undefined && status.status !== TransferStatus.Pending) {
            console.dir(status, { depth: 100 })
            console.log("tx complete:", TransferStatus[status.status])
            break
        }
        console.dir(status, { depth: 100 })
        console.log("waiting for tx to be completed...")
        await setTimeout(60_000)
    }

    // 11. Cleanup api
    await api.destroy()
})()

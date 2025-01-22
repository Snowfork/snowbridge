import "dotenv/config"
import { executeTransfer } from "./index"
import { ethers } from "ethers"
import { WsProvider, ApiPromise } from "@polkadot/api"

// XC20 address referenced from the list in https://docs.moonbeam.network/builders/interoperability/xcm/xc20/overview/
const xc20TokenAddress = "0xfFffFFFF86829AFE1521AD2296719DF3ACE8DED7".toLowerCase()

// beneficiary
const beneficiary = process.env["BENEFICIARY"] || "0x302F0B71B8aD3CF6dD90aDb668E49b2168d652fd"

// Private key of the ethereum signer
const privateKey = process.env["PRIVATE_KEY"] || "INSERT_YOUR_PRIVATE_KEY"

const ethereumProviderURL = process.env["MOONBEAM_URL"] || "https://rpc.api.moonbeam.network"
const ASSETHUB_WS_URL = process.env["ASSETHUB_WS_URL"] || "wss://asset-hub-polkadot-rpc.dwellir.com"
const MOONBEAM_WS_URL = process.env["MOONBEAM_WS_URL"] || "wss://moonbeam-rpc.n.dwellir.com"

const run = async () => {
    const provider = new ethers.JsonRpcProvider(ethereumProviderURL)
    const signer = new ethers.Wallet(privateKey, provider)
    const api = await ApiPromise.create({ provider: new WsProvider(MOONBEAM_WS_URL) })
    const assetHubApi = await ApiPromise.create({
        provider: new WsProvider(ASSETHUB_WS_URL),
    })
    // 0.001 WETH
    const amount = 1000000000000000
    await executeTransfer(signer, api, assetHubApi, xc20TokenAddress, amount, beneficiary)
}

run()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error("Error:", error)
        process.exit(1)
    })

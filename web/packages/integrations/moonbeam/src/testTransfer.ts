import "dotenv/config"
import { executeTransfer } from "./index"
import { ethers } from "ethers"
import { WsProvider, ApiPromise } from "@polkadot/api"

// XC20 address for WEth
const xc20TokenAddress = "0xfFffFFFF86829AFE1521AD2296719DF3ACE8DED7".toLowerCase()

const beneficiary = process.env["BENEFICIARY"] || "0x302F0B71B8aD3CF6dD90aDb668E49b2168d652fd"

// The claimer is the address on asset hub
const claimer =
    process.env["CLAIMER"] || "0x5628194e9f9ff8bd593f490fcafd033289f393e2ba860c6c51bca39c01091b39"

// The private key of the ethereum signer
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
    // transfer 0.0002 WEth
    const amount = 200_000_000_000_000
    await executeTransfer(signer, api, assetHubApi, xc20TokenAddress, amount, beneficiary, claimer)
}

run()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error("Error:", error)
        process.exit(1)
    })

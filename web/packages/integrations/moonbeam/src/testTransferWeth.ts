import "dotenv/config"
import { executeTransfer } from "./index"
import { ethers } from "ethers"
import { WsProvider, ApiPromise } from "@polkadot/api"

// XC20 address referenced from the list in https://docs.moonbeam.network/builders/interoperability/xcm/xc20/overview/
const xc20TokenAddress = "0xfFffFFFF86829AFE1521AD2296719DF3ACE8DED7".toLowerCase()

// Todo: should get ERC20 from XC20
const erc20TokenAddress = "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2"

const eth_beneficiary = "0x302F0B71B8aD3CF6dD90aDb668E49b2168d652fd"

// 0.001 WETH
const amount = 1000000000000000

const ethereumProviderURL = process.env["MOONBEAM_URL"] || "https://rpc.api.moonbeam.network"
const privateKey = process.env["PRIVATE_KEY"] || "INSERT_YOUR_PRIVATE_KEY"
const provider = new ethers.JsonRpcProvider(ethereumProviderURL)
const signer = new ethers.Wallet(privateKey, provider)
const ASSETHUB_WS_URL = process.env["ASSETHUB_WS_URL"] || "wss://asset-hub-polkadot-rpc.dwellir.com"
const MOONBEAM_WS_URL = process.env["MOONBEAM_WS_URL"] || "wss://moonbeam-rpc.n.dwellir.com"

const run = async () => {
    const api = await ApiPromise.create({ provider: new WsProvider(MOONBEAM_WS_URL) })
    const assetHubApi = await ApiPromise.create({
        provider: new WsProvider(ASSETHUB_WS_URL),
    })
    await executeTransfer(
        signer,
        api,
        assetHubApi,
        xc20TokenAddress,
        erc20TokenAddress,
        amount,
        eth_beneficiary
    )
}

run()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error("Error:", error)
        process.exit(1)
    })

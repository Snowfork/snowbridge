import { ApiPromise } from "@polkadot/api"
import { erc20Location } from "../xcmBuilder"
import { getNativeBalance } from "../assets_v2"

export const MUSE_CHAIN_ID = 11155111 // Sepolia
export const MUSE_TOKEN_ID = "0xb34a6924a02100ba6ef12af1c798285e8f7a16ee"
export const MYTHOS_CHAIN_ID = 1 // Ethereum Mainnet
export const MYTHOS_TOKEN_ID = "0xba41ddf06b7ffd89d1267b5a93bfef2424eb2003"

export async function getMythosLocationBalance(
    location: any,
    provider: ApiPromise,
    specName: string,
    account: string
) {
    if (
        specName === "muse" &&
        JSON.stringify(location) == JSON.stringify(erc20Location(MUSE_CHAIN_ID, MUSE_TOKEN_ID))
    ) {
        return await getNativeBalance(provider, account)
    } else if (
        specName === "mythos" &&
        JSON.stringify(location) == JSON.stringify(erc20Location(MYTHOS_CHAIN_ID, MYTHOS_TOKEN_ID))
    ) {
        return await getNativeBalance(provider, account)
    } else {
        throw Error(
            `Cannot get balance for spec ${specName}. Location = ${JSON.stringify(location)}`
        )
    }
}

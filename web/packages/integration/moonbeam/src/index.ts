import { ApiPromise, WsProvider } from "@polkadot/api"
import { ethers } from "ethers"
import { Precompiles_XcmUtil_sol_XcmUtils__factory } from "./bindings"

const xcmUtilsAddress = "0x000000000000000000000000000000000000080C"
const substrateProviderURL =
    process.env["MOONBEAM_SUBSTRATE_URL"] || "wss://wss.api.moonbase.moonbeam.network"
const ethereumProviderURL =
    process.env["MOONBEAM_Ethereum_URL"] || "https://rpc.api.moonbase.moonbeam.network"

const getEncodedXcmMessage = async (api: ApiPromise, beneficiary: string) => {
    const instr1 = {
        WithdrawAsset: [
            {
                id: { parents: 0, interior: { X1: [{ PalletInstance: 3 }] } },
                fun: { Fungible: 100000000000000000n },
            },
        ],
    }
    const instr2 = {
        DepositAsset: {
            assets: { Wild: { AllCounted: 1 } },
            beneficiary: {
                parents: 0,
                interior: {
                    X1: [
                        {
                            AccountKey20: {
                                key: beneficiary,
                            },
                        },
                    ],
                },
            },
        },
    }
    const message = { V4: [instr1, instr2] }
    const maxWeight = { refTime: 7250000000n, proofSize: 19374n }

    const tx = api.tx.polkadotXcm.execute(message, maxWeight)

    const encodedXcmMessage = tx.args[0].toHex()
    console.log(`Encoded Calldata for XCM Message: ${encodedXcmMessage}`)

    return encodedXcmMessage
}

export const executeXcmMessage = async () => {
    const substrateProvider = new WsProvider(substrateProviderURL)
    const api = await ApiPromise.create({ provider: substrateProvider })

    const privateKey = process.env["PRIVATE_KEY"] || "INSERT_YOUR_PRIVATE_KEY"
    const provider = new ethers.JsonRpcProvider(ethereumProviderURL)
    const signer = new ethers.Wallet(privateKey, provider)

    const xcmUtils = Precompiles_XcmUtil_sol_XcmUtils__factory.connect(xcmUtilsAddress, signer)
    const encodedCalldata = await getEncodedXcmMessage(api, "")
    const maxWeight = "400000000"

    /* Execute the custom XCM message */
    const tx = await xcmUtils.xcmExecute(encodedCalldata, maxWeight)
    await tx.wait()
    console.log(`Transaction receipt: ${tx.hash}`)
}

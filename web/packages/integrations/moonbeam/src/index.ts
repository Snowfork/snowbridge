import { ApiPromise } from "@polkadot/api"
import { blake2AsHex } from "@polkadot/util-crypto"
import { BigNumberish, BytesLike, Wallet } from "ethers"
import { Precompiles_XcmInterface_sol_XCM__factory, XCM } from "./bindings"
import { numberToHex } from "@polkadot/util"

// https://github.com/moonbeam-foundation/moonbeam/blob/b2b1bde7ced13aad4bd2928effc415c521fd48cb/runtime/moonbeam/src/precompiles.rs#L281
const xcmInterfacePrecompile = "0x000000000000000000000000000000000000081A"
const ETH_CHAIN_ID = process.env["ETH_CHAIN_ID"] || 1

export const executeTransfer = async (
    signer: Wallet,
    api: ApiPromise,
    assetHubApi: ApiPromise,
    xc20TokenAddress: string,
    erc20tokenAddress: string,
    amount: BigNumberish,
    beneficiary: string
) => {
    const xcmInterface = Precompiles_XcmInterface_sol_XCM__factory.connect(
        xcmInterfacePrecompile,
        signer
    )

    const BRIDGE_LOCATION = {
        parents: 2,
        interior: {
            X1: [{ GlobalConsensus: { Ethereum: { chain_id: ETH_CHAIN_ID } } }],
        },
    }
    const ERC20_TOKEN_LOCATION = {
        parents: 2,
        interior: {
            X2: [
                { GlobalConsensus: { Ethereum: { chain_id: ETH_CHAIN_ID } } },
                { AccountKey20: { key: erc20tokenAddress } },
            ],
        },
    }
    const ERC20_TOKEN_LOCATION_REANCHORED = {
        parents: 0,
        interior: { X1: [{ AccountKey20: { key: erc20tokenAddress } }] },
    }

    const customXcm = [
        // Initiate the bridged transfer
        {
            initiateReserveWithdraw: {
                assets: {
                    Wild: {
                        AllOf: { id: ERC20_TOKEN_LOCATION, fun: "Fungible" },
                    },
                },
                reserve: BRIDGE_LOCATION,
                xcm: [
                    {
                        buyExecution: {
                            fees: {
                                id: ERC20_TOKEN_LOCATION_REANCHORED, // CAUTION: Must use reanchored locations.
                                fun: {
                                    Fungible: "1", // Offering 1 unit as fee, but it is returned to the destination address.
                                },
                            },
                            weight_limit: "Unlimited",
                        },
                    },
                    {
                        depositAsset: {
                            assets: {
                                Wild: {
                                    AllCounted: 1,
                                },
                            },
                            beneficiary: {
                                parents: 0,
                                interior: { x1: [{ AccountKey20: { key: beneficiary } }] },
                            },
                        },
                    },
                    {
                        setTopic:
                            "0x0000000000000000000000000000000000000000000000000000000000000000",
                    },
                ],
            },
        },
        {
            setTopic: "0x0000000000000000000000000000000000000000000000000000000000000000",
        },
    ]
    const xcmHash = assetHubApi.createType("Xcm", customXcm)
    const sender = await signer.getAddress()
    const [parachainId, accountNextId] = await Promise.all([
        api.query.parachainInfo.parachainId(),
        api.rpc.system.accountNextIndex(sender),
    ])
    const entropy = new Uint8Array([
        ...parachainId.toU8a(),
        ...accountNextId.toU8a(),
        ...xcmHash.toU8a(),
    ])
    const messageId = blake2AsHex(entropy)
    customXcm[0].initiateReserveWithdraw!.xcm[2].setTopic = messageId
    customXcm[1].setTopic = messageId
    const xcmOnDest = assetHubApi.createType("XcmVersionedXcm", {
        V4: customXcm,
    })
    let customXcmOnDest: BytesLike = xcmOnDest.toHex()

    let token: XCM.AssetAddressInfoStruct = {
        asset: xc20TokenAddress,
        amount,
    }
    // Fee always in xcDOT, hardcode as 6.5 DOT for now should be enough referenced from https://moonbeam.subscan.io/extrinsic/8501143-6
    // Todo: calculate the fee amount on demand
    let fee: XCM.AssetAddressInfoStruct = {
        asset: "0xFfFFfFff1FcaCBd218EDc0EbA20Fc2308C778080",
        amount: 65_000_000_000,
    }

    // This represents (1,X1(Parachain(1000)))
    const paraIdInHex = numberToHex(1000, 32)
    const parachain_enum_selector = "0x00"
    let dest: XCM.LocationStruct = {
        parents: 1,
        interior: [parachain_enum_selector + paraIdInHex.slice(2)],
    }
    let assets: XCM.AssetAddressInfoStruct[] = [fee, token]
    // DestinationReserve for the asset
    let assetTransferType: BigNumberish = 2n
    // Fee as first asset
    let remoteFeesIdIndex: BigNumberish = 0n
    // DestinationReserve for the fee
    let feesTransferType: BigNumberish = 2n

    /* Execute the custom XCM message */
    const tx = await xcmInterface[
        "transferAssetsUsingTypeAndThenAddress((uint8,bytes[]),(address,uint256)[],uint8,uint8,uint8,bytes)"
    ](dest, assets, assetTransferType, remoteFeesIdIndex, feesTransferType, customXcmOnDest)
    await tx.wait()
    console.log(`Transaction receipt: ${tx.hash}`)
}

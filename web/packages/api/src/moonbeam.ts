import { ApiPromise } from "@polkadot/api"
import { blake2AsHex, decodeAddress, xxhashAsHex } from "@polkadot/util-crypto"
import { BigNumberish, BytesLike, Contract, Wallet } from "ethers"
import { BN, numberToHex, u8aToHex } from "@polkadot/util"
import { Codec } from "@polkadot/types/types"

// https://github.com/moonbeam-foundation/moonbeam/blob/b2b1bde7ced13aad4bd2928effc415c521fd48cb/runtime/moonbeam/src/precompiles.rs#L281
const xcmInterfacePrecompile = "0x000000000000000000000000000000000000081A"
const ETH_CHAIN_ID = process.env["ETH_CHAIN_ID"] || 1
const XCDOT = "0xFfFFfFff1FcaCBd218EDc0EbA20Fc2308C778080"
const ASSET_HUB_PARAID = 1000
const TokenPairs = [
    {
        id: "WETH",
        address: "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2",
        xc20Address: "0xfFffFFFF86829AFE1521AD2296719DF3ACE8DED7",
    },
    {
        id: "WBTC",
        address: "0x2260fac5e5542a773aa44fbcfedf7c193bc2c599",
        xc20Address: "0xfFffFFFf1B4Bb1ac5749F73D866FfC91a34g32c47",
    },
    {
        id: "wstETH",
        address: "0x7f39c581f595b53c5cb19bd0b3f8da6c935e2ca0",
        xc20Address: "0xFfFFFfFF5D5DEB44BF7278DEE5381BEB24CB6573",
    },
]

/**
 * Transfer Snowbridge ERC20 back to Ethereum
 * @param signer - The wallet to sign the transaction
 * @param api - ApiPromise for moonbeam
 * @param assetHubApi - ApiPromise for assethub
 * @param xc20TokenAddress - The xc20 address of the token
 * @param amount - amount of the token
 * @param beneficiary - The beneficiary address on ethereum
 * @param claimer - The claimer address is an optional param with the 32 bytes address on AssetHub to deposit into
 *                  when there is extra fee left or assets trapped
 */
export const executeTransferToEthereum = async (
    signer: Wallet,
    api: ApiPromise,
    assetHubApi: ApiPromise,
    xc20TokenAddress: string,
    amount: BigNumberish,
    beneficiary: string,
    claimer: string | null
) => {
    let erc20tokenAddress = getERC20TokenAddress(xc20TokenAddress)
    if (!erc20tokenAddress) {
        throw new Error("token not registed")
    }
    let abi = [
        {
            inputs: [
                {
                    components: [
                        { internalType: "uint8", name: "parents", type: "uint8" },
                        { internalType: "bytes[]", name: "interior", type: "bytes[]" },
                    ],
                    internalType: "struct XCM.Location",
                    name: "dest",
                    type: "tuple",
                },
                {
                    components: [
                        { internalType: "address", name: "asset", type: "address" },
                        { internalType: "uint256", name: "amount", type: "uint256" },
                    ],
                    internalType: "struct XCM.AssetAddressInfo[]",
                    name: "assets",
                    type: "tuple[]",
                },
                {
                    internalType: "enum XCM.TransferType",
                    name: "assetsTransferType",
                    type: "uint8",
                },
                { internalType: "uint8", name: "remoteFeesIdIndex", type: "uint8" },
                {
                    internalType: "enum XCM.TransferType",
                    name: "feesTransferType",
                    type: "uint8",
                },
                { internalType: "bytes", name: "customXcmOnDest", type: "bytes" },
            ],
            name: "transferAssetsUsingTypeAndThenAddress",
            outputs: [],
            stateMutability: "nonpayable",
            type: "function",
        },
    ]
    let xcmInterface = new Contract(xcmInterfacePrecompile, abi, signer)

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

    // construct xcm referenced https://moonbeam.subscan.io/extrinsic/8501143-6
    let setAppendixXcm: any[] = []
    if (claimer) {
        // add a `setAppendix` when claimer assigned
        setAppendixXcm = [
            {
                setAppendix: [
                    {
                        depositAsset: {
                            assets: {
                                Wild: "All",
                            },
                            beneficiary: {
                                parents: 0,
                                interior: {
                                    x1: [
                                        {
                                            AccountId32: {
                                                id: u8aToHex(decodeAddress(claimer)),
                                            },
                                        },
                                    ],
                                },
                            },
                        },
                    },
                ],
            },
        ]
    }
    let customXcm = [
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
    customXcm = setAppendixXcm.concat(customXcm)
    // Generate an unique messageId and set into `setTopic` for remote track
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
    if (customXcm.length == 2) {
        customXcm[0].initiateReserveWithdraw!.xcm[2].setTopic = messageId
        customXcm[1].setTopic = messageId
    } else if (customXcm.length == 3) {
        customXcm[1].initiateReserveWithdraw!.xcm[2].setTopic = messageId
        customXcm[2].setTopic = messageId
    } else {
        throw new Error("invalid xcm")
    }

    const xcmOnDest = assetHubApi.createType("XcmVersionedXcm", {
        V4: customXcm,
    })
    let customXcmOnDest: BytesLike = xcmOnDest.toHex()

    // Fee always in xcDOT
    // transfer_bridge_fee is the fee to cover the execution cost on ethereum
    // transfer_assethub_execution_fee is the fee to cover the execution cost on assethub
    // hardcode as 0.35 DOT should be fair enough, can improve with a dry run call
    const transfer_bridge_fee: bigint = await getSendFee(assetHubApi)
    const transfer_assethub_execution_fee = 3500000000n
    const transfer_fee = (transfer_bridge_fee + transfer_assethub_execution_fee).toString()

    // Execute the custom XCM message with the precompile
    const tx = await xcmInterface[
        "transferAssetsUsingTypeAndThenAddress((uint8,bytes[]),(address,uint256)[],uint8,uint8,uint8,bytes)"
    ](
        // This represents (1,X1(Parachain(1000)))
        [1, ["0x00" + numberToHex(ASSET_HUB_PARAID, 32).slice(2)]],
        // Assets including fee and the ERC20 asset, with fee be the first
        [
            [XCDOT, transfer_fee],
            [xc20TokenAddress, amount],
        ],
        // The TransferType corresponding to asset being sent, 2 represents `DestinationReserve`
        2,
        // index for the fee
        0,
        // The TransferType corresponding to fee asset
        2,
        customXcmOnDest
    )
    await tx.wait()
    console.log(`Transaction receipt: ${tx.hash}`)
}

export const getSendFee = async (
    assetHub: ApiPromise,
    options = {
        defaultFee: 65_000_000_000, //6.5 DOT by default
    }
): Promise<bigint> => {
    // Fees stored in 0x5fbc5c7ba58845ad1f1a9a7c5bc12fad
    const feeStorageKey = xxhashAsHex(":BridgeHubEthereumBaseFee:", 128, true)
    const feeStorageItem = await assetHub.rpc.state.getStorage(feeStorageKey)
    const leFee = new BN((feeStorageItem as Codec).toHex().replace("0x", ""), "hex", "le")
    return leFee.eqn(0) ? BigInt(options.defaultFee) : BigInt(leFee.toString())
}

// Get the ERC20 address from the XC20 through a static map
// Todo: From on-chain storage `assetManager.assetIdType`
const getERC20TokenAddress = (xc20TokenAddress: string): string => {
    for (let entry of TokenPairs) {
        if (entry.xc20Address.toLowerCase() == xc20TokenAddress.toLowerCase()) {
            return entry.address
        }
    }
    return ""
}

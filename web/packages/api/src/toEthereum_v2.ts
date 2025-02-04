import { ApiPromise } from "@polkadot/api";
import { AddressOrPair, SignerOptions, SubmittableExtrinsic } from "@polkadot/api/types";
import { Codec, ISubmittableResult } from "@polkadot/types/types";
import { BN, hexToU8a, isHex, stringToU8a, u8aToHex } from "@polkadot/util";
import { blake2AsHex, decodeAddress, xxhashAsHex } from "@polkadot/util-crypto";
import { bridgeLocation, buildResultXcmAssetHubERC20TransferFromParachain, buildAssetHubERC20TransferFromParachain, DOT_LOCATION, erc20Location, parahchainLocation } from "./xcmBuilder";
import { Asset, AssetRegistry, calculateDestinationFee, ERC20Metadata, getDotBalance, getNativeBalance, getParachainId, getTokenBalance, Parachain } from "./assets_v2";
import { getOperatingStatus } from "./status";
import { IGateway } from "@snowbridge/contract-types";
import { CallDryRunEffects, EventRecord, XcmDryRunApiError } from "@polkadot/types/interfaces";
import { Result } from "@polkadot/types";

export type Transfer = {
    input: {
        registry: AssetRegistry
        sourceAccount: string
        beneficiaryAccount: any
        tokenAddress: string
        amount: bigint
        fee: DeliveryFee
    },
    computed: {
        sourceParaId: number
        sourceAccountHex: string
        tokenErcMetadata: ERC20Metadata
        ahAssetMetadata: Asset
        sourceAssetMetadata: Asset
        sourceParachain: Parachain
        messageId?: string
    },
    tx: SubmittableExtrinsic<"promise", ISubmittableResult>
}

type DeliveryFee = {
    deliveryFeeInDot: bigint
    assetHubFeeInDot: bigint
    totalFeeInDot: bigint
}

export async function createTransfer(
    parachain: ApiPromise,
    registry: AssetRegistry,
    sourceAccount: string,
    beneficiaryAccount: string,
    tokenAddress: string,
    amount: bigint,
    fee: DeliveryFee,
): Promise<Transfer> {
    const { ethChainId, assetHubParaId } = registry

    let sourceAccountHex = sourceAccount
    if (!isHex(sourceAccountHex)) {
        sourceAccountHex = u8aToHex(decodeAddress(sourceAccount))
    }

    const sourceParaId = await getParachainId(parachain)
    const { tokenErcMetadata, sourceParachain, ahAssetMetadata, sourceAssetMetadata } = resolveInputs(registry, tokenAddress, sourceParaId)

    let messageId: string | undefined
    let tx: SubmittableExtrinsic<"promise", ISubmittableResult>;
    if (sourceParaId === assetHubParaId) {
        tx = createERC20AssetHubTx(parachain, ethChainId, tokenAddress, beneficiaryAccount, amount)
    } else {
        const [accountNextId] = await Promise.all([
            parachain.rpc.system.accountNextIndex(sourceAccountHex),
        ]);
        const entropy = new Uint8Array([
            ...stringToU8a(sourceParaId.toString()),
            ...hexToU8a(sourceAccountHex),
            ...accountNextId.toU8a(),
            ...hexToU8a(tokenAddress),
            ...stringToU8a(beneficiaryAccount),
            ...stringToU8a(amount.toString()),
        ]);
        messageId = blake2AsHex(entropy);
        tx = createERC20SourceParachainTx(parachain, ethChainId, assetHubParaId, sourceAccountHex, tokenAddress, beneficiaryAccount, amount, fee.totalFeeInDot, messageId)
    }

    return {
        input: {
            registry,
            sourceAccount,
            beneficiaryAccount,
            tokenAddress,
            amount,
            fee,
        },
        computed: {
            sourceParaId,
            sourceAccountHex,
            tokenErcMetadata,
            sourceParachain,
            ahAssetMetadata,
            sourceAssetMetadata,
            messageId,
        },
        tx
    }
}

export async function getDeliveryFee(assetHub: ApiPromise, registry: AssetRegistry, defaultFee?: bigint): Promise<DeliveryFee> {
    // Fees stored in 0x5fbc5c7ba58845ad1f1a9a7c5bc12fad
    const feeStorageKey = xxhashAsHex(":BridgeHubEthereumBaseFee:", 128, true)
    const feeStorageItem = await assetHub.rpc.state.getStorage(feeStorageKey)
    let leFee = new BN((feeStorageItem as Codec).toHex().replace("0x", ""), "hex", "le")

    let deliveryFeeInDot = 0n
    if (leFee.eqn(0)) {
        console.warn("Asset Hub onchain BridgeHubEthereumBaseFee not set. Using default fee.")
        deliveryFeeInDot = defaultFee ?? 2_750_872_500_000n
    }
    else {
        deliveryFeeInDot = BigInt(leFee.toString())
    }

    const assetHubFeeInDot = await calculateDestinationFee(assetHub,
        buildResultXcmAssetHubERC20TransferFromParachain(
            assetHub.registry,
            registry.ethChainId,
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            340282366920938463463374607431768211455n,
            340282366920938463463374607431768211455n,
        )
    )

    return {
        deliveryFeeInDot,
        assetHubFeeInDot,
        totalFeeInDot: deliveryFeeInDot + assetHubFeeInDot
    }
}

interface Connections {
    sourceParachain: ApiPromise
    assetHub: ApiPromise
    gateway: IGateway
    bridgeHub: ApiPromise
}

export enum ValidationKind {
    Warning, Error
}

export enum ValidationReason {
    BridgeStatusNotOperational,
    InsufficientTokenBalance,
    FeeEstimationError,
    InsufficientDotFee,
    InsufficientNativeFee,
    DryRunApiNotAvailable,
    DryRunFailed,
}

export type ValidationLog = {
    kind: ValidationKind
    reason: ValidationReason
    message: string
}

export async function validateTransfer(connections: Connections, transfer: Transfer) {
    const { sourceParachain, gateway, bridgeHub, assetHub } = connections
    const { registry, fee, tokenAddress, amount, beneficiaryAccount } = transfer.input
    const { sourceAccountHex, sourceParaId, sourceParachain: source } = transfer.computed
    const { tx } = transfer

    const logs: ValidationLog[] = []

    const [nativeBalance, dotBalance, tokenBalance] = await Promise.all([
        getNativeBalance(sourceParachain, sourceAccountHex),
        getDotBalance(sourceParachain, source.info.specName, sourceAccountHex),
        getTokenBalance(sourceParachain, source.info.specName, sourceAccountHex, registry.ethChainId, tokenAddress)
    ])

    if (amount > tokenBalance) {
        logs.push({ kind: ValidationKind.Error, reason: ValidationReason.InsufficientTokenBalance, message: 'Insufficient token balance to submit transaction.' })
    }

    let sourceDryRunError;
    let assetHubDryRunError;
    if (source.features.hasDryRunApi) {
        // do the dry run, get the forwarded xcm and dry run that
        const dryRunSource = await dryRunOnSourceParachain(sourceParachain, transfer)
        if (!dryRunSource.success) {
            logs.push({ kind: ValidationKind.Error, reason: ValidationReason.DryRunFailed, message: 'Dry run call on source failed.' })
            sourceDryRunError = dryRunSource.error
        }

        if (dryRunSource.success && sourceParaId !== registry.assetHubParaId) {
            if (!dryRunSource.xcm) {
                logs.push({ kind: ValidationKind.Error, reason: ValidationReason.DryRunFailed, message: 'Dry run call did not provide a forwared xcm.' })
            } else {
                const dryRunResultAssetHub = await dryRunAssetHub(assetHub, sourceParaId, dryRunSource.xcm)
                if (!dryRunResultAssetHub.success) {
                    logs.push({ kind: ValidationKind.Error, reason: ValidationReason.DryRunFailed, message: 'Dry run failed on Asset Hub.' })
                    assetHubDryRunError = dryRunResultAssetHub.errorMessage
                }
            }
        }
    } else {
        logs.push({ kind: ValidationKind.Warning, reason: ValidationReason.DryRunApiNotAvailable, message: 'Source parachain can not dry run call. Cannot verify success.' })
        const dryRunResultAssetHub = await dryRunAssetHub(assetHub, sourceParaId, buildResultXcmAssetHubERC20TransferFromParachain(
            sourceParachain.registry,
            registry.ethChainId,
            sourceAccountHex,
            beneficiaryAccount,
            tokenAddress,
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            amount,
            fee.totalFeeInDot,
        ))
        if (!dryRunResultAssetHub.success) {
            logs.push({ kind: ValidationKind.Error, reason: ValidationReason.DryRunFailed, message: 'Dry run failed on Asset Hub.' })
            assetHubDryRunError = dryRunResultAssetHub.errorMessage
        }
    }

    const paymentInfo = await tx.paymentInfo(sourceAccountHex)
    const sourceExecutionFee = paymentInfo['partialFee'].toBigInt()

    if (sourceParaId === registry.assetHubParaId) {
        if ((sourceExecutionFee + fee.totalFeeInDot) > (dotBalance)) {
            logs.push({ kind: ValidationKind.Error, reason: ValidationReason.InsufficientDotFee, message: 'Insufficient DOT balance to submit transaction on the source parachain.' })
        }
    }
    else {
        if (fee.totalFeeInDot > dotBalance) {
            logs.push({ kind: ValidationKind.Error, reason: ValidationReason.InsufficientDotFee, message: 'Insufficient DOT balance to submit transaction on the source parachain.' })
        }
        if (sourceExecutionFee > nativeBalance) {
            logs.push({ kind: ValidationKind.Error, reason: ValidationReason.InsufficientNativeFee, message: 'Insufficient native balance to submit transaction on the source parachain.' })
        }
    }
    const bridgeStatus = await getOperatingStatus({ gateway, bridgeHub })
    if (bridgeStatus.toEthereum.outbound !== "Normal") {
        logs.push({ kind: ValidationKind.Error, reason: ValidationReason.BridgeStatusNotOperational, message: 'Bridge operations have been paused by onchain governance.' })
    }

    return {
        logs,
        data: {
            bridgeStatus,
            nativeBalance,
            dotBalance,
            sourceExecutionFee,
            tokenBalance,
            sourceDryRunError,
            assetHubDryRunError
        },
        transfer,
    }
}

export type MessageReceipt = {
    blockNumber: number
    blockHash: string
    txIndex: number
    txHash: string
    success: boolean
    events: EventRecord[]
    dispatchError?: any
    messageId?: string
}

export async function signAndSend(parachain: ApiPromise, transfer: Transfer, account: AddressOrPair, options: Partial<SignerOptions>): Promise<MessageReceipt> {
    const result = await new Promise<MessageReceipt>((resolve, reject) => {
        try {
            transfer.tx.signAndSend(account, options, (c) => {
                if (c.isError) {
                    console.error(c)
                    reject(c.internalError || c.dispatchError || c)
                }
                if (c.isInBlock) {
                    const result = {
                        txHash: u8aToHex(c.txHash),
                        txIndex: c.txIndex || 0,
                        blockNumber: Number((c as any).blockNumber),
                        blockHash: "",
                        events: c.events,
                    }
                    for (const e of c.events) {
                        if (parachain.events.system.ExtrinsicFailed.is(e.event)) {
                            resolve({
                                ...result,
                                success: false,
                                dispatchError: (e.event.data.toHuman(true) as any)
                                    ?.dispatchError,
                            })
                        }

                        if (parachain.events.polkadotXcm.Sent.is(e.event)) {
                            resolve({
                                ...result,
                                success: true,
                                messageId: (e.event.data.toPrimitive() as any)[3],
                            })
                        }
                    }
                    resolve({
                        ...result,
                        success: false,
                    })
                }
            })
        } catch (e) {
            console.error(e)
            reject(e)
        }
    })

    result.blockHash = u8aToHex(await parachain.rpc.chain.getBlockHash(result.blockNumber))
    result.messageId = transfer.computed.messageId ?? result.messageId

    return result
}

function resolveInputs(registry: AssetRegistry, tokenAddress: string, sourceParaId: number) {
    const tokenErcMetadata = registry.ethereumChains[registry.ethChainId.toString()].assets[tokenAddress.toLowerCase()];
    if (!tokenErcMetadata) {
        throw Error(`No token ${tokenAddress} registered on ethereum chain ${registry.ethChainId}.`)
    }
    const sourceParachain = registry.parachains[sourceParaId.toString()]
    if (!sourceParachain) {
        throw Error(`Could not find ${sourceParaId} in the asset registry.`)
    }
    const ahAssetMetadata = registry.parachains[registry.assetHubParaId].assets[tokenAddress.toLowerCase()]
    if (!ahAssetMetadata) {
        throw Error(`Token ${tokenAddress} not registered on asset hub.`)
    }

    const sourceAssetMetadata = sourceParachain.assets[tokenAddress.toLowerCase()]
    if (!sourceAssetMetadata) {
        throw Error(`Token ${tokenAddress} not registered on source parachain ${sourceParaId}.`)
    }

    return { tokenErcMetadata, sourceParachain, ahAssetMetadata, sourceAssetMetadata }
}

function createERC20AssetHubTx(
    parachain: ApiPromise,
    ethChainId: number,
    tokenAddress: string,
    beneficiaryAccount: string,
    amount: bigint
): SubmittableExtrinsic<"promise", ISubmittableResult> {
    const assetLocation = erc20Location(ethChainId, tokenAddress)
    const assets = {
        v4: [
            {
                id: assetLocation,
                fun: { Fungible: amount },
            }
        ]
    }
    const destination = { v4: bridgeLocation(ethChainId) }
    const beneficiaryLocation = {
        v4: {
            parents: 0,
            interior: { x1: [{ accountKey20: { key: beneficiaryAccount } }] },
        }
    }
    return parachain.tx.polkadotXcm.transferAssets(destination, beneficiaryLocation, assets, 0, "Unlimited")
}

function createERC20SourceParachainTx(
    parachain: ApiPromise,
    ethChainId: number,
    assetHubParaId: number,
    sourceAccount: string,
    tokenAddress: string,
    beneficiaryAccount: string,
    amount: bigint,
    totalFeeInDot: bigint,
    messageId: string
): SubmittableExtrinsic<"promise", ISubmittableResult> {
    const assets = {
        v4: [
            {
                id: DOT_LOCATION,
                fun: { Fungible: totalFeeInDot },
            },
            {
                id: erc20Location(ethChainId, tokenAddress),
                fun: { Fungible: amount },
            },
        ]
    }
    const destination = { v4: parahchainLocation(assetHubParaId) }

    const feeAsset = {
        v4: DOT_LOCATION
    }
    const customXcm = buildAssetHubERC20TransferFromParachain(parachain.registry, ethChainId, sourceAccount, beneficiaryAccount, tokenAddress, messageId)
    return parachain.tx.polkadotXcm.transferAssetsUsingTypeAndThen(destination, assets, "DestinationReserve", feeAsset, "DestinationReserve", customXcm, "Unlimited")
}

async function dryRunOnSourceParachain(source: ApiPromise, transfer: Transfer) {
    const { sourceAccount } = transfer.input
    const origin = { system: { signed: sourceAccount } }
    const result = (await source.call.dryRunApi.dryRunCall<Result<CallDryRunEffects, XcmDryRunApiError>>(
        origin,
        transfer.tx,
    ))
    const success = result.isOk && result.asOk.executionResult.isOk && result.asOk.forwardedXcms.length === 1
    return {
        success,
        error: result.isOk && result.asOk.executionResult.isErr ? result.asOk.executionResult.asErr.toJSON() : undefined,
        destination: success ? result.asOk.forwardedXcms[0][0] : undefined,
        xcm: success ? result.asOk.forwardedXcms[0][1][0] : undefined,
    }
}

async function dryRunAssetHub(assetHub: ApiPromise, parachainId: number, xcm: any) {
    const sourceParachain = { v4: { parents: 1, interior: { x1: [{ parachain: parachainId }] } } }
    const result = (await assetHub.call.dryRunApi.dryRunXcm(
        sourceParachain,
        xcm
    ))

    const resultPrimitive = result.toPrimitive() as any
    const resultHuman = result.toHuman() as any

    return {
        success: resultPrimitive.ok?.executionResult?.complete !== undefined,
        errorMessage: resultHuman.Ok.executionResult.Incomplete?.error,
    }
}

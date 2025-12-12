import { ApiPromise } from "@polkadot/api"
import { AddressOrPair, SignerOptions, SubmittableExtrinsic } from "@polkadot/api/types"
import { Codec, ISubmittableResult } from "@polkadot/types/types"
import { BN, hexToU8a, isHex, stringToU8a, u8aToHex } from "@polkadot/util"
import { blake2AsHex, decodeAddress, xxhashAsHex } from "@polkadot/util-crypto"
import {
    erc20Location,
    kusamaAssetHubLocation,
    buildAssetHubERC20TransferToKusama,
    polkadotAssetHubLocation,
    isKSMOnOtherConsensusSystem,
    isRelaychainLocation,
    NATIVE_TOKEN_LOCATION,
    dotLocationOnKusamaAssetHub,
    ksmLocationOnPolkadotAssetHub,
    DOT_LOCATION,
    matchesConsensusSystem,
} from "./xcmBuilder"
import {
    buildKusamaToPolkadotDestAssetHubXCM,
    buildPolkadotToKusamaDestAssetHubXCM,
    buildTransferKusamaToPolkadotExportXCM,
    buildTransferPolkadotToKusamaExportXCM,
} from "./xcmBuilderKusama"
import { getAssetHubConversionPalletSwap, validateAccount } from "./assets_v2"
import { Asset, AssetRegistry, Parachain, AssetMap } from "@snowbridge/base-types"
import {
    CallDryRunEffects,
    EventRecord,
    XcmDryRunApiError,
    XcmDryRunEffects,
} from "@polkadot/types/interfaces"
import { Result } from "@polkadot/types"
import { beneficiaryMultiAddress, padFeeByPercentage } from "./utils"
import { paraImplementation } from "./parachains"
import { ParachainBase } from "./parachains/parachainBase"

export type Transfer = {
    input: {
        registry: AssetRegistry
        sourceAccount: string
        beneficiaryAccount: any
        tokenAddress: string
        amount: bigint
        fee: DeliveryFee
    }
    computed: {
        sourceParaId: number
        beneficiaryAddressHex: string
        sourceAccountHex: string
        sourceAssetMetadata: Asset
        destAssetMetadata: Asset
        sourceParachain: Parachain
        messageId?: string
    }
    tx: SubmittableExtrinsic<"promise", ISubmittableResult>
}

export type DeliveryFee = {
    xcmBridgeFee: bigint
    bridgeHubDeliveryFee: bigint
    destinationFee: bigint
    totalFeeInNative: bigint
}

export enum Direction {
    ToKusama,
    ToPolkadot,
}

const KUSAMA_BASE_FEE = 10_602_492_378n // 0.0106KSM
const KUSAMA_FEE_PER_BYTE = 1000000n // 0.000001 KSM
const POLKADOT_BASE_FEE = 333_794_429n // 0.033 DOT
const POLKADOT_FEE_PER_BYTE = 16666n // 0.0000016666 DOT

function resolveInputs(
    registry: AssetRegistry,
    tokenAddress: string,
    sourceParaId: number,
    destParaId: number,
) {
    const sourceParachain = registry.parachains[sourceParaId.toString()]
    if (!sourceParachain) {
        throw Error(`Could not find ${sourceParaId} in the asset registry.`)
    }
    const destParachain = registry.kusama?.parachains[destParaId.toString()]
    if (!destParachain) {
        throw Error(`Could not find ${destParaId} in the asset registry.`)
    }

    const sourceAssetMetadata = registry.parachains[sourceParaId].assets[tokenAddress.toLowerCase()]
    if (!sourceAssetMetadata) {
        throw Error(`Token ${tokenAddress} not registered on source asset hub.`)
    }
    const destAssetMetadata =
        registry.kusama?.parachains[destParaId].assets[tokenAddress.toLowerCase()]
    if (!destAssetMetadata) {
        throw Error(`Token ${tokenAddress} not registered on destination asset hub.`)
    }

    return { sourceAssetMetadata, destAssetMetadata, sourceParachain }
}

export async function getDeliveryFee(
    sourceAssetHub: ApiPromise,
    destAssetHub: ApiPromise,
    direction: Direction,
    registry: AssetRegistry,
    tokenAddress: string,
): Promise<DeliveryFee> {
    // Get base bridge fee
    // https://github.com/polkadot-fellows/runtimes/blob/main/system-parachains/asset-hubs/asset-hub-polkadot/src/xcm_config.rs#L546
    let baseFeeInStorage = await getStorageItem(sourceAssetHub, ":XcmBridgeHubRouterBaseFee:")
    let xcmBridgeBaseFee: bigint
    if (baseFeeInStorage.eqn(0)) {
        console.warn("Asset Hub onchain XcmBridgeHubRouterBaseFee not set. Using default fee.")
        if (direction == Direction.ToPolkadot) {
            xcmBridgeBaseFee = KUSAMA_BASE_FEE
        } else {
            xcmBridgeBaseFee = POLKADOT_BASE_FEE
        }
    } else {
        xcmBridgeBaseFee = BigInt(baseFeeInStorage.toString())
    }

    // Get fee per byte
    // https://github.com/polkadot-fellows/runtimes/blob/main/system-parachains/asset-hubs/asset-hub-polkadot/src/xcm_config.rs#L551
    let feePerByteInStorage = await getStorageItem(sourceAssetHub, ":XcmBridgeHubRouterByteFee:")
    let xcmFeePerByte: bigint
    if (feePerByteInStorage.eqn(0)) {
        console.warn(
            "Asset Hub onchain XcmBridgeHubRouterByteFee not set. Using default fee per byte.",
        )
        if (direction == Direction.ToPolkadot) {
            xcmFeePerByte = KUSAMA_FEE_PER_BYTE
        } else {
            xcmFeePerByte = POLKADOT_FEE_PER_BYTE
        }
    } else {
        xcmFeePerByte = BigInt(baseFeeInStorage.toString())
    }

    let tokenLocation = getTokenLocation(registry, direction, tokenAddress)

    if (!registry.kusama) {
        throw Error("Kusama config is not set")
    }

    let forwardedXcm
    // Message from dest AH to BH
    if (direction == Direction.ToPolkadot) {
        forwardedXcm = buildTransferKusamaToPolkadotExportXCM(
            sourceAssetHub.registry,
            tokenLocation,
            xcmBridgeBaseFee,
            xcmBridgeBaseFee,
            registry.kusama?.assetHubParaId,
            registry.assetHubParaId,
            100000000000n,
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000000000000000000000000000",
        )
    } else {
        forwardedXcm = buildTransferPolkadotToKusamaExportXCM(
            sourceAssetHub.registry,
            tokenLocation,
            xcmBridgeBaseFee,
            xcmBridgeBaseFee,
            registry.assetHubParaId,
            registry.kusama?.assetHubParaId,
            100000000000n,
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000000000000000000000000000",
        )
    }

    let bytes = forwardedXcm.toU8a().length
    console.log("forwardedXcm length:", bytes)
    let xcmBytesFee = BigInt(bytes) * xcmFeePerByte
    let totalXcmBridgeFee = xcmBridgeBaseFee + xcmBytesFee
    console.info("xcmBridgeBaseFee:", xcmBridgeBaseFee)
    console.info("xcmBytesFee:", xcmBytesFee)

    // Message from dest AH to BH
    let destXcm
    if (direction == Direction.ToPolkadot) {
        destXcm = buildKusamaToPolkadotDestAssetHubXCM(
            destAssetHub.registry,
            totalXcmBridgeFee,
            registry.assetHubParaId,
            tokenLocation,
            100000000000n,
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000000000000000000000000000",
        )
    } else {
        destXcm = buildPolkadotToKusamaDestAssetHubXCM(
            destAssetHub.registry,
            totalXcmBridgeFee,
            registry.assetHubParaId,
            tokenLocation,
            100000000000n,
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000000000000000000000000000",
        )
    }
    const destAssetHubImpl = await paraImplementation(destAssetHub)
    let destinationFeeInDestNative = await destAssetHubImpl.calculateXcmFee(destXcm, DOT_LOCATION)

    const sourceAssetHubImpl = await paraImplementation(sourceAssetHub)
    let bridgeHubDeliveryFee = await sourceAssetHubImpl.calculateDeliveryFeeInDOT(
        registry.bridgeHubParaId,
        forwardedXcm,
    )

    let feeAssetOnDest
    let minBalanceFeeDest: bigint
    if (direction == Direction.ToPolkadot) {
        feeAssetOnDest = ksmLocationOnPolkadotAssetHub
        minBalanceFeeDest = getDestFeeAssetMinimumBalance(
            registry.parachains[registry.assetHubParaId].assets,
            "kusama",
        )
    } else {
        feeAssetOnDest = dotLocationOnKusamaAssetHub
        minBalanceFeeDest = getDestFeeAssetMinimumBalance(
            registry.kusama.parachains[registry.kusama.assetHubParaId].assets,
            "polkadot",
        )
    }
    let destinationFee = await getAssetHubConversionPalletSwap(
        destAssetHub,
        feeAssetOnDest,
        NATIVE_TOKEN_LOCATION,
        destinationFeeInDestNative,
    )
    // pad destination XCM fee
    destinationFee = padFeeByPercentage(destinationFee, 33n)

    // add minimum balance to the dest fee, to avoid not being able to deposit leftover fees
    destinationFee = destinationFee + BigInt(minBalanceFeeDest)

    // pad destination XCM fee
    totalXcmBridgeFee = padFeeByPercentage(totalXcmBridgeFee, 33n)

    let totalFee = totalXcmBridgeFee + bridgeHubDeliveryFee + destinationFee

    return {
        xcmBridgeFee: totalXcmBridgeFee,
        destinationFee,
        bridgeHubDeliveryFee,
        totalFeeInNative: totalFee,
    }
}

export async function createTransfer(
    parachain: ApiPromise,
    direction: Direction,
    registry: AssetRegistry,
    sourceAccount: string,
    beneficiaryAccount: string,
    tokenAddress: string,
    amount: bigint,
    fee: DeliveryFee,
): Promise<Transfer> {
    const { assetHubParaId } = registry
    const destParaId = registry.kusama?.assetHubParaId
    let sourceParaId = assetHubParaId

    let sourceAccountHex = sourceAccount
    if (!isHex(sourceAccountHex)) {
        sourceAccountHex = u8aToHex(decodeAddress(sourceAccount))
    }

    if (!destParaId) {
        throw Error("Kusama destination para ID is not set")
    }

    let { hexAddress: beneficiaryAddressHex } = beneficiaryMultiAddress(beneficiaryAccount)

    const { sourceAssetMetadata, destAssetMetadata, sourceParachain } = resolveInputs(
        registry,
        tokenAddress,
        sourceParaId,
        destParaId,
    )
    let messageId = await buildMessageId(
        parachain,
        sourceParaId,
        sourceAccountHex,
        tokenAddress,
        beneficiaryAccount,
        amount,
    )

    let tokenLocationOnSource = getTokenLocation(registry, direction, tokenAddress)
    let tx
    if (direction == Direction.ToPolkadot) {
        tx = createERC20ToPolkadotTx(
            sourceParaId,
            parachain,
            tokenLocationOnSource,
            beneficiaryAddressHex,
            amount,
            fee.destinationFee,
            messageId,
        )
    } else {
        tx = createERC20ToKusamaTx(
            destParaId,
            parachain,
            tokenLocationOnSource,
            beneficiaryAddressHex,
            amount,
            fee.destinationFee,
            messageId,
        )
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
            sourceParachain,
            sourceAssetMetadata,
            sourceAccountHex,
            destAssetMetadata,
            messageId,
            beneficiaryAddressHex,
        },
        tx,
    }
}

export enum ValidationKind {
    Warning,
    Error,
}

export enum ValidationReason {
    InsufficientTokenBalance,
    InsufficientFee,
    DryRunFailed,
    MaxConsumersReached,
    AccountDoesNotExist,
}

export type ValidationLog = {
    kind: ValidationKind
    reason: ValidationReason
    message: string
}

export type ValidationResult = {
    logs: ValidationLog[]
    success: boolean
    data: {
        nativeBalance: bigint
        sourceExecutionFee: bigint
        tokenBalance: bigint
        assetHubDryRunError: any
    }
    transfer: Transfer
}

export async function validateTransfer(
    connections: {
        sourceAssetHub: ApiPromise
        destAssetHub: ApiPromise
    },
    direction: Direction,
    transfer: Transfer,
): Promise<ValidationResult> {
    let sourceAssetHub = connections.sourceAssetHub
    let destAssetHub = connections.destAssetHub

    const { registry, fee, tokenAddress, amount } = transfer.input
    const {
        sourceAccountHex,
        sourceParachain: source,
        beneficiaryAddressHex,
        destAssetMetadata,
    } = transfer.computed
    const { tx } = transfer

    let tokenLocation = getTokenLocation(registry, direction, tokenAddress)

    const sourceAssetHubImpl = await paraImplementation(sourceAssetHub)
    let nativeBalance = await sourceAssetHubImpl.getNativeBalance(sourceAccountHex)

    let tokenAsset = getTransferAsset(direction, tokenAddress, transfer.input.registry)

    let tokenBalance: bigint
    if (isRelaychainLocation(tokenLocation)) {
        tokenBalance = nativeBalance
    } else {
        tokenBalance = await sourceAssetHubImpl.getTokenBalance(
            sourceAccountHex,
            registry.ethChainId,
            tokenAddress,
            tokenAsset,
        )
    }

    const logs: ValidationLog[] = []

    if (amount > tokenBalance) {
        logs.push({
            kind: ValidationKind.Error,
            reason: ValidationReason.InsufficientTokenBalance,
            message: "Insufficient token balance to submit transaction.",
        })
    }

    let assetHubDryRunError

    const dryRunSource = await dryRunSourceAssetHub(
        sourceAssetHub,
        registry.assetHubParaId,
        registry.bridgeHubParaId,
        transfer.tx,
        sourceAccountHex,
    )
    if (!dryRunSource.success) {
        logs.push({
            kind: ValidationKind.Error,
            reason: ValidationReason.DryRunFailed,
            message: "Dry run call on source failed.",
        })
        assetHubDryRunError = dryRunSource.error
    }

    const paymentInfo = await tx.paymentInfo(sourceAccountHex)
    const sourceExecutionFee = paymentInfo["partialFee"].toBigInt()

    if (sourceExecutionFee + fee.totalFeeInNative > nativeBalance) {
        logs.push({
            kind: ValidationKind.Error,
            reason: ValidationReason.InsufficientFee,
            message:
                "Insufficient " +
                nativeFeeAsset(direction) +
                " balance to submit transaction on the source parachain.",
        })
    }

    let destAssetHubXCM: any
    if (direction == Direction.ToPolkadot) {
        destAssetHubXCM = buildKusamaToPolkadotDestAssetHubXCM(
            destAssetHub.registry,
            fee.destinationFee,
            registry.assetHubParaId,
            tokenLocation,
            transfer.input.amount,
            transfer.computed.beneficiaryAddressHex,
            "0x0000000000000000000000000000000000000000000000000000000000000000",
        )
    } else {
        destAssetHubXCM = buildPolkadotToKusamaDestAssetHubXCM(
            destAssetHub.registry,
            fee.destinationFee,
            registry.assetHubParaId,
            tokenLocation,
            transfer.input.amount,
            transfer.computed.beneficiaryAddressHex,
            "0x0000000000000000000000000000000000000000000000000000000000000000",
        )
    }

    const dryRunAssetHubDest = await dryRunDestAssetHub(
        destAssetHub,
        registry.bridgeHubParaId,
        destAssetHubXCM,
    )
    if (!dryRunAssetHubDest.success) {
        logs.push({
            kind: ValidationKind.Error,
            reason: ValidationReason.DryRunFailed,
            message: "Dry run call on destination AH failed: " + dryRunAssetHubDest.errorMessage,
        })
        assetHubDryRunError = dryRunAssetHubDest.errorMessage

        // Only run the account validation if the dry run failed.
        const destAssetHubImpl = await paraImplementation(destAssetHub)
        const { accountMaxConsumers, accountExists } = await validateAccount(
            destAssetHubImpl,
            beneficiaryAddressHex,
            registry.ethChainId,
            tokenAddress,
            destAssetMetadata,
        )
        if (accountMaxConsumers) {
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.MaxConsumersReached,
                message:
                    "Beneficiary account has reached the max consumer limit on the destination chain.",
            })
        }
        if (!accountExists) {
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.AccountDoesNotExist,
                message: "Beneficiary account does not exist on the destination chain.",
            })
        }
    }

    const success = logs.find((l) => l.kind === ValidationKind.Error) === undefined

    return {
        logs,
        success,
        data: {
            nativeBalance,
            sourceExecutionFee,
            tokenBalance,
            assetHubDryRunError,
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

export async function signAndSend(
    parachain: ApiPromise,
    transfer: Transfer,
    account: AddressOrPair,
    options: Partial<SignerOptions>,
): Promise<MessageReceipt> {
    const result = await new Promise<MessageReceipt>((resolve, reject) => {
        try {
            transfer.tx.signAndSend(account, options, (c) => {
                if (c.isError) {
                    console.error(c)
                    reject(c.internalError || c.dispatchError || c)
                }
                // We have to check for finalization here because re-orgs will produce a different messageId on Asset Hub.
                // TODO: Change back to isInBlock when we switch to pallet-xcm.execute for Asset Hub and we can generate the messageId offchain.
                if (c.isFinalized) {
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
                                dispatchError: (e.event.data.toHuman(true) as any)?.dispatchError,
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

export function createERC20ToKusamaTx(
    destParaId: number,
    parachain: ApiPromise,
    tokenLocation: any,
    beneficiaryAccount: string,
    amount: bigint,
    destFeeInSourceNative: bigint,
    topic: string,
): SubmittableExtrinsic<"promise", ISubmittableResult> {
    let assets: any
    // is DOT
    if (isRelaychainLocation(tokenLocation)) {
        assets = {
            v4: [
                {
                    id: NATIVE_TOKEN_LOCATION,
                    fun: { Fungible: destFeeInSourceNative + amount },
                },
            ],
        }
    } else {
        assets = {
            v4: [
                {
                    id: NATIVE_TOKEN_LOCATION,
                    fun: { Fungible: destFeeInSourceNative },
                },
                {
                    id: tokenLocation,
                    fun: { Fungible: amount },
                },
            ],
        }
    }
    let reserveTypeAsset = "LocalReserve"
    if (isKSM(Direction.ToKusama, tokenLocation)) {
        reserveTypeAsset = "DestinationReserve"
    }

    const destination = { v4: kusamaAssetHubLocation(destParaId) }

    const feeAsset = {
        v4: NATIVE_TOKEN_LOCATION,
    }
    const customXcm = buildAssetHubERC20TransferToKusama(
        parachain.registry,
        beneficiaryAccount,
        topic,
    )
    return parachain.tx.polkadotXcm.transferAssetsUsingTypeAndThen(
        destination,
        assets,
        reserveTypeAsset,
        feeAsset,
        "LocalReserve",
        customXcm,
        "Unlimited",
    )
}

export function createERC20ToPolkadotTx(
    destParaId: number,
    parachain: ApiPromise,
    tokenLocation: any,
    beneficiaryAccount: string,
    amount: bigint,
    destFeeInSourceNative: bigint,
    topic: string,
): SubmittableExtrinsic<"promise", ISubmittableResult> {
    let assets: any
    let reserveTypeAsset = "DestinationReserve"
    // is KSM
    if (isRelaychainLocation(tokenLocation)) {
        assets = {
            v4: [
                {
                    id: NATIVE_TOKEN_LOCATION,
                    fun: { Fungible: destFeeInSourceNative + amount },
                },
            ],
        }
        reserveTypeAsset = "LocalReserve"
    } else {
        assets = {
            v4: [
                {
                    id: NATIVE_TOKEN_LOCATION,
                    fun: { Fungible: destFeeInSourceNative },
                },
                {
                    id: tokenLocation,
                    fun: { Fungible: amount },
                },
            ],
        }
    }

    const destination = { v4: polkadotAssetHubLocation(destParaId) }

    const feeAsset = {
        v4: NATIVE_TOKEN_LOCATION,
    }
    const customXcm = buildAssetHubERC20TransferToKusama(
        parachain.registry,
        beneficiaryAccount,
        topic,
    )
    return parachain.tx.polkadotXcm.transferAssetsUsingTypeAndThen(
        destination,
        assets,
        reserveTypeAsset,
        feeAsset,
        "LocalReserve",
        customXcm,
        "Unlimited",
    )
}

export async function dryRunSourceAssetHub(
    source: ApiPromise,
    assetHubParaId: number,
    bridgeHubParaId: number,
    tx: SubmittableExtrinsic<"promise", ISubmittableResult>,
    sourceAccount: string,
) {
    const origin = { system: { signed: sourceAccount } }
    let result: Result<CallDryRunEffects, XcmDryRunApiError>
    result = await source.call.dryRunApi.dryRunCall<Result<CallDryRunEffects, XcmDryRunApiError>>(
        origin,
        tx,
        4,
    )

    let assetHubForwarded
    let bridgeHubForwarded
    const success = result.isOk && result.asOk.executionResult.isOk
    if (!success) {
        console.error(
            "Error during dry run on source parachain:",
            sourceAccount,
            tx.toHuman(),
            result.toHuman(true),
        )
        let err =
            result.isOk && result.asOk.executionResult.isErr
                ? result.asOk.executionResult.asErr.toJSON()
                : undefined
        console.error("Result:", err)
    } else {
        bridgeHubForwarded = result.asOk.forwardedXcms.find((x) => {
            return (
                x[0].isV4 &&
                x[0].asV4.parents.toNumber() === 1 &&
                x[0].asV4.interior.isX1 &&
                x[0].asV4.interior.asX1[0].isParachain &&
                x[0].asV4.interior.asX1[0].asParachain.toNumber() === bridgeHubParaId
            )
        })
        assetHubForwarded = result.asOk.forwardedXcms.find((x) => {
            return (
                x[0].isV4 &&
                x[0].asV4.parents.toNumber() === 1 &&
                x[0].asV4.interior.isX1 &&
                x[0].asV4.interior.asX1[0].isParachain &&
                x[0].asV4.interior.asX1[0].asParachain.toNumber() === assetHubParaId
            )
        })
    }
    return {
        success: success && (bridgeHubForwarded || assetHubForwarded),
        error:
            result.isOk && result.asOk.executionResult.isErr
                ? result.asOk.executionResult.asErr.toJSON()
                : undefined,
        assetHubForwarded,
        bridgeHubForwarded,
    }
}

async function dryRunDestAssetHub(assetHub: ApiPromise, parachainId: number, xcm: any) {
    const sourceParachain = { v4: { parents: 1, interior: { x1: [{ parachain: parachainId }] } } }
    const result = await assetHub.call.dryRunApi.dryRunXcm<
        Result<XcmDryRunEffects, XcmDryRunApiError>
    >(sourceParachain, xcm)

    const resultHuman = result.toHuman() as any

    const success = result.isOk && result.asOk.executionResult.isComplete
    if (!success) {
        console.error("Error during dry run on asset hub:", xcm.toHuman(), result.toHuman())
    }
    return {
        success: success,
        errorMessage: resultHuman.Ok.executionResult.Incomplete?.error,
    }
}

async function buildMessageId(
    parachain: ApiPromise,
    sourceParaId: number,
    sourceAccountHex: string,
    tokenAddress: string,
    beneficiaryAccount: string,
    amount: bigint,
) {
    const [accountNextId] = await Promise.all([
        parachain.rpc.system.accountNextIndex(sourceAccountHex),
    ])
    const entropy = new Uint8Array([
        ...stringToU8a(sourceParaId.toString()),
        ...hexToU8a(sourceAccountHex),
        ...accountNextId.toU8a(),
        ...hexToU8a(tokenAddress),
        ...stringToU8a(beneficiaryAccount),
        ...stringToU8a(amount.toString()),
    ])
    return blake2AsHex(entropy)
}

function getTokenLocation(registry: AssetRegistry, direction: Direction, tokenAddress: string) {
    let location
    if (direction == Direction.ToPolkadot) {
        location =
            registry.kusama?.parachains[registry.kusama?.assetHubParaId].assets[tokenAddress]
                .location
        if (!location) {
            location = erc20Location(registry.ethChainId, tokenAddress)
        }
    } else {
        location = registry.parachains[registry.assetHubParaId].assets[tokenAddress].location
        if (!location) {
            location = erc20Location(registry.ethChainId, tokenAddress)
        }
    }

    return location
}

function isKSM(direction: Direction, location: any) {
    if (direction == Direction.ToKusama) {
        return isKSMOnOtherConsensusSystem(location)
    } else {
        return isRelaychainLocation(location)
    }
}

function nativeFeeAsset(direction: Direction) {
    if (direction == Direction.ToPolkadot) {
        return "KSM"
    } else {
        return "DOT"
    }
}

function getTransferAsset(direction: Direction, tokenAddress: string, registry: AssetRegistry) {
    if (direction == Direction.ToPolkadot) {
        return registry.kusama?.parachains[registry.kusama?.assetHubParaId].assets[tokenAddress]
    } else {
        return registry.parachains[registry.assetHubParaId].assets[tokenAddress]
    }
}

async function getStorageItem(sourceAssetHub: ApiPromise, key: string) {
    const feeStorageKey = xxhashAsHex(key, 128, true)
    const feeStorageItem = await sourceAssetHub.rpc.state.getStorage(feeStorageKey)
    return new BN((feeStorageItem as Codec).toHex().replace("0x", ""), "hex", "le")
}

function getDestFeeAssetMinimumBalance(assetMap: AssetMap, network: string): bigint {
    const assets = Object.values(assetMap)
    for (const asset of assets) {
        if (asset.location === undefined) {
            continue
        }
        if (matchesConsensusSystem(asset.location, network)) {
            return asset.minimumBalance
        }
    }

    return 0n
}

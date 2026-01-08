import { ApiPromise } from "@polkadot/api"
import { AddressOrPair, SignerOptions, SubmittableExtrinsic } from "@polkadot/api/types"
import { ISubmittableResult } from "@polkadot/types/types"
import {
    erc20Location,
    DOT_LOCATION,
    parachainLocation,
    buildParachainERC20ReceivedXcmOnDestination,
    buildERC20ToAssetHubFromParachain,
    buildDepositAllAssetsWithTopic,
} from "./xcmBuilder"
import { Asset, AssetRegistry, Parachain } from "@snowbridge/base-types"
import { beneficiaryMultiAddress, padFeeByPercentage } from "./utils"
import { paraImplementation } from "./parachains"
import { Context } from "."
import { buildMessageId } from "./toEthereum_v2"
import { Result } from "@polkadot/types"
import {
    CallDryRunEffects,
    EventRecord,
    XcmDryRunApiError,
    XcmDryRunEffects,
} from "@polkadot/types/interfaces"
import { u8aToHex } from "@polkadot/util"

export type Transfer = {
    input: {
        registry: AssetRegistry
        sourceAccount: string
        beneficiaryAccount: any
        tokenAddress: string
        destinationParaId: number
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
        destParachain: Parachain
        messageId?: string
    }
    tx: SubmittableExtrinsic<"promise", ISubmittableResult>
}

export type DeliveryFee = {
    deliveryFee: bigint
    executionFee: bigint
    totalFeeInDot: bigint
}

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
    const destParachain = registry.parachains[destParaId.toString()]
    if (!destParachain) {
        throw Error(`Could not find ${destParaId} in the asset registry.`)
    }

    if (destParachain.parachainId === sourceParachain.parachainId) {
        throw Error("Source and destination are the same.")
    }

    const sourceAssetMetadata = sourceParachain.assets[tokenAddress.toLowerCase()]
    if (!sourceAssetMetadata) {
        throw Error(`Token ${tokenAddress} not registered on source asset hub.`)
    }
    const destAssetMetadata = destParachain.assets[tokenAddress.toLowerCase()]
    if (!destAssetMetadata) {
        throw Error(`Token ${tokenAddress} not registered on destination asset hub.`)
    }

    if (destAssetMetadata.location) {
        throw Error("PNA not supported")
    }

    return { sourceAssetMetadata, destAssetMetadata, sourceParachain, destParachain }
}

export async function getDeliveryFee(
    connections:
        | { context: Context; sourceParaId: number; destinationParaId: number }
        | { sourceParachain: ApiPromise; destParachain: ApiPromise },
    registry: AssetRegistry,
    tokenAddress: string,
    options?: {
        padPercentage?: bigint
    },
): Promise<DeliveryFee> {
    const { sourceParachain, destParachain } =
        "sourceParaId" in connections
            ? {
                  sourceParachain: await connections.context.parachain(connections.sourceParaId),
                  destParachain: await connections.context.parachain(connections.destinationParaId),
              }
            : connections

    const [source, destination] = await Promise.all([
        paraImplementation(sourceParachain),
        paraImplementation(destParachain),
    ])

    // PNA filtered out by resolve inputs.
    resolveInputs(registry, tokenAddress, source.parachainId, destination.parachainId)
    let xcm
    if (source.parachainId === registry.assetHubParaId) {
        xcm = buildParachainERC20ReceivedXcmOnDestination(
            sourceParachain.registry,
            registry.ethChainId,
            "0x0000000000000000000000000000000000000000",
            340282366920938463463374607431768211455n,
            340282366920938463463374607431768211455n,
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000000000000000000000000000",
        )
    } else {
        xcm = buildERC20ToAssetHubFromParachain(
            sourceParachain.registry,
            registry.ethChainId,
            "0x0000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            340282366920938463463374607431768211455n,
            340282366920938463463374607431768211455n,
            340282366920938463463374607431768211455n,
            DOT_LOCATION,
        )
    }

    const deliveryFee = padFeeByPercentage(
        await source.calculateDeliveryFeeInDOT(destination.parachainId, xcm),
        options?.padPercentage ?? 33n,
    )
    const executionFee = padFeeByPercentage(
        await destination.calculateXcmFee(xcm, DOT_LOCATION),
        options?.padPercentage ?? 33n,
    )

    return {
        deliveryFee,
        executionFee,
        totalFeeInDot: deliveryFee + executionFee,
    }
}

export async function createTransfer(
    connections: { context: Context; sourceParaId: number } | { sourceParachain: ApiPromise },
    registry: AssetRegistry,
    sourceAccount: string,
    beneficiaryAccount: string,
    destinationParaId: number,
    tokenAddress: string,
    amount: bigint,
    fee: DeliveryFee,
): Promise<Transfer> {
    const { sourceParachain } =
        "sourceParaId" in connections
            ? {
                  sourceParachain: await connections.context.parachain(connections.sourceParaId),
              }
            : connections

    const source = await paraImplementation(sourceParachain)

    let { hexAddress: beneficiaryAddressHex } = beneficiaryMultiAddress(beneficiaryAccount)
    let { hexAddress: sourceAccountHex } = beneficiaryMultiAddress(sourceAccount)

    const {
        sourceAssetMetadata,
        destAssetMetadata,
        sourceParachain: sourceParachainMeta,
        destParachain,
    } = resolveInputs(registry, tokenAddress, source.parachainId, destinationParaId)
    let messageId = await buildMessageId(
        sourceParachain,
        source.parachainId,
        sourceAccountHex,
        tokenAddress,
        beneficiaryAccount,
        amount,
    )

    const tx = createTx(
        sourceParachain,
        registry.ethChainId,
        destinationParaId,
        tokenAddress,
        beneficiaryAccount,
        messageId,
        amount,
        fee.totalFeeInDot,
        source.parachainId === registry.assetHubParaId ? "LocalReserve" : "DestinationReserve",
    )

    return {
        input: {
            registry,
            sourceAccount,
            beneficiaryAccount,
            destinationParaId,
            tokenAddress,
            amount,
            fee,
        },
        computed: {
            sourceParaId: source.parachainId,
            sourceParachain: sourceParachainMeta,
            destParachain,
            sourceAssetMetadata,
            destAssetMetadata,
            sourceAccountHex,
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
    DryRunFailed,
    MinimumAmountValidation,
    InsufficientFee,
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
        dryRunError: any
    }
    transfer: Transfer
}

export async function validateTransfer(
    connections:
        | { context: Context; sourceParaId: number; destinationParaId: number }
        | { sourceParachain: ApiPromise; destParachain: ApiPromise },
    transfer: Transfer,
): Promise<ValidationResult> {
    const { sourceParachain, destParachain } =
        "sourceParaId" in connections
            ? {
                  sourceParachain: await connections.context.parachain(connections.sourceParaId),
                  destParachain: await connections.context.parachain(connections.destinationParaId),
              }
            : connections

    const [source, destination] = await Promise.all([
        paraImplementation(sourceParachain),
        paraImplementation(destParachain),
    ])

    const { registry, fee, tokenAddress, amount, destinationParaId } = transfer.input
    const {
        sourceAccountHex,
        sourceAssetMetadata,
        destAssetMetadata,
        destParachain: destParachainMeta,
        beneficiaryAddressHex,
    } = transfer.computed
    const { tx } = transfer

    const nativeBalance = await source.getNativeBalance(sourceAccountHex)
    const tokenBalance = await source.getTokenBalance(
        sourceAccountHex,
        registry.ethChainId,
        tokenAddress,
        sourceAssetMetadata,
    )

    const logs: ValidationLog[] = []

    if (amount > tokenBalance) {
        logs.push({
            kind: ValidationKind.Error,
            reason: ValidationReason.InsufficientTokenBalance,
            message: "Insufficient token balance to submit transaction.",
        })
    }

    if (amount < destAssetMetadata.minimumBalance) {
        logs.push({
            kind: ValidationKind.Error,
            reason: ValidationReason.MinimumAmountValidation,
            message: "The amount transferred is less than the minimum amount.",
        })
    }

    let dryRunError

    const dryRunSource = await dryRunTx(
        sourceParachain,
        destinationParaId,
        transfer.tx,
        sourceAccountHex,
        source.parachainId === registry.assetHubParaId,
    )
    if (!dryRunSource.success) {
        logs.push({
            kind: ValidationKind.Error,
            reason: ValidationReason.DryRunFailed,
            message: "Dry run call on source failed.",
        })
        dryRunError = dryRunSource.error
    }

    const dryRunDestination = await dryRunXcm(
        destParachain,
        source.parachainId,
        dryRunSource.forwardedXcm,
    )
    if (!dryRunDestination.success) {
        logs.push({
            kind: ValidationKind.Error,
            reason: ValidationReason.DryRunFailed,
            message: "Dry run call on destination failed.",
        })
        dryRunError = dryRunDestination.errorMessage

        if (!destAssetMetadata.isSufficient) {
            const { accountMaxConsumers, accountExists } = await destination.validateAccount(
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
    }

    const paymentInfo = await tx.paymentInfo(sourceAccountHex)
    const sourceExecutionFee = paymentInfo["partialFee"].toBigInt()

    if (sourceExecutionFee > nativeBalance) {
        logs.push({
            kind: ValidationKind.Error,
            reason: ValidationReason.InsufficientFee,
            message:
                "Insufficient native asset balance to submit transaction on the source parachain.",
        })
    }

    const success = logs.find((l) => l.kind === ValidationKind.Error) === undefined

    return {
        logs,
        success,
        data: {
            nativeBalance,
            sourceExecutionFee,
            tokenBalance,
            dryRunError: dryRunError,
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
    connections: { context: Context; sourceParaId: number } | { sourceParachain: ApiPromise },
    transfer: Transfer,
    account: AddressOrPair,
    options: Partial<SignerOptions>,
): Promise<MessageReceipt> {
    const { sourceParachain } =
        "sourceParaId" in connections
            ? { sourceParachain: await connections.context.parachain(connections.sourceParaId) }
            : connections
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
                        if (sourceParachain.events.system.ExtrinsicFailed.is(e.event)) {
                            resolve({
                                ...result,
                                success: false,
                                dispatchError: (e.event.data.toHuman(true) as any)?.dispatchError,
                            })
                        }

                        if (sourceParachain.events.polkadotXcm.Sent.is(e.event)) {
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

    result.blockHash = u8aToHex(await sourceParachain.rpc.chain.getBlockHash(result.blockNumber))
    result.messageId = transfer.computed.messageId ?? result.messageId

    return result
}

function createTx(
    parachain: ApiPromise,
    ethChainId: number,
    destinationParachainId: number,
    tokenAddress: string,
    beneficiaryAccount: string,
    messageId: string,
    amount: bigint,
    feeAmount: bigint,
    reserveType: "LocalReserve" | "DestinationReserve",
): SubmittableExtrinsic<"promise", ISubmittableResult> {
    let assetLocation = erc20Location(ethChainId, tokenAddress)
    const assets = {
        v4: [
            {
                id: DOT_LOCATION,
                fun: { Fungible: feeAmount },
            },
            {
                id: assetLocation,
                fun: { Fungible: amount },
            },
        ],
    }
    const destination = { v4: parachainLocation(destinationParachainId) }

    const feeAsset = {
        v4: DOT_LOCATION,
    }

    const customXcm: any = buildDepositAllAssetsWithTopic(
        parachain.registry,
        beneficiaryAccount,
        messageId,
    )
    return parachain.tx.polkadotXcm.transferAssetsUsingTypeAndThen(
        destination,
        assets,
        reserveType,
        feeAsset,
        reserveType,
        customXcm,
        "Unlimited",
    )
}

export async function dryRunTx(
    source: ApiPromise,
    destParaId: number,
    tx: SubmittableExtrinsic<"promise", ISubmittableResult>,
    sourceAccount: string,
    useNewVersion: boolean,
) {
    const origin = { system: { signed: sourceAccount } }
    let result: Result<CallDryRunEffects, XcmDryRunApiError>

    if (useNewVersion) {
        result = await source.call.dryRunApi.dryRunCall<
            Result<CallDryRunEffects, XcmDryRunApiError>
        >(origin, tx, 4)
    } else {
        result = await source.call.dryRunApi.dryRunCall<
            Result<CallDryRunEffects, XcmDryRunApiError>
        >(origin, tx)
    }

    let forwardedXcm
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
        forwardedXcm = result.asOk.forwardedXcms
            .find(
                (x) =>
                    x[0].isV4 &&
                    x[0].asV4.parents.toNumber() === 1 &&
                    x[0].asV4.interior.isX1 &&
                    x[0].asV4.interior.asX1[0].isParachain &&
                    x[0].asV4.interior.asX1[0].asParachain.toNumber() === destParaId,
            )
            ?.toPrimitive() as any
    }
    return {
        success: success && forwardedXcm !== undefined,
        error:
            result.isOk && result.asOk.executionResult.isErr
                ? result.asOk.executionResult.asErr.toJSON()
                : undefined,
        forwardedXcm: forwardedXcm[1][0],
    }
}

async function dryRunXcm(source: ApiPromise, originParachainId: number, xcm: any) {
    const sourceParachain = {
        v4: { parents: 1, interior: { x1: [{ parachain: originParachainId }] } },
    }
    const result = await source.call.dryRunApi.dryRunXcm<
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

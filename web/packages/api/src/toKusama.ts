import { ApiPromise } from "@polkadot/api";
import { AddressOrPair, SignerOptions, SubmittableExtrinsic } from "@polkadot/api/types";
import { Codec, ISubmittableResult } from "@polkadot/types/types";
import { BN, hexToU8a, isHex, numberToHex, stringToU8a, u8aToHex } from "@polkadot/util";
import { blake2AsHex, decodeAddress, xxhashAsHex } from "@polkadot/util-crypto";
import {
    bridgeLocation,
    buildResultXcmAssetHubERC20TransferFromParachain,
    buildAssetHubERC20TransferFromParachain,
    DOT_LOCATION,
    erc20Location,
    parahchainLocation,
    buildParachainERC20ReceivedXcmOnDestination,
    kusamaAssetHubLocation, buildAssetHubERC20TransferToKusama
} from "./xcmBuilder";
import { Asset, AssetRegistry, calculateDeliveryFee, calculateDestinationFee, ERC20Metadata, EthereumChain, getDotBalance, getNativeBalance, getParachainId, getTokenBalance, padFeeByPercentage, Parachain } from "./assets_v2";
import { getOperatingStatus, OperationStatus } from "./status";
import { IGatewayV1 as IGateway } from "@snowbridge/contract-types"
import { CallDryRunEffects, EventRecord, XcmDryRunApiError, XcmDryRunEffects } from "@polkadot/types/interfaces";
import { Result } from "@polkadot/types";
import { AbstractProvider, Contract, ContractTransaction, FeeData,  TransactionReceipt } from "ethers";

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

export type DeliveryFee = {
    assetHubExecutionFeeDOT: bigint
    totalFeeInDot: bigint
}

function resolveInputs(registry: AssetRegistry, tokenAddress: string, sourceParaId: number) {
    const tokenErcMetadata =
        registry.ethereumChains[registry.ethChainId.toString()].assets[tokenAddress.toLowerCase()]
    if (!tokenErcMetadata) {
        throw Error(`No token ${tokenAddress} registered on ethereum chain ${registry.ethChainId}.`)
    }
    const sourceParachain = registry.parachains[sourceParaId.toString()]
    if (!sourceParachain) {
        throw Error(`Could not find ${sourceParaId} in the asset registry.`)
    }
    const ahAssetMetadata =
        registry.parachains[registry.assetHubParaId].assets[tokenAddress.toLowerCase()]
    if (!ahAssetMetadata) {
        throw Error(`Token ${tokenAddress} not registered on asset hub.`)
    }

    const sourceAssetMetadata = sourceParachain.assets[tokenAddress.toLowerCase()]
    if (!sourceAssetMetadata) {
        throw Error(`Token ${tokenAddress} not registered on source parachain ${sourceParaId}.`)
    }

    return { tokenErcMetadata, sourceParachain, ahAssetMetadata, sourceAssetMetadata }
}

export async function createTransfer(
    parachain: ApiPromise,
    registry: AssetRegistry,
    sourceAccount: string,
    beneficiaryAccount: string,
    tokenAddress: string,
    amount: bigint,
    totalFeeInDot: bigint,
): Promise<Transfer> {
    const { ethChainId, assetHubParaId } = registry
    let sourceParaId = assetHubParaId;

    let sourceAccountHex = sourceAccount
    if (!isHex(sourceAccountHex)) {
        sourceAccountHex = u8aToHex(decodeAddress(sourceAccount))
    }

    const { tokenErcMetadata, sourceParachain, ahAssetMetadata, sourceAssetMetadata } =
        resolveInputs(registry, tokenAddress, sourceParaId)
    let messageId = await buildMessageId(parachain, sourceParaId, sourceAccountHex, tokenAddress, beneficiaryAccount, amount)
    let  tx = createERC20SourceParachainTxKusama(parachain, ethChainId,  sourceAccountHex, tokenAddress, sourceAccountHex, amount, totalFeeInDot)

    return {
        input: {
            registry,
            sourceAccount,
            beneficiaryAccount,
            tokenAddress,
            amount,
            fee: {
                totalFeeInDot,
                assetHubExecutionFeeDOT: 0n
            },
        },
        computed: {
            sourceParaId: assetHubParaId,
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

export type ValidationResult = {
    logs: ValidationLog[]
    success: boolean
    data: {
        nativeBalance: bigint
        dotBalance: bigint
        sourceExecutionFee: bigint
        tokenBalance: bigint
        sourceDryRunError: any
        assetHubDryRunError: any
    };
    transfer: Transfer
}

export async function validateTransfer(
    assetHub: ApiPromise,
    transfer: Transfer): Promise<ValidationResult> {

    const { registry, fee, tokenAddress, amount, beneficiaryAccount } = transfer.input
    const { sourceAccountHex, sourceParaId, sourceParachain: source } = transfer.computed
    const { tx } = transfer

    const logs: ValidationLog[] = []

    const [nativeBalance, dotBalance, tokenBalance] = await Promise.all([
        getNativeBalance(assetHub, sourceAccountHex),
        getDotBalance(assetHub, source.info.specName, sourceAccountHex),
        getTokenBalance(assetHub, source.info.specName, sourceAccountHex, registry.ethChainId, tokenAddress)
    ])

    if (amount > tokenBalance) {
        logs.push({ kind: ValidationKind.Error, reason: ValidationReason.InsufficientTokenBalance, message: 'Insufficient token balance to submit transaction.' })
    }

    let sourceDryRunError;
    let assetHubDryRunError;
    // do the dry run, get the forwarded xcm and dry run that
    const dryRunSource = await dryRunOnSourceParachain(assetHub, registry.assetHubParaId, registry.bridgeHubParaId, transfer.tx, sourceAccountHex)
    if (!dryRunSource.success) {
        logs.push({ kind: ValidationKind.Error, reason: ValidationReason.DryRunFailed, message: 'Dry run call on source failed.' })
        sourceDryRunError = dryRunSource.error
    }

    const paymentInfo = await tx.paymentInfo(sourceAccountHex)
    const sourceExecutionFee = paymentInfo['partialFee'].toBigInt()

    console.log("paymentInfo:", paymentInfo);
    console.log("sourceExecutionFee:", sourceExecutionFee);
    console.log("dotBalance:", dotBalance);
    console.log("nativeBalance:", nativeBalance);
    console.log("tokenBalance:", tokenBalance);
    console.log("amount:", amount);

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

    const success = logs.find(l => l.kind === ValidationKind.Error) === undefined

    return {
        logs,
        success,
        data: {
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

export type MessageReceiptEvm = {
    blockNumber: number
    blockHash: string
    substrateBlockHash: string
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

    return result
}

export function createERC20SourceParachainTxKusama(
    parachain: ApiPromise,
    ethChainId: number,
    sourceAccount: string,
    tokenAddress: string,
    beneficiaryAccount: string,
    amount: bigint,
    totalFeeInDot: bigint,
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
    const destination = { v4: kusamaAssetHubLocation() }

    const feeAsset = {
        v4: DOT_LOCATION
    }
    const customXcm = buildAssetHubERC20TransferToKusama(parachain.registry, sourceAccount, beneficiaryAccount)
    return parachain.tx.polkadotXcm.transferAssetsUsingTypeAndThen(destination, assets, "LocalReserve", feeAsset, "LocalReserve", customXcm, "Unlimited")
}

export async function dryRunOnSourceParachain(
    source: ApiPromise,
    assetHubParaId: number,
    bridgeHubParaId: number,
    tx: SubmittableExtrinsic<"promise", ISubmittableResult>,
    sourceAccount: string
) {
    const origin = { system: { signed: sourceAccount } }
    const result = (await source.call.dryRunApi.dryRunCall<Result<CallDryRunEffects, XcmDryRunApiError>>(
        origin,
        tx,
    ))

    let assetHubForwarded;
    let bridgeHubForwarded;
    const success = result.isOk && result.asOk.executionResult.isOk
    if (!success) {
        console.error("Error during dry run on source parachain:", sourceAccount, tx.toHuman(), result.toHuman(true));
        let err = result.isOk && result.asOk.executionResult.isErr ? result.asOk.executionResult.asErr.toJSON() : undefined;
        console.error("Result:", err);
    } else {
        bridgeHubForwarded = result.asOk.forwardedXcms.find(x => {
            return x[0].isV4
                && x[0].asV4.parents.toNumber() === 1
                && x[0].asV4.interior.isX1
                && x[0].asV4.interior.asX1[0].isParachain
                && x[0].asV4.interior.asX1[0].asParachain.toNumber() === bridgeHubParaId
        })
        assetHubForwarded = result.asOk.forwardedXcms.find(x => {
            return x[0].isV4
                && x[0].asV4.parents.toNumber() === 1
                && x[0].asV4.interior.isX1
                && x[0].asV4.interior.asX1[0].isParachain
                && x[0].asV4.interior.asX1[0].asParachain.toNumber() === assetHubParaId
        })
    }
    return {
        success: success && (bridgeHubForwarded || assetHubForwarded),
        error: result.isOk && result.asOk.executionResult.isErr ? result.asOk.executionResult.asErr.toJSON() : undefined,
        assetHubForwarded,
        bridgeHubForwarded,
    }
}

async function dryRunAssetHub(assetHub: ApiPromise, parachainId: number, bridgeHubParaId: number, xcm: any) {
    const sourceParachain = { v4: { parents: 1, interior: { x1: [{ parachain: parachainId }] } } }
    const result = (await assetHub.call.dryRunApi.dryRunXcm<Result<XcmDryRunEffects, XcmDryRunApiError>>(
        sourceParachain,
        xcm
    ))

    const resultPrimitive = result.toPrimitive() as any
    const resultHuman = result.toHuman() as any

    const success = result.isOk && result.asOk.executionResult.isComplete
    let sourceParachainForwarded;
    let bridgeHubForwarded;
    if (!success) {
        console.error("Error during dry run on asset hub:", xcm.toHuman(), result.toHuman())
    } else {
        bridgeHubForwarded = result.asOk.forwardedXcms.find(x => {
            return x[0].isV4
                && x[0].asV4.parents.toNumber() === 1
                && x[0].asV4.interior.isX1
                && x[0].asV4.interior.asX1[0].isParachain
                && x[0].asV4.interior.asX1[0].asParachain.toNumber() === bridgeHubParaId
        })
        sourceParachainForwarded = result.asOk.forwardedXcms.find(x => {
            return x[0].isV4
                && x[0].asV4.parents.toNumber() === 1
                && x[0].asV4.interior.isX1
                && x[0].asV4.interior.asX1[0].isParachain
                && x[0].asV4.interior.asX1[0].asParachain.toNumber() === parachainId
        })
    }
    return {
        success: success && bridgeHubForwarded,
        sourceParachainForwarded,
        bridgeHubForwarded,
        errorMessage: resultHuman.Ok.executionResult.Incomplete?.error,
    }
}

async function buildMessageId(parachain: ApiPromise, sourceParaId: number, sourceAccountHex: string, tokenAddress: string, beneficiaryAccount: string, amount: bigint) {
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
    return blake2AsHex(entropy);
}

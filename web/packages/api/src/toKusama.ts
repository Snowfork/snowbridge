import { ApiPromise } from "@polkadot/api";
import { AddressOrPair, SignerOptions, SubmittableExtrinsic } from "@polkadot/api/types";
import { Codec, ISubmittableResult } from "@polkadot/types/types";
import { BN, hexToU8a, isHex, stringToU8a, u8aToHex } from "@polkadot/util";
import { blake2AsHex, decodeAddress, xxhashAsHex } from "@polkadot/util-crypto";
import {
    DOT_LOCATION,
    erc20Location,
    kusamaAssetHubLocation,
    buildAssetHubERC20TransferToKusama,
    dotLocationOnKusamaAssetHubLocation,
    polkadotAssetHubLocation,
    buildExportXcmForERC20,
    buildTransferToKusamaExportXCM,
    buildExportXcmForPNA,
    buildKusamaAssetHubExportXCM
} from "./xcmBuilder";
import {
    Asset,
    AssetRegistry, calculateDeliveryFee,
    getDotBalance, getLocationBalance,
    getNativeAccount,
    getNativeBalance,
    getTokenBalance, padFeeByPercentage,
    Parachain
} from "./assets_v2";
import {CallDryRunEffects, EventRecord, XcmDryRunApiError, XcmDryRunEffects} from "@polkadot/types/interfaces";
import { Result } from "@polkadot/types";
import {beneficiaryMultiAddress} from "./utils";

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
        beneficiaryAddressHex: string
        sourceAccountHex: string
        sourceAssetMetadata: Asset
        destAssetMetadata: Asset
        sourceParachain: Parachain
        messageId?: string
    },
    tx: SubmittableExtrinsic<"promise", ISubmittableResult>
}

export type DeliveryFee = {
    xcmBridgeFeeDOT: bigint
    bridgeHubDeliveryFeeDOT: bigint
    totalFeeInDot: bigint
}

export enum Direction {
    ToKusama, ToPolkadot
}

function resolveInputs(registry: AssetRegistry, tokenAddress: string, sourceParaId: number, destParaId: number) {
    const sourceParachain = registry.parachains[sourceParaId.toString()]
    if (!sourceParachain) {
        throw Error(`Could not find ${sourceParaId} in the asset registry.`)
    }
    const destParachain = registry.kusama?.parachains[destParaId.toString()];
    if (!destParachain) {
        throw Error(`Could not find ${destParaId} in the asset registry.`)
    }

    const sourceAssetMetadata =
        registry.parachains[sourceParaId].assets[tokenAddress.toLowerCase()]
    if (!sourceAssetMetadata) {
        throw Error(`Token ${tokenAddress} not registered on source asset hub.`)
    }
    const destAssetMetadata =
        registry.kusama?.parachains[destParaId].assets[tokenAddress.toLowerCase()];
    if (!destAssetMetadata) {
        throw Error(`Token ${tokenAddress} not registered on destination asset hub.`)
    }

    return { sourceAssetMetadata, destAssetMetadata, sourceParachain }
}

export async function getDeliveryFee(
    assetHub: ApiPromise,
    registry: AssetRegistry,
    defaultFee?: bigint,
    padPercentage?: bigint
): Promise<DeliveryFee> {
    const feeStorageKey = xxhashAsHex(":XcmBridgeHubRouterBaseFee:", 128, true)
    const feeStorageItem = await assetHub.rpc.state.getStorage(feeStorageKey)
    const feePadPercentage = padPercentage ?? 20n
    let leFee = new BN((feeStorageItem as Codec).toHex().replace("0x", ""), "hex", "le")

    let xcmBridgeFee: bigint
    if (leFee.eqn(0)) {
        console.warn("Asset Hub onchain XcmBridgeHubRouterBaseFee not set. Using default fee.")
        xcmBridgeFee = defaultFee ?? 333_794_429n
    } else {
        xcmBridgeFee = BigInt(leFee.toString())
    }

    xcmBridgeFee = padFeeByPercentage(xcmBridgeFee, feePadPercentage);

    let forwardedXcm = buildTransferToKusamaExportXCM(
        assetHub.registry,
        xcmBridgeFee,
        xcmBridgeFee,
        registry.assetHubParaId,
        erc20Location(registry.ethChainId, "0x0000000000000000000000000000000000000000"), // actual token location doesn't matter here, just weighing the message
        340282366920938463463374607431768211455n,
        "0x0000000000000000000000000000000000000000000000000000000000000000",
        "0x0000000000000000000000000000000000000000000000000000000000000000");

    let bridgeHubDeliveryFeeDOT = await calculateDeliveryFee(
        assetHub,
        registry.bridgeHubParaId,
        forwardedXcm
    )

    console.info("Total fee in DOT:", xcmBridgeFee + bridgeHubDeliveryFeeDOT)

    return {
        xcmBridgeFeeDOT: xcmBridgeFee,
        bridgeHubDeliveryFeeDOT: bridgeHubDeliveryFeeDOT,
        totalFeeInDot: (xcmBridgeFee + bridgeHubDeliveryFeeDOT) * feePadPercentage
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
    const { ethChainId, assetHubParaId } = registry
    const destParaId = registry.kusama?.assetHubParaId
    let sourceParaId = assetHubParaId;

    let sourceAccountHex = sourceAccount
    if (!isHex(sourceAccountHex)) {
        sourceAccountHex = u8aToHex(decodeAddress(sourceAccount))
    }

    if (!destParaId) {
        throw Error("Kusama destination para ID is not set")
    }

    const { sourceAssetMetadata, destAssetMetadata, sourceParachain } = resolveInputs(registry, tokenAddress, sourceParaId, destParaId)
    let messageId = await buildMessageId(parachain, sourceParaId, sourceAccountHex, tokenAddress, beneficiaryAccount, amount)

    let tx;
    if (direction == Direction.ToPolkadot) {
        tx = createERC20ToPolkadotTx(parachain, ethChainId,  sourceAccountHex, tokenAddress, sourceAccountHex, amount, fee.totalFeeInDot)
    } else {
        tx = createERC20ToKusamaTx(parachain, ethChainId,  sourceAccountHex, tokenAddress, sourceAccountHex, amount, fee.totalFeeInDot)
    }

    let { hexAddress: beneficiaryAddressHex } =
        beneficiaryMultiAddress(beneficiaryAccount)

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
            sourceParaId: assetHubParaId,
            sourceParachain,
            sourceAssetMetadata,
            sourceAccountHex,
            destAssetMetadata,
            messageId,
            beneficiaryAddressHex
        },
        tx
    }
}

export enum ValidationKind {
    Warning, Error
}

export enum ValidationReason {
    InsufficientTokenBalance,
    InsufficientDotFee,
    DryRunFailed,
    MaxConsumersReached,
    AccountDoesNotExist
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
        assetHubDryRunError: any
    };
    transfer: Transfer
}

export async function validateTransfer(
    connections: {
        sourceAssetHub: ApiPromise
        destAssetHub: ApiPromise
        sourceBridgeHub: ApiPromise
        destinationBridgeHub: ApiPromise
    },
    direction: Direction,
    transfer: Transfer,
    ): Promise<ValidationResult> {

    let sourceAssetHub = connections.sourceAssetHub;
    let destAssetHub = connections.destAssetHub;

    const { registry, fee, tokenAddress, amount } = transfer.input
    const { sourceAccountHex, sourceParachain: source, beneficiaryAddressHex, destAssetMetadata } = transfer.computed
    const { tx } = transfer

    const [nativeBalance, tokenBalance] = await Promise.all([
        getNativeBalance(sourceAssetHub, sourceAccountHex),
        getTokenBalance(sourceAssetHub, source.info.specName, sourceAccountHex, registry.ethChainId, tokenAddress)
    ])

    let dotBalance = 0n;
    if (direction == Direction.ToPolkadot) {
        dotBalance = await getLocationBalance(sourceAssetHub, source.info.specName, dotLocationOnKusamaAssetHubLocation(), sourceAccountHex);
    } else {
        dotBalance = await getDotBalance(sourceAssetHub, source.info.specName, sourceAccountHex);
    }

    const logs: ValidationLog[] = []

    const { accountMaxConsumers, accountExists } = await validateAccount(
        destAssetHub,
        "statemint",
        beneficiaryAddressHex,
        registry.ethChainId,
        tokenAddress,
        destAssetMetadata
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

    if (amount > tokenBalance) {
        logs.push({ kind: ValidationKind.Error, reason: ValidationReason.InsufficientTokenBalance, message: 'Insufficient token balance to submit transaction.' })
    }

    let assetHubDryRunError;

    const dryRunSource = await dryRunOnSourceParachain(sourceAssetHub, direction, registry.assetHubParaId, registry.bridgeHubParaId, transfer.tx, sourceAccountHex)
    if (!dryRunSource.success) {
        logs.push({ kind: ValidationKind.Error, reason: ValidationReason.DryRunFailed, message: 'Dry run call on source failed.' })
        assetHubDryRunError = dryRunSource.error
    }

    const paymentInfo = await tx.paymentInfo(sourceAccountHex)
    const sourceExecutionFee = paymentInfo['partialFee'].toBigInt()

    if ((sourceExecutionFee + fee.totalFeeInDot) > (dotBalance)) {
        logs.push({ kind: ValidationKind.Error, reason: ValidationReason.InsufficientDotFee, message: 'Insufficient DOT balance to submit transaction on the source parachain.' })
    }

    let kahXCM = buildKusamaAssetHubExportXCM(
        destAssetHub.registry,
        fee.totalFeeInDot,
        registry.assetHubParaId,
        registry.kusama?.parachains[registry.assetHubParaId].assets[tokenAddress].location,
        transfer.input.amount,
        transfer.input.beneficiaryAccount,
        "0x0000000000000000000000000000000000000000000000000000000000000000",
    );

    const dryRunKAH = await dryRunAssetHub(destAssetHub, registry.assetHubParaId, registry.bridgeHubParaId, kahXCM);
    if (!dryRunKAH.success) {
        logs.push({ kind: ValidationKind.Error, reason: ValidationReason.DryRunFailed, message: 'Dry run call on Kusama AH failed: ' + dryRunKAH.errorMessage })
        assetHubDryRunError = dryRunKAH.errorMessage
    }

    console.log("dotBalance:", dotBalance);
    console.log("nativeBalance:", nativeBalance);
    console.log("tokenBalance:", tokenBalance);
    console.log("amount:", amount);
    console.log("fee:", fee)
    console.log("sourceExecutionFee:", sourceExecutionFee)
    console.log("TOTAL FEE", sourceExecutionFee + fee.totalFeeInDot)

    const success = logs.find(l => l.kind === ValidationKind.Error) === undefined

    return {
        logs,
        success,
        data: {
            nativeBalance,
            dotBalance,
            sourceExecutionFee,
            tokenBalance,
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

    return result
}

export function createERC20ToKusamaTx(
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

export function createERC20ToPolkadotTx(
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
                id: dotLocationOnKusamaAssetHubLocation(),
                fun: { Fungible: totalFeeInDot },
            },
            {
                id: erc20Location(ethChainId, tokenAddress),
                fun: { Fungible: amount },
            },
        ]
    }
    const destination = { v4: polkadotAssetHubLocation() }

    const feeAsset = {
        v4: dotLocationOnKusamaAssetHubLocation()
    }
    const customXcm = buildAssetHubERC20TransferToKusama(parachain.registry, sourceAccount, beneficiaryAccount)
    return parachain.tx.polkadotXcm.transferAssetsUsingTypeAndThen(destination, assets, "DestinationReserve", feeAsset, "DestinationReserve", customXcm, "Unlimited")
}

export async function dryRunOnSourceParachain(
    source: ApiPromise,
    direction: Direction,
    assetHubParaId: number,
    bridgeHubParaId: number,
    tx: SubmittableExtrinsic<"promise", ISubmittableResult>,
    sourceAccount: string
) {
    const origin = { system: { signed: sourceAccount } }
    let result: Result<CallDryRunEffects, XcmDryRunApiError>;
    if (direction == Direction.ToPolkadot) {
        result = (await source.call.dryRunApi.dryRunCall<Result<CallDryRunEffects, XcmDryRunApiError>>(
            origin,
            tx,
            4
        ))
    } else {
        result = (await source.call.dryRunApi.dryRunCall<Result<CallDryRunEffects, XcmDryRunApiError>>(
            origin,
            tx,
        ))
    }

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

async function dryRunAssetHub(
    assetHub: ApiPromise,
    parachainId: number,
    bridgeHubParaId: number,
    xcm: any
) {
    const sourceParachain = { v4: { parents: 1, interior: { x1: [{ parachain: parachainId }] } } }
    const result = await assetHub.call.dryRunApi.dryRunXcm<
        Result<XcmDryRunEffects, XcmDryRunApiError>
    >(sourceParachain, xcm)

    const resultPrimitive = result.toPrimitive() as any
    const resultHuman = result.toHuman() as any

    const success = result.isOk && result.asOk.executionResult.isComplete
    let sourceParachainForwarded
    let bridgeHubForwarded
    if (!success) {
        console.error("Error during dry run on asset hub:", xcm.toHuman(), result.toHuman())
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
        sourceParachainForwarded = result.asOk.forwardedXcms.find((x) => {
            return (
                x[0].isV4 &&
                x[0].asV4.parents.toNumber() === 1 &&
                x[0].asV4.interior.isX1 &&
                x[0].asV4.interior.asX1[0].isParachain &&
                x[0].asV4.interior.asX1[0].asParachain.toNumber() === parachainId
            )
        })
    }
    return {
        success: success && bridgeHubForwarded,
        sourceParachainForwarded,
        bridgeHubForwarded,
        errorMessage: resultHuman.Ok.executionResult.Incomplete?.error,
    }
}

async function validateAccount(
    parachain: ApiPromise,
    specName: string,
    beneficiaryAddress: string,
    ethChainId: number,
    tokenAddress: string,
    assetMetadata?: Asset,
    maxConsumers?: bigint
) {
    // Check if the account is created
    const [beneficiaryAccount, beneficiaryTokenBalance] = await Promise.all([
        getNativeAccount(parachain, beneficiaryAddress),
        getTokenBalance(
            parachain,
            specName,
            beneficiaryAddress,
            ethChainId,
            tokenAddress,
            assetMetadata
        ),
    ])
    return {
        accountExists: !(
            beneficiaryAccount.consumers === 0n &&
            beneficiaryAccount.providers === 0n &&
            beneficiaryAccount.sufficients === 0n
        ),
        accountMaxConsumers:
            beneficiaryAccount.consumers >= (maxConsumers ?? 63n) && beneficiaryTokenBalance === 0n,
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

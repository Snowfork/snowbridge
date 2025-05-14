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
    buildTransferToKusamaExportXCM,
    buildPolkadotToKusamaAssetHubExportXCM,
    buildKusamaToPolkadotAssetHubExportXCM, isDOTOnOtherConsensusSystem, isDOTOnPolkadotAssetHub
} from "./xcmBuilder";
import {
    Asset,
    AssetRegistry, calculateDeliveryFee,
    getDotBalance, getLocationBalance,
    getNativeAccount,
    getNativeBalance,
    getTokenBalance, padFeeByPercentage,
    Parachain, quoteFeeSwap
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
    xcmBridgeFee: bigint
    bridgeHubDeliveryFee: bigint
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
    sourceAssetHub: ApiPromise,
    direction: Direction,
    registry: AssetRegistry,
    defaultFee?: bigint,
    padPercentage?: bigint
): Promise<DeliveryFee> {
    const feeStorageKey = xxhashAsHex(":XcmBridgeHubRouterBaseFee:", 128, true)
    const feeStorageItem = await sourceAssetHub.rpc.state.getStorage(feeStorageKey)
    const feePadPercentage = padPercentage ?? 10n
    let leFee = new BN((feeStorageItem as Codec).toHex().replace("0x", ""), "hex", "le")

    let defaultFeeConfiguredInRuntime:bigint;
    if (direction == Direction.ToPolkadot) {
        defaultFeeConfiguredInRuntime = 10_602_492_378n; // .0106KSM
    } else {
        defaultFeeConfiguredInRuntime = 333_794_429n; // 0.033 DOT
    }
    let xcmBridgeFee: bigint
    if (leFee.eqn(0)) {
        console.warn("Asset Hub onchain XcmBridgeHubRouterBaseFee not set. Using default fee.")
        xcmBridgeFee = defaultFee ?? defaultFeeConfiguredInRuntime
    } else {
        xcmBridgeFee = BigInt(leFee.toString())
    }

    xcmBridgeFee = padFeeByPercentage(xcmBridgeFee, feePadPercentage);

    let forwardedXcm = buildTransferToKusamaExportXCM(
        sourceAssetHub.registry,
        DOT_LOCATION,
        dotLocationOnKusamaAssetHubLocation(),
        erc20Location(registry.ethChainId, "0x0000000000000000000000000000000000000000"), // actual token location doesn't matter here, just weighing the message
        xcmBridgeFee,
        xcmBridgeFee,
        registry.assetHubParaId,
        340282366920938463463374607431768211455n,
        "0x0000000000000000000000000000000000000000000000000000000000000000",
        "0x0000000000000000000000000000000000000000000000000000000000000000");

    let bridgeHubDeliveryFee = await calculateDeliveryFee(
        sourceAssetHub,
        registry.bridgeHubParaId,
        forwardedXcm
    )

    // In either DOT or KSM
    let totalFee = (xcmBridgeFee + bridgeHubDeliveryFee)

    let totalFeeInDot = 0n;
    // Convert KSM to DOT
    if (direction == Direction.ToPolkadot) {
        console.info("Converting KSM to DOT");
        let amount = xcmBridgeFee + bridgeHubDeliveryFee;
        totalFeeInDot = await quoteFeeSwap(
            sourceAssetHub,
            { parents: 1, interior: "Here" },
            { parents: 2, interior: { x1: [{ GlobalConsensus: { Polkadot: null } }] } },
            amount
        )
    } else {
        totalFeeInDot = totalFee
    }

    console.info("Total fee in DOT:", totalFeeInDot)

    return {
        xcmBridgeFee: xcmBridgeFee,
        bridgeHubDeliveryFee: bridgeHubDeliveryFee,
        totalFeeInDot: totalFeeInDot * feePadPercentage
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

    let tokenLocation = getTokenLocation(registry, direction, tokenAddress);
    let tx;
    if (direction == Direction.ToPolkadot) {
        tx = createERC20ToPolkadotTx(parachain,  sourceAccountHex, tokenLocation, sourceAccountHex, amount, fee.totalFeeInDot)
    } else {
        tx = createERC20ToKusamaTx(parachain,  sourceAccountHex, tokenLocation, sourceAccountHex, amount, fee.totalFeeInDot)
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

    let location = getTokenLocation(registry, direction, tokenAddress);

    let dotBalance = 0n;
    if (direction == Direction.ToPolkadot) {
        dotBalance = await getLocationBalance(sourceAssetHub, source.info.specName, dotLocationOnKusamaAssetHubLocation(), sourceAccountHex);
    } else {
        dotBalance = await getDotBalance(sourceAssetHub, source.info.specName, sourceAccountHex);
    }

    let tokenBalance: bigint;
    if (isDOT(direction, location)) {
        console.log("token transferred is DOT");
        tokenBalance = dotBalance;
    } else {
        console.log("token transferred is not DOT");
        tokenBalance = await getTokenBalance(sourceAssetHub, source.info.specName, sourceAccountHex, registry.ethChainId, tokenAddress);
    }

    console.log("dotBalance:", dotBalance);
    console.log("tokenBalance:", tokenBalance);

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

    const dryRunSource = await dryRunSourceAssetHub(sourceAssetHub, direction, registry.assetHubParaId, registry.bridgeHubParaId, transfer.tx, sourceAccountHex)
    if (!dryRunSource.success) {
        logs.push({ kind: ValidationKind.Error, reason: ValidationReason.DryRunFailed, message: 'Dry run call on source failed.' })
        assetHubDryRunError = dryRunSource.error
    }

    console.dir(transfer.tx.toHuman(), {depth: 100});

    const paymentInfo = await tx.paymentInfo(sourceAccountHex)
    const sourceExecutionFee = paymentInfo['partialFee'].toBigInt()

    if ((sourceExecutionFee + fee.totalFeeInDot) > (dotBalance)) {
        logs.push({ kind: ValidationKind.Error, reason: ValidationReason.InsufficientDotFee, message: 'Insufficient DOT balance to submit transaction on the source parachain.' })
    }

    let destAssetHubXCM: any;
    if (direction == Direction.ToPolkadot) {
        destAssetHubXCM = buildKusamaToPolkadotAssetHubExportXCM(
            destAssetHub.registry,
            fee.totalFeeInDot,
            registry.assetHubParaId,
            location,
            transfer.input.amount,
            transfer.input.beneficiaryAccount,
            "0x0000000000000000000000000000000000000000000000000000000000000000",
        );
    } else {
        destAssetHubXCM = buildPolkadotToKusamaAssetHubExportXCM(
            destAssetHub.registry,
            fee.totalFeeInDot,
            registry.assetHubParaId,
            location,
            transfer.input.amount,
            transfer.input.beneficiaryAccount,
            "0x0000000000000000000000000000000000000000000000000000000000000000",
        );
    }

    console.dir(destAssetHubXCM.toHuman(), {depth: 100});

    const dryRunAssetHubDest = await dryRunDestAssetHub(destAssetHub, registry.bridgeHubParaId, destAssetHubXCM);
    if (!dryRunAssetHubDest.success) {
        logs.push({ kind: ValidationKind.Error, reason: ValidationReason.DryRunFailed, message: 'Dry run call on destination AH failed: ' + dryRunAssetHubDest.errorMessage })
        assetHubDryRunError = dryRunAssetHubDest.errorMessage
    }

    console.log("amount:", amount);
    console.log("fee:", fee)
    console.log("sourceExecutionFee:", sourceExecutionFee)
    console.log("TOTAL FEE", sourceExecutionFee + fee.totalFeeInDot)

    const success = logs.find(l => l.kind === ValidationKind.Error) === undefined

    return {
        logs,
        success,
        data: {
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
    sourceAccount: string,
    tokenLocation: any,
    beneficiaryAccount: string,
    amount: bigint,
    totalFeeInDot: bigint,
): SubmittableExtrinsic<"promise", ISubmittableResult> {
    let assets: any;
    if (isDOT(Direction.ToKusama, tokenLocation)) {
        assets = {v4: [
                {
                    id: DOT_LOCATION,
                    fun: { Fungible: totalFeeInDot +  amount},
                },
            ]};
    } else {
        assets = {
            v4: [
                {
                    id: DOT_LOCATION,
                    fun: { Fungible: totalFeeInDot },
                },
                {
                    id: tokenLocation,
                    fun: { Fungible: amount },
                },
            ]
        }
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
    sourceAccount: string,
    tokenLocation: any,
    beneficiaryAccount: string,
    amount: bigint,
    totalFeeInDot: bigint,
): SubmittableExtrinsic<"promise", ISubmittableResult> {
    let assets: any;
    if (isDOT(Direction.ToPolkadot, tokenLocation)) {
        assets = {v4: [
            {
                id: dotLocationOnKusamaAssetHubLocation(),
                fun: { Fungible: totalFeeInDot +  amount},
            },
        ]};
    } else {
        assets = {
            v4: [
                {
                    id: dotLocationOnKusamaAssetHubLocation(),
                    fun: { Fungible: totalFeeInDot },
                },
                {
                    id: tokenLocation,
                    fun: { Fungible: amount },
                },
            ]
        }
    }

    const destination = { v4: polkadotAssetHubLocation() }

    const feeAsset = {
        v4: dotLocationOnKusamaAssetHubLocation()
    }
    const customXcm = buildAssetHubERC20TransferToKusama(parachain.registry, sourceAccount, beneficiaryAccount)
    return parachain.tx.polkadotXcm.transferAssetsUsingTypeAndThen(destination, assets, "DestinationReserve", feeAsset, "DestinationReserve", customXcm, "Unlimited")
}

export async function dryRunSourceAssetHub(
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

async function dryRunDestAssetHub(
    assetHub: ApiPromise,
    parachainId: number,
    xcm: any
) {
    const sourceParachain = { v4: { parents: 1, interior: { x1: [{ parachain: parachainId }] } } }
    const result = await assetHub.call.dryRunApi.dryRunXcm<
        Result<XcmDryRunEffects, XcmDryRunApiError>
    >(sourceParachain, xcm)

    const resultHuman = result.toHuman() as any

    const success = result.isOk && result.asOk.executionResult.isComplete
    if (!success) {
        console.error("Error during dry run on asset hub:", xcm.toHuman(), result.toHuman());
    }
    return {
        success: success,
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

function getTokenLocation(registry: AssetRegistry, direction: Direction, tokenAddress: string) {
    let location;
    if (direction == Direction.ToPolkadot) {
        location = registry.kusama?.parachains[registry.kusama?.assetHubParaId].assets[tokenAddress].location;
        if (!location) {
            location = erc20Location(registry.ethChainId, tokenAddress);
        }
    } else {
        location = registry.parachains[registry.assetHubParaId].assets[tokenAddress].location;
        if (!location) {
            location = erc20Location(registry.ethChainId, tokenAddress);
        }
    }

    return location;
}

function isDOT(direction: Direction, location: any) {
    if (direction == Direction.ToPolkadot) {
        return isDOTOnOtherConsensusSystem(location)
    } else {
        return isDOTOnPolkadotAssetHub(location)
    }
}

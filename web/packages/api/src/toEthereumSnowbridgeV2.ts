import { ApiPromise } from "@polkadot/api"
import { AddressOrPair, SignerOptions, SubmittableExtrinsic } from "@polkadot/api/types"
import { Codec, ISubmittableResult } from "@polkadot/types/types"
import {
    AssetRegistry,
    ChainId,
    ContractCall,
    EthereumChain,
    EthereumProviderTypes,
    Parachain,
    TransferRoute,
} from "@snowbridge/base-types"
import { CallDryRunEffects, XcmDryRunApiError, XcmDryRunEffects } from "@polkadot/types/interfaces"
import { Result } from "@polkadot/types"
import { DeliveryFee, dryRunBridgeHub } from "./toEthereum_v2"
import { PNAFromAH } from "./transfers/toEthereum/pnaFromAH"
import { TransferInterface } from "./transfers/toEthereum/transferInterface"
import { ERC20FromAH } from "./transfers/toEthereum/erc20FromAH"
import { PNAFromParachain } from "./transfers/toEthereum/pnaFromParachain"
import { ERC20FromParachain } from "./transfers/toEthereum/erc20FromParachain"
import {
    isRelaychainLocation,
    isParachainNative,
    HERE_LOCATION,
    bridgeLocation,
} from "./xcmBuilder"
import { xxhashAsHex } from "@polkadot/util-crypto"
import { BN } from "@polkadot/util"
import { ensureValidationSuccess, padFeeByPercentage, scaledPadPercentage } from "./utils"
import { Context } from "./index"
import { DOT_LOCATION, ETHER_TOKEN_ADDRESS, findL2TokenAddress } from "./assets_v2"
import { getOperatingStatus } from "./status"
import { calculateVolumeTipInWei, VolumeFeeParams } from "./feeSchedule"
import {
    addBreakdown,
    computeTotals,
    findInBreakdownOrZero,
    findTotal,
    findTotalOrUndefined,
} from "./fees"
import { estimateFees } from "./across/api"
import type { MessageReceipt, Transfer, ValidatedTransfer, ValidationLog } from "./types/toEthereum"
import { ValidationKind, ValidationReason } from "./types/toEthereum"
import {
    checkDotEthPoolLiquidityForPolkadotToEthereum,
    checkNativeDotPoolLiquidityForParachainToEthereum,
} from "./poolReserves"
import { ParachainBase } from "./parachains/parachainBase"

type V2CommandStruct = {
    kind: number
    gas: bigint
    payload: string
}

const V2_DISPATCH_OVERHEAD_GAS = 24_000n
const DRY_RUN_GAS_BUFFER = 30_000n
const GATEWAY_COMMAND_FAILED_TOPIC0 =
    "0xa6dc208277bb3da3666e7305baf550db2daf26f8f386a431a4b27cc7a02965a2"
const L1_ADAPTOR_DEPOSIT_CALL_INVOKED_TOPIC0 =
    "0x14bfd4fd7e654256d3222db5d1ec5e59cd23dd5df10bd8faccc1cabe984b3508"
const L1_ADAPTOR_DEPOSIT_CALL_FAILED_TOPIC0 =
    "0x759aee2ba41080c1e3a57140ba7b446c1347cff289214a2fd1c81554ddc17380"

type EthereumDryRunTx = {
    from?: string
    to?: string
    data?: string
    value?: bigint | string | number
}

type ForkedRpcProvider = {
    send(method: string, params: unknown[]): Promise<string>
    waitForTransaction(txHash: string): Promise<{
        status?: number | bigint | string | null
        logs: unknown[]
    } | null>
}

async function tryImpersonateForkedSigner(
    forkedProvider: ForkedRpcProvider,
    from?: string,
): Promise<boolean> {
    if (!from) {
        return false
    }

    // Ensure the impersonated account has balance to pass intrinsic checks.
    try {
        await forkedProvider.send("anvil_setBalance", [from, "0x56BC75E2D63100000"])
    } catch (e) {
        // Ignore: method may not exist on non-anvil nodes.
    }

    try {
        await forkedProvider.send("anvil_impersonateAccount", [from])
        return true
    } catch (e) {
        // fall through
    }

    try {
        await forkedProvider.send("hardhat_impersonateAccount", [from])
        return true
    } catch (e) {
        return false
    }
}

function encodeAbiTuple<T extends EthereumProviderTypes>(
    context: Context<T>,
    types: string[],
    values: readonly unknown[],
): string {
    return context.ethereumProvider.encodeAbiParameters(types, values)
}

export { signAndSendTransfer } from "./toEthereum_v2"
export { ValidationKind } from "./types/toEthereum"

export class TransferToEthereum<T extends EthereumProviderTypes> implements TransferInterface<T> {
    #pnaImpl?: TransferInterface<T>
    #erc20Impl?: TransferInterface<T>

    constructor(
        public readonly context: Context<T>,
        private readonly route: TransferRoute,
        private readonly registry: AssetRegistry,
        private readonly source: Parachain,
        private readonly destination: EthereumChain,
    ) {}

    get from(): ChainId {
        return this.route.from
    }

    get to(): ChainId {
        return this.route.to
    }

    #resolveByTokenAddress(tokenAddress: string): TransferInterface<T> {
        const sourceParaId = this.route.from.id
        const sourceAssetMetadata = this.source.assets[tokenAddress.toLowerCase()]
        if (!sourceAssetMetadata) {
            throw Error(
                `Token ${tokenAddress} not registered on source parachain ${this.source.id}.`,
            )
        }

        if (sourceAssetMetadata.location) {
            this.#pnaImpl ??=
                sourceParaId == this.registry.assetHubParaId
                    ? new PNAFromAH(
                          this.context,
                          this.registry,
                          this.route,
                          this.source,
                          this.destination,
                      )
                    : new PNAFromParachain(
                          this.context,
                          this.registry,
                          this.route,
                          this.source,
                          this.destination,
                      )
            return this.#pnaImpl
        }

        this.#erc20Impl ??=
            sourceParaId == this.registry.assetHubParaId
                ? new ERC20FromAH(
                      this.context,
                      this.registry,
                      this.route,
                      this.source,
                      this.destination,
                  )
                : new ERC20FromParachain(
                      this.context,
                      this.registry,
                      this.route,
                      this.source,
                      this.destination,
                  )
        return this.#erc20Impl
    }

    async fee(
        tokenAddress: string,
        options?: {
            padFeeByPercentage?: bigint
            slippagePadPercentage?: bigint
            defaultFee?: bigint
            feeTokenLocation?: any
            claimerLocation?: any
            contractCall?: ContractCall
            volumeFee?: VolumeFeeParams
        },
    ): Promise<DeliveryFee> {
        return this.#resolveByTokenAddress(tokenAddress).fee(tokenAddress, options)
    }

    async tx(
        sourceAccount: string,
        beneficiaryAccount: string,
        tokenAddress: string,
        amount: bigint,
        fee: DeliveryFee,
        options?: {
            claimerLocation?: any
            contractCall?: ContractCall
        },
    ): Promise<Transfer> {
        return this.#resolveByTokenAddress(tokenAddress).tx(
            sourceAccount,
            beneficiaryAccount,
            tokenAddress,
            amount,
            fee,
            options,
        )
    }

    async build(
        sourceAccount: string,
        beneficiaryAccount: string,
        tokenAddress: string,
        amount: bigint,
        options?: {
            fee?: {
                padFeeByPercentage?: bigint
                slippagePadPercentage?: bigint
                defaultFee?: bigint
                feeTokenLocation?: any
                claimerLocation?: any
                contractCall?: ContractCall
                volumeFee?: VolumeFeeParams
            }
            tx?: {
                claimerLocation?: any
                contractCall?: ContractCall
            }
        },
    ): Promise<ValidatedTransfer> {
        const fee = await this.fee(tokenAddress, options?.fee)
        const transfer = await this.tx(
            sourceAccount,
            beneficiaryAccount,
            tokenAddress,
            amount,
            fee,
            options?.tx,
        )
        return ensureValidationSuccess(await this.validate(transfer))
    }

    async validate(transfer: Transfer): Promise<ValidatedTransfer> {
        return this.#resolveByTokenAddress(transfer.input.tokenAddress).validate(transfer)
    }

    async signAndSend(
        transfer: Transfer,
        account: AddressOrPair,
        options: Partial<SignerOptions>,
    ): Promise<MessageReceipt> {
        return this.#resolveByTokenAddress(transfer.input.tokenAddress).signAndSend(
            transfer,
            account,
            options,
        )
    }
}

export async function dryRunOnSourceParachain(
    source: ApiPromise,
    assetHubParaId: number,
    bridgeHubParaId: number,
    tx: SubmittableExtrinsic<"promise", ISubmittableResult>,
    sourceAccount: string,
) {
    const origin = { system: { signed: sourceAccount } }
    // To ensure compatibility, dryRunCall includes the version parameter in XCMv5.
    let result
    try {
        result = await source.call.dryRunApi.dryRunCall<
            Result<CallDryRunEffects, XcmDryRunApiError>
        >(origin, tx.inner.toHex(), 5)
    } catch {
        result = await source.call.dryRunApi.dryRunCall<
            Result<CallDryRunEffects, XcmDryRunApiError>
        >(origin, tx.inner.toHex())
    }

    let assetHubForwarded
    let bridgeHubForwarded
    const success = result.isOk && result.asOk.executionResult.isOk
    if (!success) {
        console.error(
            "Error during dry run on source parachain:",
            sourceAccount,
            tx.toHuman(),
            result.toHuman(),
        )
    } else {
        bridgeHubForwarded =
            result.asOk.forwardedXcms.find((x) => {
                return (
                    x[0].isV4 &&
                    x[0].asV4.parents.toNumber() === 1 &&
                    x[0].asV4.interior.isX1 &&
                    x[0].asV4.interior.asX1[0].isParachain &&
                    x[0].asV4.interior.asX1[0].asParachain.toNumber() === bridgeHubParaId
                )
            }) ??
            result.asOk.forwardedXcms.find((x) => {
                return (
                    x[0].isV5 &&
                    x[0].asV5.parents.toNumber() === 1 &&
                    x[0].asV5.interior.isX1 &&
                    x[0].asV5.interior.asX1[0].isParachain &&
                    x[0].asV5.interior.asX1[0].asParachain.toNumber() === bridgeHubParaId
                )
            })
        assetHubForwarded =
            result.asOk.forwardedXcms.find((x) => {
                return (
                    x[0].isV4 &&
                    x[0].asV4.parents.toNumber() === 1 &&
                    x[0].asV4.interior.isX1 &&
                    x[0].asV4.interior.asX1[0].isParachain &&
                    x[0].asV4.interior.asX1[0].asParachain.toNumber() === assetHubParaId
                )
            }) ??
            result.asOk.forwardedXcms.find((x) => {
                return (
                    x[0].isV5 &&
                    x[0].asV5.parents.toNumber() === 1 &&
                    x[0].asV5.interior.isX1 &&
                    x[0].asV5.interior.asX1[0].isParachain &&
                    x[0].asV5.interior.asX1[0].asParachain.toNumber() === assetHubParaId
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

export async function dryRunAssetHub(
    assetHub: ApiPromise,
    parachainId: number,
    bridgeHubParaId: number,
    xcm: any,
) {
    const sourceParachain = { v5: { parents: 1, interior: { x1: [{ parachain: parachainId }] } } }
    const result = await assetHub.call.dryRunApi.dryRunXcm<
        Result<XcmDryRunEffects, XcmDryRunApiError>
    >(sourceParachain, xcm)

    const resultHuman = result.toHuman() as any

    const success = result.isOk && result.asOk.executionResult.isComplete
    let sourceParachainForwarded
    let bridgeHubForwarded
    if (!success) {
        console.error("Error during dry run on asset hub:", xcm.toHuman(), result.toHuman())
    } else {
        bridgeHubForwarded = result.asOk.forwardedXcms.find((x) => {
            return (
                x[0].isV5 &&
                x[0].asV5.parents.toNumber() === 1 &&
                x[0].asV5.interior.isX1 &&
                x[0].asV5.interior.asX1[0].isParachain &&
                x[0].asV5.interior.asX1[0].asParachain.toNumber() === bridgeHubParaId
            )
        })
        sourceParachainForwarded = result.asOk.forwardedXcms.find((x) => {
            return (
                x[0].isV5 &&
                x[0].asV5.parents.toNumber() === 1 &&
                x[0].asV5.interior.isX1 &&
                x[0].asV5.interior.asX1[0].isParachain &&
                x[0].asV5.interior.asX1[0].asParachain.toNumber() === parachainId
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

// pallet_xcm::execute reserves max_weight up front in block weight accounting.
// Querying the runtime for the actual weight (with margin) avoids over-reserving
// past the collator's per-extrinsic carve-out, which can leave the tx stuck.
// Per-field: a 0 from the runtime (some chains don't account for proofSize in
// their XCM Weigher) is replaced with the chain's default; otherwise the value
// is padded 25% and capped at the default.
export const queryXcmExecuteWeight = async (
    sourceParachainImpl: ParachainBase,
    sourceParachain: Parachain,
    xcm: any,
): Promise<{ refTime: bigint; proofSize: bigint }> => {
    const max = sourceParachainImpl.getMaxWeight()
    if (!sourceParachain.features.hasXcmPaymentApi) {
        return max
    }
    const result = (
        await sourceParachainImpl.provider.call.xcmPaymentApi.queryXcmWeight(xcm)
    ).toPrimitive() as any
    if (!result?.ok) {
        return max
    }
    const apply = (raw: bigint, ceiling: bigint): bigint => {
        if (raw === 0n) return ceiling
        const padded = (raw * 125n) / 100n
        return padded > ceiling ? ceiling : padded
    }
    return {
        refTime: apply(BigInt(result.ok.refTime.toString()), max.refTime),
        proofSize: apply(BigInt(result.ok.proofSize.toString()), max.proofSize),
    }
}

export const isFeeAllowed = (feeLocation: any, sourceParaId: number) => {
    return isRelaychainLocation(feeLocation) || isParachainNative(feeLocation, sourceParaId)
}

export const getSnowbridgeDeliveryFee = async (assetHub: ApiPromise, defaultFee?: bigint) => {
    const feeStorageKey = xxhashAsHex(":BridgeHubEthereumBaseFeeV2:", 128, true)
    const feeStorageItem = await assetHub.rpc.state.getStorage(feeStorageKey)
    let leFee = new BN((feeStorageItem as Codec).toHex().replace("0x", ""), "hex", "le")
    let snowbridgeDeliveryFeeDOT = 0n
    if (leFee.eqn(0)) {
        snowbridgeDeliveryFeeDOT = defaultFee ?? 150_000_000_000n
    } else {
        snowbridgeDeliveryFeeDOT = BigInt(leFee.toString())
    }
    return snowbridgeDeliveryFeeDOT
}

export type DeliveryXcm = {
    localXcm: any
    forwardedXcmToBH: any
    forwardXcmToAH?: any
    returnToSenderXcm?: any
}

export const estimateEthereumExecutionFee = async <T extends EthereumProviderTypes>(
    context: Context<T>,
    registry: AssetRegistry,
    sourceParachain: Parachain,
    tokenAddress: string,
    options?: {
        contractCall?: ContractCall
        fillDeadlineBuffer?: bigint
        accelerated?: boolean
    },
): Promise<bigint> => {
    const { executionFee } = await estimateEthereumExecutionFees(
        context,
        registry,
        sourceParachain,
        tokenAddress,
        options,
    )
    return executionFee
}

const estimateEthereumExecutionFees = async <T extends EthereumProviderTypes>(
    context: Context<T>,
    registry: AssetRegistry,
    sourceParachain: Parachain,
    tokenAddress: string,
    options?: {
        contractCall?: ContractCall
        fillDeadlineBuffer?: bigint
        accelerated?: boolean
    },
): Promise<{ executionFee: bigint; accelerationFee: bigint }> => {
    const ethereum = await context.ethereum()
    const tokenErcMetadata =
        registry.ethereumChains[`ethereum_${registry.ethChainId}`].assets[
            tokenAddress.toLowerCase()
        ]
    if (!tokenErcMetadata) {
        throw Error(`No token ${tokenAddress} registered on ethereum chain ${registry.ethChainId}.`)
    }

    // Calculate execution cost on ethereum including:
    // 1. the consensus update, which is the fiat-shamir submit (if accelerated) or two phase submit if not.
    // 2. message verification
    // 3. a static dispatch margin
    // 4. token delivery
    // 5. and the optional contract call.
    // All should leave enough margin to make sure the relay is profitable even in worst case scenarios.
    const ethereumChain = registry.ethereumChains[`ethereum_${registry.ethChainId}`]
    const feeData = await context.ethereumProvider.getFeeData(ethereum)
    const gasPrice = feeData.gasPrice ?? 2_000_000_000n
    const twoPhaseSubmitGas = ethereumChain.twoPhaseSubmitGas ?? 1_000_000n
    const submitFiatShamirGas = ethereumChain.submitFiatShamirGas ?? 2_000_000n
    const consensusUpdateGas = options?.accelerated ? submitFiatShamirGas : twoPhaseSubmitGas
    const messageVerificationGas = ethereumChain.baseVerificationGas ?? 120_000n
    const dispatchGas = ethereumChain.baseDispatchGas ?? 80_000n
    const tokenDeliveryGas = tokenErcMetadata.deliveryGas ?? 100_000n
    const contractCallGas = options?.contractCall?.gas ?? 0n
    const totalGas =
        consensusUpdateGas +
        messageVerificationGas +
        dispatchGas +
        tokenDeliveryGas +
        contractCallGas
    const ethereumExecutionFee = gasPrice * totalGas
    const accelerationGas: bigint =
        options?.accelerated && submitFiatShamirGas > twoPhaseSubmitGas
            ? submitFiatShamirGas - twoPhaseSubmitGas
            : 0n
    const accelerationFee = gasPrice * accelerationGas
    return { executionFee: ethereumExecutionFee, accelerationFee }
}

export const estimateFeesFromAssetHub = async <T extends EthereumProviderTypes>(
    context: Context<T>,
    registry: AssetRegistry,
    tokenAddress: string,
    deliveryXcm: DeliveryXcm,
    options?: {
        padFeeByPercentage?: bigint
        slippagePadPercentage?: bigint
        defaultFee?: bigint
        feeTokenLocation?: any
        contractCall?: ContractCall
        l2PadFeeByPercentage?: bigint
        l2TransferGasLimit?: bigint
        fillDeadlineBuffer?: bigint
        volumeFee?: VolumeFeeParams
        accelerated?: boolean
    },
    l2ChainId?: number,
    tokenAmount?: bigint,
): Promise<DeliveryFee> => {
    const assetHub = await context.parachain(registry.assetHubParaId)
    const assetHubImpl = await context.paraImplementation(assetHub)

    const feePadPercentage = options?.padFeeByPercentage ?? 33n
    const feeSlippagePadPercentage = options?.slippagePadPercentage ?? 20n

    let localExecutionFeeDOT = 0n
    let assetHubExecutionFeeDOT = 0n
    let returnToSenderExecutionFeeDOT = 0n
    let returnToSenderDeliveryFeeDOT = 0n
    let bridgeHubDeliveryFeeDOT = 0n
    let snowbridgeDeliveryFeeDOT = 0n

    localExecutionFeeDOT = padFeeByPercentage(
        await assetHubImpl.calculateXcmFee(deliveryXcm.localXcm, DOT_LOCATION),
        feePadPercentage,
    )

    bridgeHubDeliveryFeeDOT = padFeeByPercentage(
        await assetHubImpl.calculateDeliveryFeeInDOT(
            registry.bridgeHubParaId,
            deliveryXcm.forwardedXcmToBH,
        ),
        feePadPercentage,
    )

    snowbridgeDeliveryFeeDOT = await getSnowbridgeDeliveryFee(assetHub, options?.defaultFee)

    let totalFeeInDot =
        localExecutionFeeDOT +
        snowbridgeDeliveryFeeDOT +
        assetHubExecutionFeeDOT +
        returnToSenderExecutionFeeDOT +
        returnToSenderDeliveryFeeDOT +
        bridgeHubDeliveryFeeDOT

    // Calculate L2 bridge fee
    let l2BridgeFeeInL1Token: bigint = 0n
    if (l2ChainId) {
        let callInfo = await buildL2Call(
            context,
            registry,
            tokenAddress,
            l2ChainId,
            tokenAmount!,
            "0x0000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            options,
        )
        options = options || {}
        options.contractCall = options.contractCall || callInfo.l2Call
        l2BridgeFeeInL1Token = callInfo.fee
    }
    const ethereumExecutionFees = await estimateEthereumExecutionFees(
        context,
        registry,
        registry.parachains[`polkadot_${registry.assetHubParaId}`],
        tokenAddress,
        options,
    )
    const rawEthereumExecutionFee = ethereumExecutionFees.executionFee

    let volumeTip: bigint | undefined
    if (options?.volumeFee) {
        volumeTip = calculateVolumeTipInWei(options.volumeFee)
    }
    const tipForScaling = volumeTip ?? 0n

    const scaledGasPad = scaledPadPercentage(
        feePadPercentage,
        tipForScaling,
        rawEthereumExecutionFee,
    )
    let ethereumExecutionFee =
        padFeeByPercentage(rawEthereumExecutionFee, scaledGasPad) + tipForScaling
    const accelerationFee =
        ethereumExecutionFees.accelerationFee > 0n
            ? padFeeByPercentage(ethereumExecutionFees.accelerationFee, scaledGasPad)
            : 0n

    const scaledSlippagePad = scaledPadPercentage(
        feeSlippagePadPercentage,
        tipForScaling,
        rawEthereumExecutionFee,
    )

    // calculate the cost of swapping in native asset
    let totalFeeInNative: bigint | undefined = undefined
    let assetHubExecutionFeeNative: bigint | undefined = undefined
    let returnToSenderExecutionFeeNative: bigint | undefined = undefined
    let ethereumExecutionFeeInNative: bigint | undefined
    let volumeTipInNative: bigint | undefined
    let accelerationFeeInNative: bigint | undefined
    let localExecutionFeeInNative: bigint | undefined
    let feeLocation = options?.feeTokenLocation
    if (feeLocation) {
        // If the fee asset is DOT, then one swap from DOT to Ether is required on AH
        if (isRelaychainLocation(feeLocation)) {
            ethereumExecutionFeeInNative = await assetHubImpl.getAssetHubConversionPalletSwap(
                DOT_LOCATION,
                bridgeLocation(registry.ethChainId),
                padFeeByPercentage(ethereumExecutionFee, scaledSlippagePad),
            )
            if (volumeTip !== undefined && volumeTip > 0n) {
                volumeTipInNative = await assetHubImpl.getAssetHubConversionPalletSwap(
                    DOT_LOCATION,
                    bridgeLocation(registry.ethChainId),
                    volumeTip,
                )
            }
            if (accelerationFee > 0n) {
                accelerationFeeInNative = await assetHubImpl.getAssetHubConversionPalletSwap(
                    DOT_LOCATION,
                    bridgeLocation(registry.ethChainId),
                    accelerationFee,
                )
            }
            totalFeeInDot += ethereumExecutionFeeInNative
            totalFeeInNative = totalFeeInDot
        } else {
            throw new Error("Unsupported fee token location")
        }
    }

    const breakdown: DeliveryFee["breakdown"] = {}
    addBreakdown(breakdown, "localExecution", { amount: localExecutionFeeDOT, symbol: "DOT" })
    addBreakdown(breakdown, "snowbridgeDelivery", {
        amount: snowbridgeDeliveryFeeDOT,
        symbol: "DOT",
    })
    addBreakdown(breakdown, "assetHubExecution", { amount: assetHubExecutionFeeDOT, symbol: "DOT" })
    addBreakdown(breakdown, "bridgeHubDelivery", { amount: bridgeHubDeliveryFeeDOT, symbol: "DOT" })
    addBreakdown(breakdown, "returnToSenderDelivery", {
        amount: returnToSenderDeliveryFeeDOT,
        symbol: "DOT",
    })
    addBreakdown(breakdown, "returnToSenderExecution", {
        amount: returnToSenderExecutionFeeDOT,
        symbol: "DOT",
    })
    addBreakdown(breakdown, "ethereumExecution", {
        amount: ethereumExecutionFee ?? 0n,
        symbol: "ETH",
    })
    const nativeSymbol = feeLocation ? registry.relaychain.tokenSymbols : undefined
    if (ethereumExecutionFeeInNative !== undefined && feeLocation) {
        addBreakdown(breakdown, "ethereumExecution", {
            amount: ethereumExecutionFeeInNative,
            symbol: nativeSymbol!,
        })
    }
    if (l2BridgeFeeInL1Token > 0n) {
        addBreakdown(breakdown, "l2Bridge", { amount: l2BridgeFeeInL1Token, symbol: "ETH" })
    }

    const xcmExecDOT =
        localExecutionFeeDOT + assetHubExecutionFeeDOT + returnToSenderExecutionFeeDOT
    const bridgeFeesDOT =
        snowbridgeDeliveryFeeDOT + bridgeHubDeliveryFeeDOT + returnToSenderDeliveryFeeDOT
    const summary: DeliveryFee["summary"] = []
    if (feeLocation) {
        const tipInNative = volumeTipInNative ?? 0n
        const accelerationInNative = accelerationFeeInNative ?? 0n
        const ethereumExecInNative =
            (ethereumExecutionFeeInNative ?? 0n) - tipInNative - accelerationInNative
        summary.push({
            description: "XCM execution fees",
            amount: xcmExecDOT,
            symbol: nativeSymbol!,
        })
        summary.push({
            description: "Ethereum execution fees",
            amount: ethereumExecInNative,
            symbol: nativeSymbol!,
        })
        summary.push({
            description: "Bridge fees",
            amount: bridgeFeesDOT,
            symbol: nativeSymbol!,
        })
        if (tipInNative > 0n) {
            summary.push({
                description: "Relayer tip",
                amount: tipInNative,
                symbol: nativeSymbol!,
            })
        }
        if (accelerationInNative > 0n) {
            summary.push({
                description: "Acceleration fee",
                amount: accelerationInNative,
                symbol: nativeSymbol!,
            })
        }
    } else {
        summary.push({
            description: "XCM execution fees",
            amount: xcmExecDOT,
            symbol: "DOT",
        })
        summary.push({
            description: "Bridge fees",
            amount: bridgeFeesDOT,
            symbol: "DOT",
        })
        summary.push({
            description: "Ethereum execution fees",
            amount: ethereumExecutionFee - (volumeTip ?? 0n) - accelerationFee,
            symbol: "ETH",
        })
        if (volumeTip !== undefined) {
            summary.push({ description: "Relayer tip", amount: volumeTip, symbol: "ETH" })
        }
        if (accelerationFee > 0n) {
            summary.push({
                description: "Acceleration fee",
                amount: accelerationFee,
                symbol: "ETH",
            })
        }
    }
    if (l2BridgeFeeInL1Token > 0n) {
        summary.push({ description: "Across fee", amount: l2BridgeFeeInL1Token, symbol: "ETH" })
    }

    return {
        kind: l2ChainId ? "polkadot->ethereum_l2" : "polkadot->ethereum",
        feeLocation,
        breakdown,
        summary,
        totals: computeTotals(summary),
    }
}

export const estimateFeesFromParachains = async <T extends EthereumProviderTypes>(
    context: Context<T>,
    sourceParaId: number,
    registry: AssetRegistry,
    tokenAddress: string,
    deliveryXcm: DeliveryXcm,
    options?: {
        padFeeByPercentage?: bigint
        slippagePadPercentage?: bigint
        defaultFee?: bigint
        feeTokenLocation?: any
        contractCall?: ContractCall
        volumeFee?: VolumeFeeParams
        accelerated?: boolean
    },
): Promise<DeliveryFee> => {
    const sourceParachain = registry.parachains[`polkadot_${sourceParaId}`]
    const sourceParachainImpl = await context.paraImplementation(
        await context.parachain(sourceParaId),
    )

    const assetHub = await context.parachain(registry.assetHubParaId)
    const assetHubImpl = await context.paraImplementation(assetHub)

    const feePadPercentage = options?.padFeeByPercentage ?? 33n
    const feeSlippagePadPercentage = options?.slippagePadPercentage ?? 20n

    let localExecutionFeeDOT = 0n
    let localDeliveryFeeDOT = 0n
    let assetHubExecutionFeeDOT = 0n
    let bridgeHubDeliveryFeeDOT = 0n
    let snowbridgeDeliveryFeeDOT = 0n

    let localExecutionFeeInNative: bigint | undefined = undefined
    let localDeliveryFeeInNative: bigint | undefined = undefined
    if (sourceParachain.features.hasDotBalance) {
        localExecutionFeeDOT = padFeeByPercentage(
            await sourceParachainImpl.calculateXcmFee(deliveryXcm.localXcm, DOT_LOCATION),
            feePadPercentage,
        )
        localDeliveryFeeDOT = padFeeByPercentage(
            await sourceParachainImpl.calculateDeliveryFeeInDOT(
                registry.assetHubParaId,
                deliveryXcm.forwardXcmToAH,
            ),
            feePadPercentage,
        )
    } else {
        localExecutionFeeInNative = padFeeByPercentage(
            await sourceParachainImpl.calculateXcmFee(deliveryXcm.localXcm, HERE_LOCATION),
            feePadPercentage,
        )
        localDeliveryFeeInNative = padFeeByPercentage(
            await sourceParachainImpl.calculateDeliveryFeeInNative(
                registry.assetHubParaId,
                deliveryXcm.forwardXcmToAH,
            ),
            feePadPercentage,
        )
    }

    assetHubExecutionFeeDOT = padFeeByPercentage(
        await assetHubImpl.calculateXcmFee(deliveryXcm.forwardXcmToAH, DOT_LOCATION),
        feePadPercentage,
    )

    bridgeHubDeliveryFeeDOT = padFeeByPercentage(
        await assetHubImpl.calculateDeliveryFeeInDOT(
            registry.bridgeHubParaId,
            deliveryXcm.forwardedXcmToBH,
        ),
        feePadPercentage,
    )

    snowbridgeDeliveryFeeDOT = await getSnowbridgeDeliveryFee(assetHub, options?.defaultFee)

    let totalFeeInDot =
        localExecutionFeeDOT +
        localDeliveryFeeDOT +
        snowbridgeDeliveryFeeDOT +
        assetHubExecutionFeeDOT +
        bridgeHubDeliveryFeeDOT

    const ethereumExecutionFees = await estimateEthereumExecutionFees(
        context,
        registry,
        sourceParachain,
        tokenAddress,
        options,
    )
    const rawEthereumExecutionFee = ethereumExecutionFees.executionFee

    let volumeTip: bigint | undefined
    if (options?.volumeFee) {
        volumeTip = calculateVolumeTipInWei(options.volumeFee)
    }
    const tipForScaling = volumeTip ?? 0n

    const scaledGasPad = scaledPadPercentage(
        feePadPercentage,
        tipForScaling,
        rawEthereumExecutionFee,
    )
    let ethereumExecutionFee =
        padFeeByPercentage(rawEthereumExecutionFee, scaledGasPad) + tipForScaling
    const accelerationFee =
        ethereumExecutionFees.accelerationFee > 0n
            ? padFeeByPercentage(ethereumExecutionFees.accelerationFee, scaledGasPad)
            : 0n

    const scaledSlippagePad = scaledPadPercentage(
        feeSlippagePadPercentage,
        tipForScaling,
        rawEthereumExecutionFee,
    )

    // calculate the cost of swapping in native asset
    let totalFeeInNative: bigint | undefined = undefined
    let assetHubExecutionFeeNative: bigint | undefined = undefined
    let ethereumExecutionFeeInNative: bigint | undefined
    let ethereumExecutionFeeInDot: bigint | undefined
    let volumeTipInNative: bigint | undefined
    let accelerationFeeInNative: bigint | undefined
    let feeLocation = options?.feeTokenLocation
    if (feeLocation) {
        // If the fee asset is DOT, then one swap from DOT to Ether is required on AH
        if (isRelaychainLocation(feeLocation)) {
            ethereumExecutionFeeInNative = await assetHubImpl.getAssetHubConversionPalletSwap(
                DOT_LOCATION,
                bridgeLocation(registry.ethChainId),
                padFeeByPercentage(ethereumExecutionFee, scaledSlippagePad),
            )
            if (volumeTip !== undefined && volumeTip > 0n) {
                volumeTipInNative = await assetHubImpl.getAssetHubConversionPalletSwap(
                    DOT_LOCATION,
                    bridgeLocation(registry.ethChainId),
                    volumeTip,
                )
            }
            if (accelerationFee > 0n) {
                accelerationFeeInNative = await assetHubImpl.getAssetHubConversionPalletSwap(
                    DOT_LOCATION,
                    bridgeLocation(registry.ethChainId),
                    accelerationFee,
                )
            }
            totalFeeInDot += ethereumExecutionFeeInNative
            totalFeeInNative = totalFeeInDot
        }
        // On Parachains, we can use their native asset as the fee token.
        // If the fee is in native, we need to swap it to DOT first, then swap DOT to Ether to cover the ethereum execution fee.
        else if (isParachainNative(feeLocation, sourceParaId)) {
            ethereumExecutionFeeInDot = await assetHubImpl.getAssetHubConversionPalletSwap(
                DOT_LOCATION,
                bridgeLocation(registry.ethChainId),
                padFeeByPercentage(ethereumExecutionFee, scaledSlippagePad),
            )
            ethereumExecutionFeeInNative = await assetHubImpl.getAssetHubConversionPalletSwap(
                feeLocation,
                DOT_LOCATION,
                padFeeByPercentage(ethereumExecutionFeeInDot, scaledSlippagePad),
            )
            if (volumeTip !== undefined && volumeTip > 0n) {
                const volumeTipInDOT = await assetHubImpl.getAssetHubConversionPalletSwap(
                    DOT_LOCATION,
                    bridgeLocation(registry.ethChainId),
                    volumeTip,
                )
                volumeTipInNative = await assetHubImpl.getAssetHubConversionPalletSwap(
                    feeLocation,
                    DOT_LOCATION,
                    volumeTipInDOT,
                )
            }
            if (accelerationFee > 0n) {
                const accelerationFeeInDOT = await assetHubImpl.getAssetHubConversionPalletSwap(
                    DOT_LOCATION,
                    bridgeLocation(registry.ethChainId),
                    accelerationFee,
                )
                accelerationFeeInNative = await assetHubImpl.getAssetHubConversionPalletSwap(
                    feeLocation,
                    DOT_LOCATION,
                    accelerationFeeInDOT,
                )
            }
            totalFeeInDot += ethereumExecutionFeeInDot
            totalFeeInNative = await assetHubImpl.getAssetHubConversionPalletSwap(
                feeLocation,
                DOT_LOCATION,
                padFeeByPercentage(totalFeeInDot, feeSlippagePadPercentage),
            )
            if (localExecutionFeeInNative) {
                totalFeeInNative += localExecutionFeeInNative
            }
            if (localDeliveryFeeInNative) {
                totalFeeInNative += localDeliveryFeeInNative
            }
        } else {
            throw new Error("Unsupported fee token location")
        }
    }

    const breakdown: DeliveryFee["breakdown"] = {}
    addBreakdown(breakdown, "localExecution", { amount: localExecutionFeeDOT, symbol: "DOT" })
    addBreakdown(breakdown, "localDelivery", { amount: localDeliveryFeeDOT, symbol: "DOT" })
    addBreakdown(breakdown, "snowbridgeDelivery", {
        amount: snowbridgeDeliveryFeeDOT,
        symbol: "DOT",
    })
    addBreakdown(breakdown, "assetHubExecution", { amount: assetHubExecutionFeeDOT, symbol: "DOT" })
    addBreakdown(breakdown, "bridgeHubDelivery", { amount: bridgeHubDeliveryFeeDOT, symbol: "DOT" })
    addBreakdown(breakdown, "ethereumExecution", { amount: ethereumExecutionFee, symbol: "ETH" })
    const sourceParaSymbol = sourceParachain.info.tokenSymbols
    const feeNativeSymbol = feeLocation
        ? isRelaychainLocation(feeLocation)
            ? registry.relaychain.tokenSymbols
            : sourceParaSymbol
        : undefined
    if (localExecutionFeeInNative !== undefined) {
        addBreakdown(breakdown, "localExecution", {
            amount: localExecutionFeeInNative,
            symbol: sourceParaSymbol,
        })
    }
    if (localDeliveryFeeInNative !== undefined) {
        addBreakdown(breakdown, "localDelivery", {
            amount: localDeliveryFeeInNative,
            symbol: sourceParaSymbol,
        })
    }
    if (ethereumExecutionFeeInNative !== undefined) {
        addBreakdown(breakdown, "ethereumExecution", {
            amount: ethereumExecutionFeeInNative,
            symbol: feeNativeSymbol!,
        })
    }
    // Native-fee path goes native → DOT → ETH on AH. Persist the slippage-padded
    // DOT input used to quote the DOT → ETH swap so reserve validation reconstructs
    // the same DOT requirement fee estimation used (rather than re-quoting from
    // unpadded ETH and getting a less conservative threshold).
    if (ethereumExecutionFeeInDot !== undefined && feeNativeSymbol !== "DOT") {
        addBreakdown(breakdown, "ethereumExecution", {
            amount: ethereumExecutionFeeInDot,
            symbol: "DOT",
        })
    }

    const xcmExecDOT = localExecutionFeeDOT + assetHubExecutionFeeDOT
    const bridgeFeesDOT = snowbridgeDeliveryFeeDOT + bridgeHubDeliveryFeeDOT + localDeliveryFeeDOT
    const summary: DeliveryFee["summary"] = []
    if (feeLocation) {
        const tipInNative = volumeTipInNative ?? 0n
        const accelerationInNative = accelerationFeeInNative ?? 0n
        if (isRelaychainLocation(feeLocation)) {
            const ethereumExecInNative =
                (ethereumExecutionFeeInNative ?? 0n) - tipInNative - accelerationInNative
            summary.push({
                description: "XCM execution fees",
                amount: xcmExecDOT,
                symbol: feeNativeSymbol!,
            })
            summary.push({
                description: "Ethereum execution fees",
                amount: ethereumExecInNative,
                symbol: feeNativeSymbol!,
            })
            summary.push({
                description: "Bridge fees",
                amount: bridgeFeesDOT,
                symbol: feeNativeSymbol!,
            })
            if (tipInNative > 0n) {
                summary.push({
                    description: "Relayer tip",
                    amount: tipInNative,
                    symbol: feeNativeSymbol!,
                })
            }
            if (accelerationInNative > 0n) {
                summary.push({
                    description: "Acceleration fee",
                    amount: accelerationInNative,
                    symbol: feeNativeSymbol!,
                })
            }
        } else {
            // parachain-native pay: split totalFeeInNative across categories using
            // proportional share of the DOT-side amounts. Sum is preserved exactly.
            const localExecN = localExecutionFeeInNative ?? 0n
            const localDelivN = localDeliveryFeeInNative ?? 0n
            const ethExecN =
                (ethereumExecutionFeeInNative ?? 0n) - tipInNative - accelerationInNative
            const otherN =
                totalFeeInNative! - localExecN - localDelivN - (ethereumExecutionFeeInNative ?? 0n)
            const totalDotOnly = xcmExecDOT + bridgeFeesDOT
            let xcmExecPortion = 0n
            let bridgeFeesPortion = 0n
            if (totalDotOnly > 0n) {
                xcmExecPortion = (otherN * xcmExecDOT) / totalDotOnly
                bridgeFeesPortion = otherN - xcmExecPortion
            } else {
                xcmExecPortion = otherN
            }
            summary.push({
                description: "XCM execution fees",
                amount: xcmExecPortion + localExecN,
                symbol: feeNativeSymbol!,
            })
            summary.push({
                description: "Ethereum execution fees",
                amount: ethExecN,
                symbol: feeNativeSymbol!,
            })
            summary.push({
                description: "Bridge fees",
                amount: bridgeFeesPortion + localDelivN,
                symbol: feeNativeSymbol!,
            })
            if (tipInNative > 0n) {
                summary.push({
                    description: "Relayer tip",
                    amount: tipInNative,
                    symbol: feeNativeSymbol!,
                })
            }
            if (accelerationInNative > 0n) {
                summary.push({
                    description: "Acceleration fee",
                    amount: accelerationInNative,
                    symbol: feeNativeSymbol!,
                })
            }
        }
    } else {
        summary.push({
            description: "XCM execution fees",
            amount: xcmExecDOT,
            symbol: "DOT",
        })
        summary.push({
            description: "Bridge fees",
            amount: bridgeFeesDOT,
            symbol: "DOT",
        })
        if (localExecutionFeeInNative !== undefined && localExecutionFeeInNative > 0n) {
            summary.push({
                description: "XCM execution fees",
                amount: localExecutionFeeInNative,
                symbol: sourceParaSymbol,
            })
        }
        if (localDeliveryFeeInNative !== undefined && localDeliveryFeeInNative > 0n) {
            summary.push({
                description: "Bridge fees",
                amount: localDeliveryFeeInNative,
                symbol: sourceParaSymbol,
            })
        }
        summary.push({
            description: "Ethereum execution fees",
            amount: ethereumExecutionFee - (volumeTip ?? 0n) - accelerationFee,
            symbol: "ETH",
        })
        if (volumeTip !== undefined) {
            summary.push({ description: "Relayer tip", amount: volumeTip, symbol: "ETH" })
        }
        if (accelerationFee > 0n) {
            summary.push({
                description: "Acceleration fee",
                amount: accelerationFee,
                symbol: "ETH",
            })
        }
    }

    return {
        kind: "polkadot->ethereum",
        feeLocation,
        breakdown,
        summary,
        totals: computeTotals(summary),
    }
}

export const validateTransferFromAssetHub = async <T extends EthereumProviderTypes>(
    context: Context<T>,
    transfer: Transfer,
): Promise<ValidatedTransfer> => {
    const { registry, fee, tokenAddress, amount } = transfer.input
    const { sourceAccountHex, sourceParaId, sourceAssetMetadata } = transfer.computed
    const { tx } = transfer

    const { sourceParachain, gateway, ethereum, bridgeHub } =
        context instanceof Context
            ? {
                  sourceParachain: await context.parachain(sourceParaId),
                  gateway: context.gateway(),
                  ethereum: context.ethereum(),
                  bridgeHub: await context.bridgeHub(),
              }
            : context

    const logs: ValidationLog[] = []
    const sourceParachainImpl = await context.paraImplementation(sourceParachain)
    const nativeBalance = await sourceParachainImpl.getNativeBalance(sourceAccountHex, true)
    let dotBalance = await sourceParachainImpl.getDotBalance(sourceAccountHex)
    let tokenBalance: any
    let isNativeBalance = false
    // For DOT on AH, get it from the native balance pallet.
    if (
        transfer.computed.sourceAssetMetadata.location &&
        isRelaychainLocation(transfer.computed.sourceAssetMetadata.location)
    ) {
        tokenBalance = await sourceParachainImpl.getNativeBalance(sourceAccountHex, true)
        isNativeBalance = true
    } else {
        tokenBalance = await sourceParachainImpl.getTokenBalance(
            sourceAccountHex,
            registry.ethChainId,
            tokenAddress,
            sourceAssetMetadata,
        )
    }
    const ahTotalNative = findTotalOrUndefined(fee, "DOT")
    if (isNativeBalance && ahTotalNative !== undefined) {
        if (amount + ahTotalNative > tokenBalance) {
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.InsufficientTokenBalance,
                message: "Insufficient token balance to submit transaction.",
            })
        }
    } else {
        if (amount > tokenBalance) {
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.InsufficientTokenBalance,
                message: "Insufficient token balance to submit transaction.",
            })
        }
    }

    // No fee specified means that the fee.ethereumExecutionFee is paid in Ether on source chain.
    if (!fee.feeLocation) {
        let etherBalance = await sourceParachainImpl.getTokenBalance(
            sourceAccountHex,
            registry.ethChainId,
            ETHER_TOKEN_ADDRESS,
        )

        if (findInBreakdownOrZero(fee.breakdown, "ethereumExecution", "ETH") > etherBalance) {
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.InsufficientEtherBalance,
                message: "Insufficient ether balance to submit transaction.",
            })
        }
    }
    let contractCall = transfer.input.contractCall
    if (contractCall) {
        const isContractAddress = await context.ethereumProvider.isContractAddress(
            ethereum,
            contractCall.target,
        )
        if (!isContractAddress) {
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.ContractCallInvalidTarget,
                message: "Contract call with invalid target address: " + contractCall.target,
            })
        }
        try {
            let agentAddress = await sourceAgentAddress(context, sourceParaId, sourceAccountHex)
        } catch (error) {
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.ContractCallAgentNotRegistered,
                message:
                    "Contract call cannot be performed because no agent is registered for source account: " +
                    sourceAccountHex +
                    " error: " +
                    String(error),
            })
        }
    }

    let sourceDryRunError
    let assetHubDryRunError
    let bridgeHubDryRunError
    let ethereumDryRunError
    let estimatedDryRunGas: bigint | undefined
    // do the dry run, get the forwarded xcm and dry run that
    const dryRunResultAssetHub = await dryRunOnSourceParachain(
        sourceParachain,
        registry.assetHubParaId,
        registry.bridgeHubParaId,
        transfer.tx,
        sourceAccountHex,
    )
    if (dryRunResultAssetHub.success && dryRunResultAssetHub.bridgeHubForwarded) {
        const dryRunResultBridgeHub = await dryRunBridgeHub(
            bridgeHub,
            registry.assetHubParaId,
            dryRunResultAssetHub.bridgeHubForwarded[1][0],
        )
        if (dryRunResultBridgeHub.success) {
            try {
                const ethereumTx = await buildEthereumDryRunCall(
                    context,
                    registry.assetHubParaId,
                    sourceAccountHex,
                    transfer,
                )
                estimatedDryRunGas = await context.ethereumProvider.estimateGas(ethereum, ethereumTx)
                const minDispatchGas = computeDryRunDispatchGasLimit(
                    dryRunCommandGasBudgets(transfer),
                )
                const txGasLimit = (estimatedDryRunGas! * 2n) > minDispatchGas
                    ? estimatedDryRunGas! * 2n
                    : minDispatchGas
                
                try {
                    const forkedProvider = context.ethereumProvider.createProvider(
                        process.env.FORKED_PROVIDER_URL ||
                            process.env.NEXT_PUBLIC_FORKED_PROVIDER_URL ||
                            "http://localhost:8545",
                    ) as unknown as ForkedRpcProvider
                    try {
                        await forkedProvider.send("eth_blockNumber", [])
                        const txRequest = {
                            from: (ethereumTx as EthereumDryRunTx).from,
                            to: (ethereumTx as EthereumDryRunTx).to,
                            data: (ethereumTx as EthereumDryRunTx).data,
                            value: (ethereumTx as EthereumDryRunTx).value ?? "0x0",
                            gas: "0x" + txGasLimit.toString(16),
                        }

                        let txHash: string
                        try {
                            txHash = await forkedProvider.send("eth_sendTransaction", [txRequest])
                        } catch (sendError) {
                            const sendErrorMessage = String((sendError as Error).message || sendError)
                            const shouldImpersonate =
                                sendErrorMessage.includes("No Signer available") ||
                                sendErrorMessage.includes("unknown account")
                            const impersonated = shouldImpersonate
                                ? await tryImpersonateForkedSigner(forkedProvider, txRequest.from)
                                : false
                            if (!impersonated) {
                                throw sendError
                            }
                            txHash = await forkedProvider.send("eth_sendTransaction", [txRequest])
                        }

                        console.log("Tx hash:", txHash)

                        const receipt = await forkedProvider.waitForTransaction(txHash)
                        if (!receipt) {
                            ethereumDryRunError =
                                "Dry run transaction simulation on forked Ethereum did not return a receipt (node may be unavailable/out of sync)."
                            logs.push({
                                kind: ValidationKind.Warning,
                                reason: ValidationReason.DryRunFailed,
                                message: ethereumDryRunError,
                            })
                        } else {
                            const receiptStatus = receipt.status
                            const isRevertedReceipt =
                                receiptStatus === 0 ||
                                receiptStatus === 0n ||
                                receiptStatus === "0x0" ||
                                receiptStatus === "0"
                            if (isRevertedReceipt) {
                                ethereumDryRunError =
                                    "Dry run transaction simulation reverted on Ethereum (receipt status 0)."
                                logs.push({
                                    kind: ValidationKind.Error,
                                    reason: ValidationReason.DryRunFailed,
                                    message: ethereumDryRunError,
                                })
                            }

                            console.log("Logs:", receipt.logs)
                            const parsedLogs = receipt.logs
                                .map((log: any) => {
                                    try {
                                        return context.gatewayProxy().interface.parseLog(log)
                                    } catch (e) {
                                        return null
                                    }
                                })
                                .filter((log: any) => log !== null)
                            const errorLogs = parsedLogs.filter((log: any) => {
                                return log.name === "CommandFailed"
                            })
                            if (errorLogs.length > 0) {
                                const failedIndex = errorLogs[0]?.args?.index
                                ethereumDryRunError =
                                    "Dry run v2_dispatch simulation reported CommandFailed at index: " +
                                    String(failedIndex)
                                logs.push({
                                    kind: ValidationKind.Warning,
                                    reason: ValidationReason.DryRunFailed,
                                    message: ethereumDryRunError,
                                })
                            }

                            const l1AdapterAddress = context.environment.l2Bridge?.l1AdapterAddress
                            const adaptorEvents = receipt.logs
                                .map((log: any) => parseL1AdaptorDryRunEvent(log, l1AdapterAddress))
                                .filter(
                                    (
                                        event,
                                    ): event is {
                                        name: "DepositCallInvoked" | "DepositCallFailed"
                                        topic?: string
                                        depositId?: bigint
                                    } => event !== null,
                                )
                            const depositCallFailed = adaptorEvents.filter((event) => {
                                return event.name === "DepositCallFailed"
                            })
                            if (depositCallFailed.length > 0) {
                                const topic = depositCallFailed[0]?.topic
                                ethereumDryRunError =
                                    "Dry run failed on Ethereum: L1 adaptor emitted DepositCallFailed" +
                                    (topic ? ` (topic: ${topic})` : "")
                                logs.push({
                                    kind: ValidationKind.Error,
                                    reason: ValidationReason.DryRunFailed,
                                    message: ethereumDryRunError,
                                })
                            }

                            const depositCallInvoked = adaptorEvents.find((event) => {
                                return event.name === "DepositCallInvoked"
                            })
                            if (depositCallInvoked) {
                                console.log("L1 adaptor event:", {
                                    name: depositCallInvoked.name,
                                    topic: depositCallInvoked.topic,
                                    depositId: depositCallInvoked.depositId?.toString(),
                                })
                            } else if (errorLogs.length > 0 && adaptorEvents.length === 0) {
                                logs.push({
                                    kind: ValidationKind.Warning,
                                    reason: ValidationReason.DryRunFailed,
                                    message:
                                        "Synthetic v2_dispatch dry run reported upstream failure before adaptor events; this may diverge from proof-based production execution.",
                                })
                            }
                        }
                    } catch (forkUnavailableError) {
                        ethereumDryRunError =
                            "Skipping Ethereum dry-run transaction simulation because the forked RPC node is unavailable: " +
                            String(
                                (forkUnavailableError as Error).message ||
                                    forkUnavailableError,
                            )
                        logs.push({
                            kind: ValidationKind.Warning,
                            reason: ValidationReason.DryRunFailed,
                            message: ethereumDryRunError,
                        })
                    }
                } catch (e) {
                    ethereumDryRunError =
                        "Dry run transaction simulation failed on Ethereum." +
                        (e as Error).message
                    logs.push({
                        kind: ValidationKind.Warning,
                        reason: ValidationReason.DryRunFailed,
                        message: ethereumDryRunError,
                    })
                }
            } catch (e) {
                ethereumDryRunError = "Could not estimate gas on Ethereum." + (e as Error).message
                logs.push({
                    kind: ValidationKind.Error,
                    reason: ValidationReason.FeeEstimationError,
                    message: ethereumDryRunError,
                })
            }
        } else {
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.DryRunFailed,
                message: "Dry run failed on Bridge Hub.",
            })
            bridgeHubDryRunError = dryRunResultBridgeHub.errorMessage
        }
    } else {
        logs.push({
            kind: ValidationKind.Error,
            reason: ValidationReason.DryRunFailed,
            message: "Dry run call failed on Asset Hub.",
        })
        assetHubDryRunError = dryRunResultAssetHub.error
    }

    const paymentInfo = await tx.paymentInfo(sourceAccountHex)
    const sourceExecutionFee = paymentInfo["partialFee"].toBigInt()

    // recheck total after fee estimation
    if (isNativeBalance && ahTotalNative !== undefined) {
        if (amount + ahTotalNative + sourceExecutionFee > tokenBalance) {
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.InsufficientTokenBalance,
                message: "Insufficient token balance to submit transaction.",
            })
        }
    }
    if (sourceExecutionFee + findTotal(fee, "DOT") > dotBalance) {
        logs.push({
            kind: ValidationKind.Error,
            reason: ValidationReason.InsufficientDotFee,
            message: "Insufficient DOT balance to submit transaction on the source parachain.",
        })
    }
    const bridgeStatus = await getOperatingStatus({
        ethereumProvider: context.ethereumProvider,
        gateway,
        bridgeHub,
    })
    if (bridgeStatus.toEthereum.outbound !== "Normal") {
        logs.push({
            kind: ValidationKind.Error,
            reason: ValidationReason.BridgeStatusNotOperational,
            message: "Bridge operations have been paused by onchain governance.",
        })
    }

    if (fee.feeLocation) {
        const requiredEthOut = findInBreakdownOrZero(fee.breakdown, "ethereumExecution", "ETH")
        if (requiredEthOut > 0n) {
            const reserveCheck = await checkDotEthPoolLiquidityForPolkadotToEthereum(
                sourceParachainImpl,
                registry.ethChainId,
                requiredEthOut,
            )
            if (!reserveCheck.ok) {
                logs.push({
                    kind: ValidationKind.Error,
                    reason: ValidationReason.InsufficientPoolReserves,
                    message:
                        reserveCheck.reason === "pool-missing"
                            ? `${reserveCheck.pool} pool does not exist on Asset Hub.`
                            : `${reserveCheck.pool} pool on Asset Hub has insufficient liquidity (need ${reserveCheck.requiredOut}, have ${reserveCheck.reserveOut}).`,
                })
            }
        }
    }

    const success = logs.find((l) => l.kind === ValidationKind.Error) === undefined

    return {
        logs,
        success,
        data: {
            bridgeStatus,
            nativeBalance,
            dotBalance,
            sourceExecutionFee,
            tokenBalance,
            sourceDryRunError,
            assetHubDryRunError,
            bridgeHubDryRunError,
            ethereumDryRunError,
        },
        ...transfer,
    }
}

export const validateTransferFromParachain = async <T extends EthereumProviderTypes>(
    context: Context<T>,
    transfer: Transfer,
): Promise<ValidatedTransfer> => {
    const { registry, fee, tokenAddress, amount } = transfer.input
    const {
        sourceAccountHex,
        sourceParaId,
        sourceParachain: source,
        sourceAssetMetadata,
    } = transfer.computed
    const { tx } = transfer

    const { sourceParachain, gateway, ethereum, bridgeHub, assetHub } =
        context instanceof Context
            ? {
                  sourceParachain: await context.parachain(sourceParaId),
                  gateway: context.gateway(),
                  ethereum: context.ethereum(),
                  bridgeHub: await context.bridgeHub(),
                  assetHub: await context.assetHub(),
              }
            : context

    const logs: ValidationLog[] = []
    const sourceParachainImpl = await context.paraImplementation(sourceParachain)
    const nativeBalance = await sourceParachainImpl.getNativeBalance(sourceAccountHex, true)
    let dotBalance: bigint | undefined = undefined
    if (source.features.hasDotBalance) {
        dotBalance = await sourceParachainImpl.getDotBalance(sourceAccountHex)
    }
    const paymentInfo = await tx.paymentInfo(sourceAccountHex)
    const sourceExecutionFee = paymentInfo["partialFee"].toBigInt()
    let tokenBalance: any
    let isNativeBalance = false

    isNativeBalance =
        sourceAssetMetadata.decimals === source.info.tokenDecimals &&
        sourceAssetMetadata.symbol == source.info.tokenSymbols
    if (isNativeBalance) {
        tokenBalance = await sourceParachainImpl.getNativeBalance(sourceAccountHex, true)
    } else {
        tokenBalance = await sourceParachainImpl.getTokenBalance(
            sourceAccountHex,
            registry.ethChainId,
            tokenAddress,
            sourceAssetMetadata,
        )
    }

    // The source `WithdrawAsset` may withdraw extra units of the token asset
    // when the token shares a location with the fee asset:
    //  - Native-ETH transfer paid with ETH (default builder): `amount + remoteEtherFeeAmount` of ETH.
    //  - PNA DOT transfer paid with DOT: `amount + totalDOTFeeAmount` of DOT.
    // Account for these overlaps so the token-balance check matches what the
    // extrinsic actually withdraws.
    const isEthToken = tokenAddress.toLowerCase() === ETHER_TOKEN_ADDRESS.toLowerCase()
    const isAllEthFeePath = !fee.feeLocation
    const isDotToken =
        sourceAssetMetadata.location !== undefined &&
        isRelaychainLocation(sourceAssetMetadata.location)
    const isNativeFeePath =
        fee.feeLocation !== undefined && isParachainNative(fee.feeLocation, sourceParaId)
    const sourceWithdrawsDot = isAllEthFeePath || (!!fee.feeLocation && !isNativeFeePath)
    const requiredDotFee = findTotalOrUndefined(fee, "DOT") ?? 0n

    const extraEthOnSourceWithdraw =
        isEthToken && isAllEthFeePath
            ? findInBreakdownOrZero(fee.breakdown, "ethereumExecution", "ETH")
            : 0n
    const extraDotOnSourceWithdraw = isDotToken && sourceWithdrawsDot ? requiredDotFee : 0n
    const requiredTokenAmount = amount + extraEthOnSourceWithdraw + extraDotOnSourceWithdraw

    const paraTotalNative = findTotalOrUndefined(fee, source.info.tokenSymbols)
    // When the bridged token is the parachain's native asset, the substrate
    // tx fee is withdrawn from the same balance, so include it in the threshold.
    const nativeTxFeeShare = isNativeBalance ? sourceExecutionFee : 0n
    if (isNativeBalance && paraTotalNative !== undefined) {
        if (requiredTokenAmount + paraTotalNative + nativeTxFeeShare > tokenBalance) {
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.InsufficientTokenBalance,
                message: "Insufficient token balance to submit transaction.",
            })
        }
    } else {
        if (requiredTokenAmount + nativeTxFeeShare > tokenBalance) {
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.InsufficientTokenBalance,
                message: "Insufficient token balance to submit transaction.",
            })
        }
    }

    // When the token is ETH and the fee path is ETH, the eth-side fee is folded
    // into the token-balance check above. Skip the standalone ether check to
    // avoid querying the same balance twice and emitting a duplicate error.
    if (!fee.feeLocation && !isEthToken) {
        let etherBalance = await sourceParachainImpl.getTokenBalance(
            sourceAccountHex,
            registry.ethChainId,
            ETHER_TOKEN_ADDRESS,
        )

        // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
        if (findInBreakdownOrZero(fee.breakdown, "ethereumExecution", "ETH") > etherBalance) {
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.InsufficientEtherBalance,
                message: "Insufficient ether balance to submit transaction.",
            })
        }
    }

    // DOT balance check: the source `WithdrawAsset` includes `totalDOTFeeAmount`
    // for the default ETH-fee and DOT-fee paths. Skip when the token is DOT —
    // the overlap is already captured in the token-balance check via `extraDotOnSourceWithdraw`.
    if (
        sourceWithdrawsDot &&
        !isDotToken &&
        source.features.hasDotBalance &&
        dotBalance !== undefined &&
        requiredDotFee > dotBalance
    ) {
        logs.push({
            kind: ValidationKind.Error,
            reason: ValidationReason.InsufficientDotFee,
            message: "Insufficient DOT balance to submit transaction on the source parachain.",
        })
    }

    // Native balance must cover the substrate tx fee plus, on the native-fee path,
    // the `totalNativeFeeAmount` withdrawn by the source XCM. Skip when the token
    // is the parachain's native asset — both costs are folded into the
    // token-balance check above.
    if (!isNativeBalance) {
        const nativeFeeShare = isNativeFeePath ? (paraTotalNative ?? 0n) : 0n
        if (sourceExecutionFee + nativeFeeShare > nativeBalance) {
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.InsufficientNativeFee,
                message:
                    "Insufficient native balance to submit transaction on the source parachain.",
            })
        }
    }

    let contractCall = transfer.input.contractCall
    if (contractCall) {
        const isContractAddress = await context.ethereumProvider.isContractAddress(
            ethereum,
            contractCall.target,
        )
        if (!isContractAddress) {
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.ContractCallInvalidTarget,
                message: "Contract call with invalid target address: " + contractCall.target,
            })
        }
    }

    let sourceDryRunError
    let assetHubDryRunError
    let bridgeHubDryRunError
    if (source.features.hasDryRunApi) {
        // do the dry run, get the forwarded xcm and dry run that
        const dryRunSource = await dryRunOnSourceParachain(
            sourceParachain,
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
            sourceDryRunError = dryRunSource.error
        }

        if (dryRunSource.success) {
            if (!dryRunSource.assetHubForwarded) {
                logs.push({
                    kind: ValidationKind.Error,
                    reason: ValidationReason.DryRunFailed,
                    message: "Dry run call did not provide a forwarded xcm.",
                })
            } else {
                const dryRunResultAssetHub = await dryRunAssetHub(
                    assetHub,
                    sourceParaId,
                    registry.bridgeHubParaId,
                    dryRunSource.assetHubForwarded[1][0],
                )
                if (dryRunResultAssetHub.success && dryRunResultAssetHub.bridgeHubForwarded) {
                    const dryRunResultBridgeHub = await dryRunBridgeHub(
                        bridgeHub,
                        registry.assetHubParaId,
                        dryRunResultAssetHub.bridgeHubForwarded[1][0],
                    )
                    if (!dryRunResultBridgeHub.success) {
                        logs.push({
                            kind: ValidationKind.Error,
                            reason: ValidationReason.DryRunFailed,
                            message: "Dry run failed on Bridge Hub.",
                        })
                        bridgeHubDryRunError = dryRunResultBridgeHub.errorMessage
                    }
                } else {
                    logs.push({
                        kind: ValidationKind.Error,
                        reason: ValidationReason.DryRunFailed,
                        message: "Dry run failed on Asset Hub.",
                    })
                    assetHubDryRunError = dryRunResultAssetHub.errorMessage
                }
            }
        }
    }

    const bridgeStatus = await getOperatingStatus({
        ethereumProvider: context.ethereumProvider,
        gateway,
        bridgeHub,
    })
    if (bridgeStatus.toEthereum.outbound !== "Normal") {
        logs.push({
            kind: ValidationKind.Error,
            reason: ValidationReason.BridgeStatusNotOperational,
            message: "Bridge operations have been paused by onchain governance.",
        })
    }

    if (fee.feeLocation) {
        const assetHubImpl = await context.paraImplementation(assetHub)
        const requiredEthOut = findInBreakdownOrZero(fee.breakdown, "ethereumExecution", "ETH")
        if (requiredEthOut > 0n) {
            const reserveCheck = await checkDotEthPoolLiquidityForPolkadotToEthereum(
                assetHubImpl,
                registry.ethChainId,
                requiredEthOut,
            )
            if (!reserveCheck.ok) {
                logs.push({
                    kind: ValidationKind.Error,
                    reason: ValidationReason.InsufficientPoolReserves,
                    message:
                        reserveCheck.reason === "pool-missing"
                            ? `${reserveCheck.pool} pool does not exist on Asset Hub.`
                            : `${reserveCheck.pool} pool on Asset Hub has insufficient liquidity (need ${reserveCheck.requiredOut}, have ${reserveCheck.reserveOut}).`,
                })
            }

            // Native-fee path runs an extra native → DOT swap on AH before DOT → ETH,
            // so the <native>/DOT pool must also have enough DOT to cover all DOT-side
            // fees plus the DOT amount fed into the DOT → ETH swap. Use the padded
            // ethereumExecution DOT entry persisted by fee estimation so the threshold
            // matches the value used to quote the native → DOT swap.
            if (isParachainNative(fee.feeLocation, sourceParaId)) {
                const requiredDotOut =
                    findInBreakdownOrZero(fee.breakdown, "localExecution", "DOT") +
                    findInBreakdownOrZero(fee.breakdown, "localDelivery", "DOT") +
                    findInBreakdownOrZero(fee.breakdown, "snowbridgeDelivery", "DOT") +
                    findInBreakdownOrZero(fee.breakdown, "assetHubExecution", "DOT") +
                    findInBreakdownOrZero(fee.breakdown, "bridgeHubDelivery", "DOT") +
                    findInBreakdownOrZero(fee.breakdown, "ethereumExecution", "DOT")
                const nativeReserveCheck = await checkNativeDotPoolLiquidityForParachainToEthereum(
                    assetHubImpl,
                    fee.feeLocation,
                    requiredDotOut,
                    source.info.tokenSymbols,
                )
                if (!nativeReserveCheck.ok) {
                    logs.push({
                        kind: ValidationKind.Error,
                        reason: ValidationReason.InsufficientPoolReserves,
                        message:
                            nativeReserveCheck.reason === "pool-missing"
                                ? `${nativeReserveCheck.pool} pool does not exist on Asset Hub.`
                                : `${nativeReserveCheck.pool} pool on Asset Hub has insufficient liquidity (need ${nativeReserveCheck.requiredOut}, have ${nativeReserveCheck.reserveOut}).`,
                    })
                }
            }
        }
    }

    const success = logs.find((l) => l.kind === ValidationKind.Error) === undefined

    return {
        logs,
        success,
        data: {
            bridgeStatus,
            nativeBalance,
            dotBalance,
            sourceExecutionFee,
            tokenBalance,
            sourceDryRunError,
            assetHubDryRunError,
            bridgeHubDryRunError,
        },
        ...transfer,
    }
}

export async function buildContractCallHex<T extends EthereumProviderTypes>(
    context: Context<T>,
    contractCall: ContractCall,
) {
    const bridgeHub = await context.bridgeHub()
    const callHex = bridgeHub.createType("ContractCall", {
        target: contractCall.target,
        calldata: contractCall.calldata,
        value: contractCall.value,
        gas: contractCall.gas,
    })
    return "0x00" + callHex.toHex().slice(2)
}

// Breakdown entries are 1n placeholders so XCM builders that read fee amounts
// via findInBreakdownOrZero produce non-zero fungible assets — the AH runtime
// panics in XcmPaymentApi_query_xcm_weight on zero-amount assets.
export const mockDeliveryFee: DeliveryFee = {
    kind: "polkadot->ethereum",
    breakdown: {
        localExecution: [{ amount: 1n, symbol: "DOT" }],
        localDelivery: [{ amount: 1n, symbol: "DOT" }],
        snowbridgeDelivery: [{ amount: 1n, symbol: "DOT" }],
        assetHubExecution: [{ amount: 1n, symbol: "DOT" }],
        bridgeHubDelivery: [{ amount: 1n, symbol: "DOT" }],
        ethereumExecution: [{ amount: 1n, symbol: "ETH" }],
    },
    summary: [{ description: "Bridge fee", amount: 10n, symbol: "DOT" }],
    totals: [{ amount: 10n, symbol: "DOT" }],
}

// Agent creation exports
export type {
    AgentCreation,
    ValidatedCreateAgent,
    AgentCreationInterface,
} from "./registration/agent/agentInterface"

export async function buildL2Call<T extends EthereumProviderTypes>(
    context: Context<T>,
    registry: AssetRegistry,
    tokenAddress: string,
    l2ChainId: number,
    tokenAmount: bigint,
    destinationAddress: string,
    topic: string,
    options?: {
        l2TransferGasLimit?: bigint
        l2PadFeeByPercentage?: bigint
        fillDeadlineBuffer?: bigint
    },
): Promise<{ fee: bigint; l2Call: ContractCall }> {
    // Calculate fee with Across SDK
    const l2TokenAddress = findL2TokenAddress(registry, l2ChainId, tokenAddress)
    if (!l2TokenAddress) {
        throw new Error("L2 token address not found")
    }
    const acrossApiUrl = context.environment.l2Bridge?.acrossAPIUrl
    if (!acrossApiUrl) {
        throw new Error("L2 bridge configuration is missing.")
    }
    const l1AdapterAddress = context.environment.l2Bridge?.l1AdapterAddress
    if (!l1AdapterAddress) {
        throw new Error("L2 bridge configuration is missing.")
    }
    let l2BridgeFeeInL1Token: bigint
    let l2Call: ContractCall
    if (tokenAddress === ETHER_TOKEN_ADDRESS) {
        const l1FeeTokenAddress = context.environment.l2Bridge?.l1FeeTokenAddress
        const l2FeeTokenAddress = context.environment.l2Bridge?.l2Chains[l2ChainId]?.feeTokenAddress
        if (!l1FeeTokenAddress || !l2FeeTokenAddress) {
            throw new Error("L2 chain configuration is missing.")
        }
        l2BridgeFeeInL1Token = padFeeByPercentage(
            await estimateFees(
                acrossApiUrl,
                l1FeeTokenAddress,
                l2FeeTokenAddress,
                registry.ethChainId,
                l2ChainId,
                tokenAmount,
            ),
            options?.l2PadFeeByPercentage ?? 33n,
        )
        const calldata = context.ethereumProvider.l1AdapterDepositNativeEther(
            {
                inputToken: tokenAddress,
                outputToken: l2FeeTokenAddress,
                inputAmount: tokenAmount,
                outputAmount: tokenAmount - l2BridgeFeeInL1Token,
                destinationChainId: l2ChainId,
                fillDeadlineBuffer: options?.fillDeadlineBuffer ?? 600n,
            },
            destinationAddress,
            topic,
        )
        l2Call = {
            target: l1AdapterAddress,
            value: 0n,
            gas: options?.l2TransferGasLimit || 500_000n,
            calldata,
        }
    } else {
        l2BridgeFeeInL1Token = padFeeByPercentage(
            await estimateFees(
                acrossApiUrl,
                tokenAddress,
                l2TokenAddress,
                registry.ethChainId,
                l2ChainId,
                tokenAmount,
            ),
            options?.l2PadFeeByPercentage ?? 33n,
        )
        const calldata = context.ethereumProvider.l1AdapterDepositToken(
            {
                inputToken: tokenAddress,
                outputToken: l2TokenAddress,
                inputAmount: tokenAmount,
                outputAmount: tokenAmount - l2BridgeFeeInL1Token,
                destinationChainId: l2ChainId,
                fillDeadlineBuffer: options?.fillDeadlineBuffer ?? 600n,
            },
            destinationAddress,
            topic,
        )
        l2Call = {
            target: l1AdapterAddress,
            value: 0n,
            gas: options?.l2TransferGasLimit || 500_000n,
            calldata,
        }
    }
    return { l2Call, fee: l2BridgeFeeInL1Token }
}

export async function sourceAgentId<T extends EthereumProviderTypes>(
    context: Context<T>,
    parachainId: number,
    sourceAccountHex: string,
) {
    const bridgeHub = await context.bridgeHub()
    let sourceLocation = {
        parents: 1,
        interior: { x2: [{ parachain: parachainId }, { accountId32: { id: sourceAccountHex } }] },
    }
    let versionedLocation = bridgeHub.registry.createType("XcmVersionedLocation", {
        v5: sourceLocation,
    })
    return (await bridgeHub.call.controlV2Api.agentId(versionedLocation)).toHex()
}

export async function sourceAgentAddress<T extends EthereumProviderTypes>(
    context: Context<T>,
    parachainId: number,
    sourceAccountHex: string,
): Promise<string> {
    const gateway = context.gateway()
    let agentID = await sourceAgentId(context, parachainId, sourceAccountHex)
    let agentAddress = await gateway.agentOf(agentID)
    return agentAddress
}

export async function buildEthereumDryRunCall<T extends EthereumProviderTypes>(
    context: Context<T>,
    parachainId: number,
    sourceAccountHex: string,
    transfer: Transfer,
): Promise<T["ContractTransaction"]> {
    let commands: V2CommandStruct[] = []
    const agentID = await sourceAgentId(context, parachainId, sourceAccountHex)
    if (transfer.computed.sourceAssetMetadata.foreignId) {
        // PNA
        const mintForeignParams = encodeAbiTuple(
            context,
            ["bytes32", "address", "uint128"],
            [
                transfer.computed.sourceAssetMetadata.foreignId,
                transfer.input.beneficiaryAccount,
                transfer.input.amount,
            ],
        )
        const mintCommand: V2CommandStruct = {
            kind: 4,
            gas: transfer.computed.tokenErcMetadata.deliveryGas || 200_000n,
            payload: mintForeignParams,
        }
        commands.push(mintCommand)
    } else {
        // ENA
        const unlockNativeParams = encodeAbiTuple(
            context,
            ["address", "address", "uint128"],
            [
                transfer.input.tokenAddress,
                transfer.input.beneficiaryAccount,
                transfer.input.amount,
            ],
        )
        const unlockCommand: V2CommandStruct = {
            kind: 2,
            gas: transfer.computed.tokenErcMetadata.deliveryGas || 200_000n,
            payload: unlockNativeParams,
        }
        commands.push(unlockCommand)
    }

    if (transfer.input.contractCall) {
        let callInfo = transfer.input.contractCall
        // 2. Transact
        const transactParams = encodeAbiTuple(
            context,
            ["address", "bytes", "uint256"],
            [callInfo.target, callInfo.calldata, callInfo.value || 0n],
        )
        const transactCommand: V2CommandStruct = {
            kind: 5,
            gas: callInfo.gas,
            // gas: 1n,
            payload: transactParams,
        }
        commands.push(transactCommand)
    }
    const ethereumTx = (await context
        .gatewayProxy()
        .getFunction("v2_dispatch")
        // nonce is irrelevant in the dry run, can be set to 0
        .populateTransaction(commands, agentID, 0n, {
            from: context.environment.gatewayContract,
        })) as T["ContractTransaction"]
    return ethereumTx
}

function computeDryRunDispatchGasLimit(commandGasBudgets: bigint[]): bigint {
    const requiredGas = commandGasBudgets.reduce((acc, commandGas) => {
        return acc + commandGas + V2_DISPATCH_OVERHEAD_GAS
    }, 0n)
    // Account for the 63/64 forwarding rule used in Gateway.v2_dispatch gas checks.
    const minGas = (requiredGas * 64n + 62n) / 63n
    return minGas + DRY_RUN_GAS_BUFFER
}

function dryRunCommandGasBudgets(transfer: Transfer): bigint[] {
    const commandGasBudgets: bigint[] = [
        transfer.computed.tokenErcMetadata.deliveryGas || 200_000n,
    ]

    if (transfer.input.contractCall) {
        commandGasBudgets.push(transfer.input.contractCall.gas)
    }

    return commandGasBudgets
}

function parseL1AdaptorDryRunEvent(log: any, l1AdapterAddress?: string): {
    name: "DepositCallInvoked" | "DepositCallFailed"
    topic?: string
    depositId?: bigint
} | null {
    if (!l1AdapterAddress) {
        return null
    }
    const address = String(log?.address || "").toLowerCase()
    if (!address || address !== l1AdapterAddress.toLowerCase()) {
        return null
    }

    const topics = Array.isArray(log?.topics) ? log.topics.map((t: any) => String(t)) : []
    const topic0 = (topics[0] || "").toLowerCase()
    const data = String(log?.data || "0x")

    if (topic0 === L1_ADAPTOR_DEPOSIT_CALL_INVOKED_TOPIC0 && data.length >= 130) {
        const topic = "0x" + data.slice(2, 66)
        const depositIdHex = data.slice(66, 130)
        return {
            name: "DepositCallInvoked",
            topic,
            depositId: BigInt("0x" + depositIdHex),
        }
    }

    if (topic0 === L1_ADAPTOR_DEPOSIT_CALL_FAILED_TOPIC0 && data.length >= 66) {
        const topic = "0x" + data.slice(2, 66)
        return {
            name: "DepositCallFailed",
            topic,
        }
    }

    return null
}

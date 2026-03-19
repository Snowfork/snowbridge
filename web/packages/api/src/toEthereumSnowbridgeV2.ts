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
import {
    DeliveryFee,
    dryRunBridgeHub,
    MessageReceipt,
    Transfer,
    ValidationKind,
    ValidationLog,
    ValidationReason,
    ValidatedTransfer,
} from "./toEthereum_v2"
import { PNAFromAH } from "./transfers/toEthereum/pnaFromAH"
import { TransferInterface } from "./transfers/toEthereum/transferInterface"
import { ERC20FromAH } from "./transfers/toEthereum/erc20FromAH"
import { PNAFromParachain } from "./transfers/toEthereum/pnaFromParachain"
import { ERC20FromParachain } from "./transfers/toEthereum/erc20FromParachain"
import {
    isRelaychainLocation,
    isParachainNative,
    DOT_LOCATION,
    HERE_LOCATION,
    bridgeLocation,
} from "./xcmBuilder"
import { xxhashAsHex } from "@polkadot/util-crypto"
import { BN } from "@polkadot/util"
import { ensureValidationSuccess, padFeeByPercentage } from "./utils"
import { Context } from "./index"
import { ETHER_TOKEN_ADDRESS, findL2TokenAddress } from "./assets_v2"
import { getOperatingStatus } from "./status"
import { estimateFees } from "./across/api"

export { ValidationKind, signAndSendTransfer } from "./toEthereum_v2"

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

export const MaxWeight = { refTime: 30_000_000_000n, proofSize: 1_000_000 }

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
    },
): Promise<bigint> => {
    const ethereum = await context.ethereum()
    const tokenErcMetadata =
        registry.ethereumChains[`ethereum_${registry.ethChainId}`].assets[
            tokenAddress.toLowerCase()
        ]
    if (!tokenErcMetadata) {
        throw Error(`No token ${tokenAddress} registered on ethereum chain ${registry.ethChainId}.`)
    }

    // Calculate execution cost on ethereum
    let ethereumChain = registry.ethereumChains[`ethereum_${registry.ethChainId}`]
    let feeData = await context.ethereumProvider.getFeeData(ethereum)
    let ethereumExecutionFee =
        (feeData.gasPrice ?? 2_000_000_000n) *
        ((tokenErcMetadata.deliveryGas ?? 80_000n) +
            (ethereumChain.baseDeliveryGas ?? 120_000n) +
            (options?.contractCall?.gas ?? 0n))
    return ethereumExecutionFee
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
    let ethereumExecutionFee = padFeeByPercentage(
        await estimateEthereumExecutionFee(
            context,
            registry,
            registry.parachains[`polkadot_${registry.assetHubParaId}`],
            tokenAddress,
            options,
        ),
        feePadPercentage,
    )

    // calculate the cost of swapping in native asset
    let totalFeeInNative: bigint | undefined = undefined
    let assetHubExecutionFeeNative: bigint | undefined = undefined
    let returnToSenderExecutionFeeNative: bigint | undefined = undefined
    let ethereumExecutionFeeInNative: bigint | undefined
    let localExecutionFeeInNative: bigint | undefined
    let feeLocation = options?.feeTokenLocation
    if (feeLocation) {
        // If the fee asset is DOT, then one swap from DOT to Ether is required on AH
        if (isRelaychainLocation(feeLocation)) {
            ethereumExecutionFeeInNative = await assetHubImpl.getAssetHubConversionPalletSwap(
                DOT_LOCATION,
                bridgeLocation(registry.ethChainId),
                padFeeByPercentage(ethereumExecutionFee, feeSlippagePadPercentage),
            )
            totalFeeInDot += ethereumExecutionFeeInNative
            totalFeeInNative = totalFeeInDot
        } else {
            throw new Error("Unsupported fee token location")
        }
    }

    return {
        kind: l2ChainId ? "polkadot->ethereum_l2" : "polkadot->ethereum",
        localExecutionFeeDOT,
        snowbridgeDeliveryFeeDOT,
        assetHubExecutionFeeDOT,
        bridgeHubDeliveryFeeDOT,
        returnToSenderDeliveryFeeDOT,
        returnToSenderExecutionFeeDOT,
        totalFeeInDot,
        ethereumExecutionFee,
        feeLocation,
        assetHubExecutionFeeNative,
        returnToSenderExecutionFeeNative,
        ethereumExecutionFeeInNative,
        localExecutionFeeInNative,
        totalFeeInNative,
        l2BridgeFeeInL1Token,
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

    let ethereumExecutionFee = await estimateEthereumExecutionFee(
        context,
        registry,
        sourceParachain,
        tokenAddress,
        options,
    )

    // calculate the cost of swapping in native asset
    let totalFeeInNative: bigint | undefined = undefined
    let assetHubExecutionFeeNative: bigint | undefined = undefined
    let ethereumExecutionFeeInNative: bigint | undefined
    let feeLocation = options?.feeTokenLocation
    if (feeLocation) {
        // If the fee asset is DOT, then one swap from DOT to Ether is required on AH
        if (isRelaychainLocation(feeLocation)) {
            ethereumExecutionFeeInNative = await assetHubImpl.getAssetHubConversionPalletSwap(
                DOT_LOCATION,
                bridgeLocation(registry.ethChainId),
                padFeeByPercentage(ethereumExecutionFee, feeSlippagePadPercentage),
            )
            totalFeeInDot += ethereumExecutionFeeInNative
            totalFeeInNative = totalFeeInDot
        }
        // On Parachains, we can use their native asset as the fee token.
        // If the fee is in native, we need to swap it to DOT first, then swap DOT to Ether to cover the ethereum execution fee.
        else if (isParachainNative(feeLocation, sourceParaId)) {
            let ethereumExecutionFeeInDOT = await assetHubImpl.getAssetHubConversionPalletSwap(
                DOT_LOCATION,
                bridgeLocation(registry.ethChainId),
                padFeeByPercentage(ethereumExecutionFee, feeSlippagePadPercentage),
            )
            ethereumExecutionFeeInNative = await assetHubImpl.getAssetHubConversionPalletSwap(
                feeLocation,
                DOT_LOCATION,
                padFeeByPercentage(ethereumExecutionFeeInDOT, feeSlippagePadPercentage),
            )
            totalFeeInDot += ethereumExecutionFeeInDOT
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

    return {
        kind: "polkadot->ethereum",
        localExecutionFeeDOT,
        localDeliveryFeeDOT,
        snowbridgeDeliveryFeeDOT,
        assetHubExecutionFeeDOT,
        bridgeHubDeliveryFeeDOT,
        returnToSenderDeliveryFeeDOT: 0n,
        returnToSenderExecutionFeeDOT: 0n,
        totalFeeInDot,
        ethereumExecutionFee,
        feeLocation,
        assetHubExecutionFeeNative,
        returnToSenderExecutionFeeNative: 0n,
        ethereumExecutionFeeInNative,
        localExecutionFeeInNative,
        localDeliveryFeeInNative,
        totalFeeInNative,
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
    if (isNativeBalance && fee.totalFeeInNative) {
        if (amount + fee.totalFeeInNative > tokenBalance) {
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

        if (fee.ethereumExecutionFee! > etherBalance) {
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
            message: "Dry run call failed on Asset Hub.",
        })
        assetHubDryRunError = dryRunResultAssetHub.error
    }

    const paymentInfo = await tx.paymentInfo(sourceAccountHex)
    const sourceExecutionFee = paymentInfo["partialFee"].toBigInt()

    // recheck total after fee estimation
    if (isNativeBalance && fee.totalFeeInNative) {
        if (amount + fee.totalFeeInNative + sourceExecutionFee > tokenBalance) {
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.InsufficientTokenBalance,
                message: "Insufficient token balance to submit transaction.",
            })
        }
    }
    if (sourceExecutionFee + fee.totalFeeInDot > dotBalance) {
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

    if (isNativeBalance && fee.totalFeeInNative) {
        if (amount + fee.totalFeeInNative > tokenBalance) {
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

    if (!fee.feeLocation) {
        let etherBalance = await sourceParachainImpl.getTokenBalance(
            sourceAccountHex,
            registry.ethChainId,
            ETHER_TOKEN_ADDRESS,
        )

        // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
        if (fee.ethereumExecutionFee! > etherBalance) {
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

    const paymentInfo = await tx.paymentInfo(sourceAccountHex)
    const sourceExecutionFee = paymentInfo["partialFee"].toBigInt()

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

export const mockDeliveryFee: DeliveryFee = {
    kind: "polkadot->ethereum",
    localExecutionFeeDOT: 1n,
    localDeliveryFeeDOT: 1n,
    snowbridgeDeliveryFeeDOT: 1n,
    assetHubExecutionFeeDOT: 1n,
    bridgeHubDeliveryFeeDOT: 1n,
    returnToSenderDeliveryFeeDOT: 0n,
    returnToSenderExecutionFeeDOT: 0n,
    totalFeeInDot: 10n,
    ethereumExecutionFee: 1n,
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

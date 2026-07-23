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
    matchesConsensusSystem,
} from "./xcmBuilder"
import { DOT_LOCATION } from "./assets_v2"
import {
    buildKusamaToPolkadotDestAssetHubXCM,
    buildPolkadotToKusamaDestAssetHubXCM,
    buildTransferKusamaToPolkadotExportXCM,
    buildTransferPolkadotToKusamaExportXCM,
} from "./xcmBuilderKusama"
import {
    Asset,
    AssetRegistry,
    AssetMap,
    BridgeInfo,
    ChainId,
    Parachain,
    TransferRoute,
    EthereumProviderTypes,
} from "@snowbridge/base-types"
import { CallDryRunEffects, XcmDryRunApiError, XcmDryRunEffects } from "@polkadot/types/interfaces"
import { Result } from "@polkadot/types"
import { ensureValidationSuccess, padFeeByPercentage, u32ToLeBytes } from "./utils"
import {
    addBreakdown,
    computeTotals,
    findInBreakdown,
    findInBreakdownOrZero,
    findTotal,
} from "./fees"
import {
    checkDotKsmPoolLiquidityForKusamaToPolkadot,
    checkKsmDotPoolLiquidityForPolkadotToKusama,
} from "./poolReserves"
import { resolveBeneficiary } from "./crypto"
import { TransferInterface as KusamaTransferInterface } from "./transfers/forKusama/transferInterface"
import { Context } from "."
import type {
    DeliveryFee,
    MessageReceipt,
    Transfer,
    ValidatedTransfer,
    ValidationLog,
} from "./types/forKusama"
import { ValidationKind, ValidationReason } from "./types/forKusama"
export { ValidationKind, ValidationReason } from "./types/forKusama"

export enum Direction {
    ToKusama,
    ToPolkadot,
}

const KUSAMA_BASE_FEE = 10_602_492_378n // 0.0106KSM
const KUSAMA_FEE_PER_BYTE = 1000000n // 0.000001 KSM
const POLKADOT_BASE_FEE = 333_794_429n // 0.033 DOT
const POLKADOT_FEE_PER_BYTE = 16666n // 0.0000016666 DOT

// Service fee (~$1) deposited on Polkadot AH for both directions, on top of the
// bridge/execution fees. TEST CONFIG: recipient is Clara's own account so the fee is
// claimable back during testing; replace for production. Denominated in the source
// chain's native asset (DOT for polkadot->kusama, KSM for kusama->polkadot), which is
// what each direction has in Polkadot-AH holding. Fixed native amounts for now, so the
// dollar value drifts with price (DOT ~$0.80, KSM ~$3.17 as of 2026-07-20).
const SERVICE_FEE_RECIPIENT =
    "0x6cd8840ea69d18a2f1dde746df70629df118b870ec228367d6bcf3348ca3b10b"
const SERVICE_FEE_DOT = 12_400_000_000n // ~1.24 DOT (10 decimals) ~= $1
const SERVICE_FEE_KSM = 315_000_000_000n // ~0.315 KSM (12 decimals) ~= $1

function serviceFeeAmount(direction: Direction): bigint {
    return direction === Direction.ToPolkadot ? SERVICE_FEE_KSM : SERVICE_FEE_DOT
}

function resolveInputs(
    registry: AssetRegistry,
    tokenAddress: string,
    sourceParaId: number,
    destParaId: number,
) {
    const sourceParachain = registry.parachains[`polkadot_${sourceParaId}`]
    if (!sourceParachain) {
        throw Error(`Could not find ${sourceParaId} in the asset registry.`)
    }
    const destParachain = registry.kusama?.parachains[`kusama_${destParaId}`]
    if (!destParachain) {
        throw Error(`Could not find ${destParaId} in the asset registry.`)
    }

    const sourceAssetMetadata = sourceParachain.assets[tokenAddress.toLowerCase()]
    if (!sourceAssetMetadata) {
        throw Error(`Token ${tokenAddress} not registered on source asset hub.`)
    }
    const destAssetMetadata = destParachain.assets[tokenAddress.toLowerCase()]
    if (!destAssetMetadata) {
        throw Error(`Token ${tokenAddress} not registered on destination asset hub.`)
    }

    return { sourceAssetMetadata, destAssetMetadata, sourceParachain }
}

export class KusamaTransfer<T extends EthereumProviderTypes> implements KusamaTransferInterface<T> {
    readonly info: BridgeInfo
    readonly context: Context<T>
    readonly route: TransferRoute
    readonly source: Parachain
    readonly destination: Parachain

    constructor(
        info: BridgeInfo,
        context: Context<T>,
        route: TransferRoute,
        source: Parachain,
        destination: Parachain,
    ) {
        this.info = info
        this.context = context
        this.route = route
        this.source = source
        this.destination = destination
    }

    get from(): ChainId {
        return this.route.from
    }

    get to(): ChainId {
        return this.route.to
    }

    #direction() {
        return this.from.kind === "kusama" ? Direction.ToPolkadot : Direction.ToKusama
    }

    async #connections() {
        const [polkadotAssetHub, kusamaAssetHub] = await Promise.all([
            this.context.assetHub(),
            this.context.kusamaAssetHub(),
        ])
        if (this.#direction() === Direction.ToPolkadot) {
            return { sourceAssetHub: kusamaAssetHub, destAssetHub: polkadotAssetHub }
        }
        return { sourceAssetHub: polkadotAssetHub, destAssetHub: kusamaAssetHub }
    }

    async fee(tokenAddress: string): Promise<DeliveryFee> {
        const { sourceAssetHub, destAssetHub } = await this.#connections()
        let baseFeeInStorage = await getStorageItem(sourceAssetHub, ":XcmBridgeHubRouterBaseFee:")
        let xcmBridgeBaseFee: bigint
        if (baseFeeInStorage.eqn(0)) {
            console.warn("Asset Hub onchain XcmBridgeHubRouterBaseFee not set. Using default fee.")
            if (this.#direction() == Direction.ToPolkadot) {
                xcmBridgeBaseFee = KUSAMA_BASE_FEE
            } else {
                xcmBridgeBaseFee = POLKADOT_BASE_FEE
            }
        } else {
            xcmBridgeBaseFee = BigInt(baseFeeInStorage.toString())
        }

        let feePerByteInStorage = await getStorageItem(
            sourceAssetHub,
            ":XcmBridgeHubRouterByteFee:",
        )
        let xcmFeePerByte: bigint
        if (feePerByteInStorage.eqn(0)) {
            console.warn(
                "Asset Hub onchain XcmBridgeHubRouterByteFee not set. Using default fee per byte.",
            )
            if (this.#direction() == Direction.ToPolkadot) {
                xcmFeePerByte = KUSAMA_FEE_PER_BYTE
            } else {
                xcmFeePerByte = POLKADOT_FEE_PER_BYTE
            }
        } else {
            xcmFeePerByte = BigInt(baseFeeInStorage.toString())
        }

        let tokenLocation = getTokenLocation(this.info.registry, this.#direction(), tokenAddress)

        if (!this.info.registry.kusama) {
            throw Error("Kusama config is not set")
        }

        let forwardedXcm
        if (this.#direction() == Direction.ToPolkadot) {
            forwardedXcm = buildTransferKusamaToPolkadotExportXCM(
                sourceAssetHub.registry,
                tokenLocation,
                xcmBridgeBaseFee,
                xcmBridgeBaseFee,
                this.info.registry.kusama?.assetHubParaId,
                this.info.registry.assetHubParaId,
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
                this.info.registry.assetHubParaId,
                this.info.registry.kusama?.assetHubParaId,
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

        let destXcm
        if (this.#direction() == Direction.ToPolkadot) {
            destXcm = buildKusamaToPolkadotDestAssetHubXCM(
                destAssetHub.registry,
                totalXcmBridgeFee,
                this.info.registry.assetHubParaId,
                tokenLocation,
                100000000000n,
                "0x0000000000000000000000000000000000000000000000000000000000000000",
                "0x0000000000000000000000000000000000000000000000000000000000000000",
            )
        } else {
            destXcm = buildPolkadotToKusamaDestAssetHubXCM(
                destAssetHub.registry,
                totalXcmBridgeFee,
                this.info.registry.assetHubParaId,
                tokenLocation,
                100000000000n,
                "0x0000000000000000000000000000000000000000000000000000000000000000",
                "0x0000000000000000000000000000000000000000000000000000000000000000",
            )
        }
        const destAssetHubImpl = await this.context.paraImplementation(destAssetHub)
        let destinationFeeInDestNative = await destAssetHubImpl.calculateXcmFee(
            destXcm,
            DOT_LOCATION,
        )

        const sourceAssetHubImpl = await this.context.paraImplementation(sourceAssetHub)
        let bridgeHubDeliveryFee = await sourceAssetHubImpl.calculateDeliveryFeeInDOT(
            this.info.registry.bridgeHubParaId,
            forwardedXcm,
        )

        let feeAssetOnDest
        let minBalanceFeeDest: bigint
        if (this.#direction() == Direction.ToPolkadot) {
            feeAssetOnDest = ksmLocationOnPolkadotAssetHub
            minBalanceFeeDest = getDestFeeAssetMinimumBalance(
                this.info.registry.parachains[`polkadot_${this.info.registry.assetHubParaId}`]
                    .assets,
                "kusama",
            )
        } else {
            feeAssetOnDest = dotLocationOnKusamaAssetHub
            minBalanceFeeDest = getDestFeeAssetMinimumBalance(
                this.info.registry.kusama.parachains[
                    `kusama_${this.info.registry.kusama.assetHubParaId}`
                ].assets,
                "polkadot",
            )
        }
        let destinationFee = await destAssetHubImpl.getAssetHubConversionPalletSwap(
            feeAssetOnDest,
            NATIVE_TOKEN_LOCATION,
            destinationFeeInDestNative,
        )
        destinationFee = padFeeByPercentage(destinationFee, 33n)
        destinationFee = destinationFee + BigInt(minBalanceFeeDest)
        totalXcmBridgeFee = padFeeByPercentage(totalXcmBridgeFee, 33n)

        // Service fee (~$1) charged in the source native asset, deposited on Polkadot AH.
        const serviceFee = serviceFeeAmount(this.#direction())

        let totalFee = totalXcmBridgeFee + bridgeHubDeliveryFee + destinationFee
        const sourceSymbol = this.#direction() === Direction.ToPolkadot ? "KSM" : "DOT"
        // destNativeSymbol is the asset coming OUT of the destination AH swap
        // (DOT for kusama→polkadot, KSM for polkadot→kusama).
        const destNativeSymbol = this.#direction() === Direction.ToPolkadot ? "DOT" : "KSM"

        const breakdown: DeliveryFee["breakdown"] = {}
        addBreakdown(breakdown, "xcmBridge", { amount: totalXcmBridgeFee, symbol: sourceSymbol })
        addBreakdown(breakdown, "bridgeHubDelivery", {
            amount: bridgeHubDeliveryFee,
            symbol: sourceSymbol,
        })
        addBreakdown(breakdown, "destinationExecution", {
            amount: destinationFee,
            symbol: sourceSymbol,
        })
        addBreakdown(breakdown, "destinationExecution", {
            amount: destinationFeeInDestNative,
            symbol: destNativeSymbol,
        })
        addBreakdown(breakdown, "serviceFee", { amount: serviceFee, symbol: sourceSymbol })

        const summary = [
            { description: "Bridge fee", amount: totalFee, symbol: sourceSymbol },
            { description: "Service fee", amount: serviceFee, symbol: sourceSymbol },
        ]

        return {
            kind: this.from.kind === "kusama" ? "kusama->polkadot" : "polkadot->kusama",
            breakdown,
            summary,
            totals: computeTotals(summary),
        }
    }

    async tx(
        sourceAccount: string,
        beneficiaryAccount: string,
        tokenAddress: string,
        amount: bigint,
        fee: DeliveryFee,
    ): Promise<Transfer> {
        const { sourceAssetHub } = await this.#connections()
        const { assetHubParaId } = this.info.registry
        const destParaId = this.info.registry.kusama?.assetHubParaId
        let sourceParaId = assetHubParaId
        const sourceParachainImpl = await this.context.paraImplementation(sourceAssetHub)

        let sourceAccountHex = sourceAccount
        if (!isHex(sourceAccountHex)) {
            sourceAccountHex = u8aToHex(decodeAddress(sourceAccount))
        }

        if (!destParaId) {
            throw Error("Kusama destination para ID is not set")
        }

        let { hexAddress: beneficiaryAddressHex } = resolveBeneficiary(beneficiaryAccount)

        const { sourceAssetMetadata, destAssetMetadata, sourceParachain } = resolveInputs(
            this.info.registry,
            tokenAddress,
            sourceParaId,
            destParaId,
        )
        const accountNonce = await sourceParachainImpl.accountNonce(sourceAccountHex)
        let messageId = buildMessageId(
            sourceParaId,
            sourceAccountHex,
            accountNonce,
            tokenAddress,
            beneficiaryAccount,
            amount,
        )

        let tokenLocationOnSource = getTokenLocation(
            this.info.registry,
            this.#direction(),
            tokenAddress,
        )
        const serviceFee = serviceFeeAmount(this.#direction())
        let tx
        if (this.#direction() == Direction.ToPolkadot) {
            tx = createERC20ToPolkadotTx(
                sourceParaId,
                sourceAssetHub,
                tokenLocationOnSource,
                beneficiaryAddressHex,
                amount,
                findInBreakdown(
                    fee.breakdown,
                    "destinationExecution",
                    fee.kind === "kusama->polkadot" ? "KSM" : "DOT",
                ),
                serviceFee,
                messageId,
            )
        } else {
            tx = createERC20ToKusamaTx(
                destParaId,
                sourceAssetHub,
                tokenLocationOnSource,
                beneficiaryAddressHex,
                amount,
                findInBreakdown(
                    fee.breakdown,
                    "destinationExecution",
                    fee.kind === "kusama->polkadot" ? "KSM" : "DOT",
                ),
                serviceFee,
                messageId,
            )
        }

        return {
            kind: `${this.from.kind}->${this.to.kind}` as Transfer["kind"],
            input: {
                registry: this.info.registry,
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

    async build(
        sourceAccount: string,
        beneficiaryAccount: string,
        tokenAddress: string,
        amount: bigint,
    ): Promise<ValidatedTransfer> {
        const fee = await this.fee(tokenAddress)
        const transfer = await this.tx(sourceAccount, beneficiaryAccount, tokenAddress, amount, fee)
        return ensureValidationSuccess(await this.validate(transfer))
    }

    async validate(transfer: Transfer): Promise<ValidatedTransfer> {
        const connections = await this.#connections()
        let sourceAssetHub = connections.sourceAssetHub
        let destAssetHub = connections.destAssetHub

        const { registry, fee, tokenAddress, amount } = transfer.input
        const {
            sourceAccountHex,
            sourceParachain: _source,
            beneficiaryAddressHex,
            sourceAssetMetadata,
            destAssetMetadata,
        } = transfer.computed
        const { tx } = transfer

        let tokenLocation = getTokenLocation(registry, this.#direction(), tokenAddress)

        const sourceAssetHubImpl = await this.context.paraImplementation(sourceAssetHub)
        let nativeBalance = await sourceAssetHubImpl.getNativeBalance(sourceAccountHex, true)

        let tokenAsset = getTransferAsset(this.#direction(), tokenAddress, transfer.input.registry)

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

        // The transferred token is deposited in full on the destination asset hub (the fee is paid
        // separately in the native asset), so the amount must clear the token's minimum balance
        // there or the deposit fails and the funds trap under the message origin. An amount exactly
        // equal to the minimum balance also traps (observed with 0.01 USDC == its min balance), so
        // require headroom above it. Mirror the other transfer impls (e.g. toKusama/erc20ToKusamaAH)
        // and take the larger of the two sides' minimum balances so the check is correct regardless
        // of direction.
        const destTokenMinimumBalance =
            sourceAssetMetadata.minimumBalance > destAssetMetadata.minimumBalance
                ? sourceAssetMetadata.minimumBalance
                : destAssetMetadata.minimumBalance
        if (amount <= destTokenMinimumBalance) {
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.MinimumAmountValidation,
                message:
                    "The amount transferred is at or below the minimum balance of the token on the destination chain.",
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

        if (
            sourceExecutionFee + findTotal(fee, fee.kind === "kusama->polkadot" ? "KSM" : "DOT") >
            nativeBalance
        ) {
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.InsufficientFee,
                message:
                    "Insufficient " +
                    nativeFeeAsset(this.#direction()) +
                    " balance to submit transaction on the source parachain.",
            })
        }

        let destAssetHubXCM: any
        if (this.#direction() == Direction.ToPolkadot) {
            // Model the service-fee skim so the dest dry-run is faithful to the real message
            // (kusama->polkadot deposits the KSM service fee on Polkadot AH).
            destAssetHubXCM = buildKusamaToPolkadotDestAssetHubXCM(
                destAssetHub.registry,
                findInBreakdown(
                    fee.breakdown,
                    "destinationExecution",
                    fee.kind === "kusama->polkadot" ? "KSM" : "DOT",
                ),
                registry.assetHubParaId,
                tokenLocation,
                transfer.input.amount,
                transfer.computed.beneficiaryAddressHex,
                "0x0000000000000000000000000000000000000000000000000000000000000000",
                serviceFeeAmount(this.#direction()),
                SERVICE_FEE_RECIPIENT,
            )
        } else {
            destAssetHubXCM = buildPolkadotToKusamaDestAssetHubXCM(
                destAssetHub.registry,
                findInBreakdown(
                    fee.breakdown,
                    "destinationExecution",
                    fee.kind === "kusama->polkadot" ? "KSM" : "DOT",
                ),
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
                message:
                    "Dry run call on destination AH failed: " + dryRunAssetHubDest.errorMessage,
            })
            assetHubDryRunError = dryRunAssetHubDest.errorMessage

            const destAssetHubImpl = await this.context.paraImplementation(destAssetHub)
            const { accountMaxConsumers, accountExists } = await destAssetHubImpl.validateAccount(
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

        const destAssetHubImpl = await this.context.paraImplementation(destAssetHub)
        if (this.#direction() === Direction.ToPolkadot) {
            const requiredDotOut = findInBreakdownOrZero(
                fee.breakdown,
                "destinationExecution",
                "DOT",
            )
            if (requiredDotOut > 0n) {
                const reserveCheck = await checkDotKsmPoolLiquidityForKusamaToPolkadot(
                    destAssetHubImpl,
                    requiredDotOut,
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
        } else {
            const requiredKsmOut = findInBreakdownOrZero(
                fee.breakdown,
                "destinationExecution",
                "KSM",
            )
            if (requiredKsmOut > 0n) {
                const reserveCheck = await checkKsmDotPoolLiquidityForPolkadotToKusama(
                    destAssetHubImpl,
                    requiredKsmOut,
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
                nativeBalance,
                sourceExecutionFee,
                tokenBalance,
                assetHubDryRunError,
            },
            ...transfer,
        }
    }

    async signAndSend(
        transfer: Transfer,
        account: AddressOrPair,
        options: Partial<SignerOptions>,
    ): Promise<MessageReceipt> {
        const { sourceAssetHub } = await this.#connections()
        const result = await new Promise<MessageReceipt>((resolve, reject) => {
            try {
                transfer.tx.signAndSend(account, options, (c) => {
                    if (c.isError) {
                        console.error(c)
                        reject(c.internalError || c.dispatchError || c)
                    }
                    if (c.isFinalized) {
                        const result = {
                            txHash: u8aToHex(c.txHash),
                            txIndex: c.txIndex || 0,
                            blockNumber: Number((c as any).blockNumber),
                            blockHash: "",
                            events: c.events,
                        }
                        for (const e of c.events) {
                            if (sourceAssetHub.events.system.ExtrinsicFailed.is(e.event)) {
                                resolve({
                                    ...result,
                                    success: false,
                                    dispatchError: (e.event.data.toHuman(true) as any)
                                        ?.dispatchError,
                                })
                            }
                            if (sourceAssetHub.events.polkadotXcm.Sent.is(e.event)) {
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
        result.blockHash = u8aToHex(await sourceAssetHub.rpc.chain.getBlockHash(result.blockNumber))
        result.messageId = transfer.computed.messageId ?? result.messageId
        return result
    }
}

function createERC20ToKusamaTx(
    destParaId: number,
    parachain: ApiPromise,
    tokenLocation: any,
    beneficiaryAccount: string,
    amount: bigint,
    destFeeInSourceNative: bigint,
    serviceFee: bigint,
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
    const transfer = parachain.tx.polkadotXcm.transferAssetsUsingTypeAndThen(
        destination,
        assets,
        reserveTypeAsset,
        feeAsset,
        "LocalReserve",
        customXcm,
        "Unlimited",
    )
    // polkadot->kusama: source IS Polkadot AH, so pay the service fee here as a local DOT
    // transfer to the recipient, atomically with the bridge transfer via batchAll. (The
    // customXcm runs on Kusama AH for this direction, so it cannot place a PAH-side fee.)
    const serviceFeeTransfer = parachain.tx.balances.transferKeepAlive(
        SERVICE_FEE_RECIPIENT,
        serviceFee,
    )
    return parachain.tx.utility.batchAll([serviceFeeTransfer, transfer])
}

function createERC20ToPolkadotTx(
    destParaId: number,
    parachain: ApiPromise,
    tokenLocation: any,
    beneficiaryAccount: string,
    amount: bigint,
    destFeeInSourceNative: bigint,
    serviceFee: bigint,
    topic: string,
): SubmittableExtrinsic<"promise", ISubmittableResult> {
    let assets: any
    let reserveTypeAsset = "DestinationReserve"
    // kusama->polkadot: the custom XCM runs on Polkadot AH (dest) holding KSM, so the service
    // fee is skimmed there. Send it from Kusama AH as extra KSM headroom on top of the
    // destination execution fee (`destFeeInSourceNative`), so it survives BuyExecution and is
    // present in Polkadot-AH holding for the fee deposit.
    // is KSM
    if (isRelaychainLocation(tokenLocation)) {
        assets = {
            v4: [
                {
                    id: NATIVE_TOKEN_LOCATION,
                    fun: { Fungible: destFeeInSourceNative + amount + serviceFee },
                },
            ],
        }
        reserveTypeAsset = "LocalReserve"
    } else {
        assets = {
            v4: [
                {
                    id: NATIVE_TOKEN_LOCATION,
                    fun: { Fungible: destFeeInSourceNative + serviceFee },
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
        serviceFee,
        SERVICE_FEE_RECIPIENT,
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

async function dryRunSourceAssetHub(
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

export async function dryRunDestAssetHub(assetHub: ApiPromise, parachainId: number, xcm: any) {
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

function buildMessageId(
    sourceParaId: number,
    sourceAccountHex: string,
    accountNonce: number,
    tokenAddress: string,
    beneficiaryAccount: string,
    amount: bigint,
): string {
    const entropy = new Uint8Array([
        ...stringToU8a(sourceParaId.toString()),
        ...hexToU8a(sourceAccountHex),
        ...u32ToLeBytes(accountNonce),
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
            registry.kusama?.parachains[`kusama_${registry.kusama?.assetHubParaId}`].assets[
                tokenAddress
            ].location
        if (!location) {
            location = erc20Location(registry.ethChainId, tokenAddress)
        }
    } else {
        location =
            registry.parachains[`polkadot_${registry.assetHubParaId}`].assets[tokenAddress].location
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
        return registry.kusama?.parachains[`kusama_${registry.kusama?.assetHubParaId}`].assets[
            tokenAddress
        ]
    } else {
        return registry.parachains[`polkadot_${registry.assetHubParaId}`].assets[tokenAddress]
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

import { ApiPromise } from "@polkadot/api"
import { AddressOrPair, SignerOptions, SubmittableExtrinsic } from "@polkadot/api/types"
import { ISubmittableResult } from "@polkadot/types/types"
import {
    erc20Location,
    parachainLocation,
    buildParachainERC20ReceivedXcmOnDestination,
    buildERC20ToAssetHubFromParachain,
    buildDepositAllAssetsWithTopic,
} from "./xcmBuilder"
import { DOT_LOCATION } from "./assets_v2"
import {
    Asset,
    AssetRegistry,
    BridgeInfo,
    ChainId,
    EthereumProviderTypes,
    Parachain,
    TransferRoute,
} from "@snowbridge/base-types"
import { ensureValidationSuccess, padFeeByPercentage } from "./utils"
import { resolveBeneficiary } from "./crypto"
import { Context } from "."
import { buildMessageId } from "./toEthereum_v2"
import { Result } from "@polkadot/types"
import {
    CallDryRunEffects,
    XcmDryRunApiError,
    XcmDryRunEffects,
} from "@polkadot/types/interfaces"
import { u8aToHex } from "@polkadot/util"
import { TransferInterface as InterParachainTransferInterface } from "./transfers/forInterParachain/transferInterface"
import type {
    DeliveryFee,
    MessageReceipt,
    Transfer,
    ValidatedTransfer,
    ValidationLog,
} from "./types/forInterParachain"
import { ValidationKind, ValidationReason } from "./types/forInterParachain"
export { ValidationKind, ValidationReason } from "./types/forInterParachain"

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
    const destParachain = registry.parachains[`polkadot_${destParaId}`]
    if (!destParachain) {
        throw Error(`Could not find ${destParaId} in the asset registry.`)
    }

    if (destParachain.id === sourceParachain.id) {
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


export class InterParachainTransfer<T extends EthereumProviderTypes>
    implements InterParachainTransferInterface<T>
{
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

    async fee(
        tokenAddress: string,
        options?: {
            padFeeByPercentage?: bigint
        },
    ): Promise<DeliveryFee> {
        const sourceParachain = await this.context.parachain(this.from.id)
        const destParachain = await this.context.parachain(this.to.id)

        const [source, destination] = await Promise.all([
            this.context.paraImplementation(sourceParachain),
            this.context.paraImplementation(destParachain),
        ])

        resolveInputs(this.info.registry, tokenAddress, source.parachainId, destination.parachainId)
        let xcm
        if (source.parachainId === this.info.registry.assetHubParaId) {
            xcm = buildParachainERC20ReceivedXcmOnDestination(
                sourceParachain.registry,
                this.info.registry.ethChainId,
                "0x0000000000000000000000000000000000000000",
                340282366920938463463374607431768211455n,
                340282366920938463463374607431768211455n,
                "0x0000000000000000000000000000000000000000000000000000000000000000",
                "0x0000000000000000000000000000000000000000000000000000000000000000",
            )
        } else {
            xcm = buildERC20ToAssetHubFromParachain(
                sourceParachain.registry,
                this.info.registry.ethChainId,
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
            options?.padFeeByPercentage ?? 33n,
        )
        const executionFee = padFeeByPercentage(
            await destination.calculateXcmFee(xcm, DOT_LOCATION),
            options?.padFeeByPercentage ?? 33n,
        )

        return {
            kind: "polkadot->polkadot",
            deliveryFee,
            executionFee,
            totalFeeInDot: deliveryFee + executionFee,
        }
    }

    async tx(
        sourceAccount: string,
        beneficiaryAccount: string,
        tokenAddress: string,
        amount: bigint,
        fee: DeliveryFee,
    ): Promise<Transfer> {
        const sourceParachain = await this.context.parachain(this.from.id)
        const source = await this.context.paraImplementation(sourceParachain)

        let { hexAddress: beneficiaryAddressHex } = resolveBeneficiary(beneficiaryAccount)
        let { hexAddress: sourceAccountHex } = resolveBeneficiary(sourceAccount)

        const {
            sourceAssetMetadata,
            destAssetMetadata,
            sourceParachain: sourceParachainMeta,
            destParachain,
        } = resolveInputs(this.info.registry, tokenAddress, source.parachainId, this.to.id)
        const accountNonce = await source.accountNonce(sourceAccountHex)
        let messageId = buildMessageId(
            source.parachainId,
            sourceAccountHex,
            accountNonce,
            tokenAddress,
            beneficiaryAccount,
            amount,
        )

        const tx = createTx(
            sourceParachain,
            this.info.registry.ethChainId,
            this.to.id,
            tokenAddress,
            beneficiaryAccount,
            messageId,
            amount,
            fee.totalFeeInDot,
            source.parachainId === this.info.registry.assetHubParaId
                ? "LocalReserve"
                : "DestinationReserve",
        )

        return {
            kind: "polkadot->polkadot",
            input: {
                registry: this.info.registry,
                sourceAccount,
                beneficiaryAccount,
                destinationParaId: this.to.id,
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

    async build(
        sourceAccount: string,
        beneficiaryAccount: string,
        tokenAddress: string,
        amount: bigint,
        options?: {
            fee?: {
                padFeeByPercentage?: bigint
            }
        },
    ): Promise<ValidatedTransfer> {
        const fee = await this.fee(tokenAddress, options?.fee)
        const transfer = await this.tx(sourceAccount, beneficiaryAccount, tokenAddress, amount, fee)
        return ensureValidationSuccess(await this.validate(transfer))
    }

    async validate(transfer: Transfer): Promise<ValidatedTransfer> {
        const sourceParachain = await this.context.parachain(this.from.id)
        const destParachain = await this.context.parachain(this.to.id)

        const [source, destination] = await Promise.all([
            this.context.paraImplementation(sourceParachain),
            this.context.paraImplementation(destParachain),
        ])

        const { registry, tokenAddress, amount, destinationParaId } = transfer.input
        const { sourceAccountHex, sourceAssetMetadata, destAssetMetadata, beneficiaryAddressHex } =
            transfer.computed
        const { tx } = transfer

        const nativeBalance = await source.getNativeBalance(sourceAccountHex, true)
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
                dryRunError,
            },
            ...transfer,
        }
    }

    async signAndSend(
        transfer: Transfer,
        account: AddressOrPair,
        options: Partial<SignerOptions>,
    ): Promise<MessageReceipt> {
        const sourceParachain = await this.context.parachain(this.from.id)
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
                            if (sourceParachain.events.system.ExtrinsicFailed.is(e.event)) {
                                resolve({
                                    ...result,
                                    success: false,
                                    dispatchError: (e.event.data.toHuman(true) as any)
                                        ?.dispatchError,
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

        result.blockHash = u8aToHex(
            await sourceParachain.rpc.chain.getBlockHash(result.blockNumber),
        )
        result.messageId = transfer.computed.messageId ?? result.messageId

        return result
    }
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
) {
    const origin = { system: { signed: sourceAccount } }
    let result: Result<CallDryRunEffects, XcmDryRunApiError>

    try {
        result = await source.call.dryRunApi.dryRunCall<
            Result<CallDryRunEffects, XcmDryRunApiError>
        >(origin, tx, 4)
    } catch {
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

import { MultiAddressStruct } from "./contracts"
import { ContractTransaction, TransactionReceipt } from "ethers"
import { padFeeByPercentage } from "./utils"
import { ETHER_TOKEN_ADDRESS } from "./assets_v2"
import {
    Asset,
    AssetRegistry,
    ChainId,
    ERC20Metadata,
    EthereumChain,
    Parachain,
    TransferRoute,
} from "@snowbridge/base-types"
import { getOperatingStatus, OperationStatus } from "./status"
import { ApiPromise } from "@polkadot/api"
import {
    buildAssetHubERC20ReceivedXcm,
    buildAssetHubPNAReceivedXcm,
    buildParachainERC20ReceivedXcmOnAssetHub,
    buildParachainERC20ReceivedXcmOnDestination,
    buildParachainPNAReceivedXcmOnAssetHub,
    buildParachainPNAReceivedXcmOnDestination,
    DOT_LOCATION,
} from "./xcmBuilder"
import { Result } from "@polkadot/types"
import { XcmDryRunApiError, XcmDryRunEffects } from "@polkadot/types/interfaces"
import { Context, EthersProviderTypes } from "./index"
import type { FeeData } from "./EthereumProvider"
import { TransferInterface as ToPolkadotTransferInterface } from "./transfers/toPolkadot/transferInterface"
import type {
    DeliveryFee as ToPolkadotV2DeliveryFee,
    Transfer as ToPolkadotV2Transfer,
    ValidationResult as ToPolkadotV2ValidationResult,
} from "./toPolkadotSnowbridgeV2"

export type Transfer = {
    input: {
        registry: AssetRegistry
        sourceAccount: string
        beneficiaryAccount: string
        tokenAddress: string
        destinationParaId: number
        amount: bigint
        fee: DeliveryFee
    }
    computed: {
        gatewayAddress: string
        beneficiaryAddressHex: string
        beneficiaryMultiAddress: MultiAddressStruct
        totalValue: bigint
        tokenErcMetadata: ERC20Metadata
        ahAssetMetadata: Asset
        destAssetMetadata: Asset
        destParachain: Parachain
        destinationFeeInDOT: bigint
        minimalBalance: bigint
    }
    tx: ContractTransaction
}

export enum ValidationKind {
    Warning,
    Error,
}

export enum ValidationReason {
    MinimumAmountValidation,
    GatewaySpenderLimitReached,
    InsufficientTokenBalance,
    FeeEstimationError,
    InsufficientEther,
    BridgeStatusNotOperational,
    DryRunNotSupportedOnDestination,
    NoDestinationParachainConnection,
    DryRunFailed,
    MaxConsumersReached,
    AccountDoesNotExist,
}

export type ValidationLog = {
    kind: ValidationKind
    reason: ValidationReason
    message: string
}

export type FeeInfo = {
    estimatedGas: bigint
    feeData: FeeData
    executionFee: bigint
    totalTxCost: bigint
}

export type DeliveryFee = {
    destinationDeliveryFeeDOT: bigint
    destinationExecutionFeeDOT: bigint
    totalFeeInWei: bigint
}

export type ValidationResult = {
    logs: ValidationLog[]
    success: boolean
    data: {
        etherBalance: bigint
        tokenBalance: {
            balance: bigint
            gatewayAllowance: bigint
        }
        feeInfo?: FeeInfo
        bridgeStatus: OperationStatus
        assetHubDryRunError?: string
        destinationParachainDryRunError?: string
    }
    transfer: Transfer
}

export type MessageReceipt = {
    channelId: string
    nonce: bigint
    messageId: string
    blockNumber: number
    blockHash: string
    txHash: string
    txIndex: number
}

function toV2DeliveryFee(fee: DeliveryFee): ToPolkadotV2DeliveryFee {
    return {
        ...fee,
        feeAsset: null,
        assetHubDeliveryFeeEther: 0n,
        assetHubExecutionFeeEther: 0n,
        destinationDeliveryFeeEther: 0n,
        destinationExecutionFeeEther: 0n,
        destinationExecutionFeeDOT: fee.destinationExecutionFeeDOT,
        relayerFee: 0n,
        extrinsicFeeDot: 0n,
        extrinsicFeeEther: 0n,
    }
}

function toV1DeliveryFee(fee: ToPolkadotV2DeliveryFee): DeliveryFee {
    const v1Fee = fee as unknown as {
        destinationDeliveryFeeDOT?: bigint
        destinationExecutionFeeDOT?: bigint
        totalFeeInWei?: bigint
    }
    if (
        typeof v1Fee.destinationDeliveryFeeDOT !== "bigint" ||
        typeof v1Fee.destinationExecutionFeeDOT !== "bigint" ||
        typeof v1Fee.totalFeeInWei !== "bigint"
    ) {
        throw new Error(
            "Unsupported fee object for v1 toPolkadot adapter. Expected destinationDeliveryFeeDOT, destinationExecutionFeeDOT, and totalFeeInWei.",
        )
    }
    return {
        destinationDeliveryFeeDOT: v1Fee.destinationDeliveryFeeDOT,
        destinationExecutionFeeDOT: v1Fee.destinationExecutionFeeDOT,
        totalFeeInWei: v1Fee.totalFeeInWei,
    }
}

function toV2Transfer(transfer: Transfer): ToPolkadotV2Transfer {
    return {
        ...transfer,
        input: {
            ...transfer.input,
            fee: toV2DeliveryFee(transfer.input.fee),
        },
        computed: {
            ...transfer.computed,
            claimer: null,
            topic: "",
            totalInputAmount: transfer.input.amount,
        },
    } as unknown as ToPolkadotV2Transfer
}

function toV1Transfer(transfer: ToPolkadotV2Transfer): Transfer {
    const candidate = transfer as unknown as Transfer
    const v1Fee = toV1DeliveryFee(transfer.input.fee)
    if (typeof candidate.computed?.destinationFeeInDOT !== "bigint") {
        throw new Error(
            "Unsupported transfer object for v1 toPolkadot adapter. Expected v1 transfer shape.",
        )
    }
    return {
        ...candidate,
        input: {
            ...candidate.input,
            fee: v1Fee,
        },
    }
}

export class V1ToPolkadotAdapter implements ToPolkadotTransferInterface<EthersProviderTypes> {
    constructor(
        public readonly context: Context<EthersProviderTypes>,
        public readonly registry: AssetRegistry,
        public readonly route: TransferRoute,
        public readonly source: EthereumChain,
        public readonly destination: Parachain,
    ) {}

    get from(): ChainId {
        return this.route.from
    }

    get to(): ChainId {
        return this.route.to
    }

    async getDeliveryFee(
        tokenAddress: string,
        options?: {
            paddFeeByPercentage?: bigint
            feeAsset?: any
            customXcm?: any[]
            overrideRelayerFee?: bigint
        },
    ): Promise<ToPolkadotV2DeliveryFee> {
        if (options?.feeAsset !== undefined) {
            throw new Error("v1 toPolkadot adapter does not support options.feeAsset.")
        }
        if (options?.customXcm !== undefined) {
            throw new Error("v1 toPolkadot adapter does not support options.customXcm.")
        }
        if (options?.overrideRelayerFee !== undefined) {
            throw new Error("v1 toPolkadot adapter does not support options.overrideRelayerFee.")
        }
        const context = this.context
        const registry = this.registry
        const gateway = context.gateway()
        const assetHub = await context.assetHub()
        const destinationApi = await context.parachain(this.destination.id)
        const destParachain = this.destination
        const destAssetMetadata = destParachain.assets[tokenAddress.toLowerCase()]
        if (!destAssetMetadata) {
            throw Error(
                `Token ${tokenAddress} not registered on destination parachain ${destParachain.id}.`,
            )
        }

        let destinationDeliveryFeeDOT = 0n
        let destinationExecutionFeeDOT = 0n
        if (this.to.id !== registry.assetHubParaId) {
            let destinationXcm: any
            if (destAssetMetadata.location) {
                destinationXcm = buildParachainPNAReceivedXcmOnDestination(
                    destinationApi.registry,
                    destAssetMetadata.location,
                    340282366920938463463374607431768211455n,
                    340282366920938463463374607431768211455n,
                    destParachain.info.accountType === "AccountId32"
                        ? "0x0000000000000000000000000000000000000000000000000000000000000000"
                        : "0x0000000000000000000000000000000000000000",
                    "0x0000000000000000000000000000000000000000000000000000000000000000",
                )
            } else {
                destinationXcm = buildParachainERC20ReceivedXcmOnDestination(
                    destinationApi.registry,
                    registry.ethChainId,
                    "0x0000000000000000000000000000000000000000",
                    340282366920938463463374607431768211455n,
                    340282366920938463463374607431768211455n,
                    destParachain.info.accountType === "AccountId32"
                        ? "0x0000000000000000000000000000000000000000000000000000000000000000"
                        : "0x0000000000000000000000000000000000000000",
                    "0x0000000000000000000000000000000000000000000000000000000000000000",
                )
            }

            const assetHubImpl = await this.context.paraImplementation(assetHub)
            destinationDeliveryFeeDOT = await assetHubImpl.calculateDeliveryFeeInDOT(
                this.to.id,
                destinationXcm,
            )
            const destinationImpl = await this.context.paraImplementation(destinationApi)
            destinationExecutionFeeDOT = padFeeByPercentage(
                await destinationImpl.calculateXcmFee(destinationXcm, DOT_LOCATION),
                options?.paddFeeByPercentage ?? 33n,
            )
        }
        const totalFeeInDOT = destinationExecutionFeeDOT + destinationDeliveryFeeDOT
        return toV2DeliveryFee({
            destinationExecutionFeeDOT,
            destinationDeliveryFeeDOT,
            totalFeeInWei: await gateway.quoteSendTokenFee(tokenAddress, this.to.id, totalFeeInDOT),
        })
    }

    async createTransfer(
        sourceAccount: string,
        beneficiaryAccount: string,
        tokenAddress: string,
        amount: bigint,
        fee: ToPolkadotV2DeliveryFee,
        customXcm?: any[],
    ): Promise<ToPolkadotV2Transfer> {
        if (customXcm !== undefined) {
            throw new Error("v1 toPolkadot adapter does not support customXcm.")
        }
        const context = this.context
        const registry = this.registry
        const v1Fee = toV1DeliveryFee(fee)
        const tokenErcMetadata =
            registry.ethereumChains[`ethereum_${registry.ethChainId}`].assets[
                tokenAddress.toLowerCase()
            ]
        if (!tokenErcMetadata) {
            throw Error(
                `No token ${tokenAddress} registered on ethereum chain ${registry.ethChainId}.`,
            )
        }
        const ahAssetMetadata =
            registry.parachains[`polkadot_${registry.assetHubParaId}`].assets[
                tokenAddress.toLowerCase()
            ]
        if (!ahAssetMetadata) {
            throw Error(`Token ${tokenAddress} not registered on asset hub.`)
        }
        const destParachain = this.destination
        const destAssetMetadata = destParachain.assets[tokenAddress.toLowerCase()]
        if (!destAssetMetadata) {
            throw Error(
                `Token ${tokenAddress} not registered on destination parachain ${destParachain.id}.`,
            )
        }
        const minimalBalance =
            ahAssetMetadata.minimumBalance > destAssetMetadata.minimumBalance
                ? ahAssetMetadata.minimumBalance
                : destAssetMetadata.minimumBalance

        let { address: beneficiary, hexAddress: beneficiaryAddressHex } =
            context.ethereumProvider.beneficiaryMultiAddress(beneficiaryAccount)
        let value = v1Fee.totalFeeInWei
        if (tokenAddress === ETHER_TOKEN_ADDRESS) {
            value += amount
        }
        const totalFeeDot = v1Fee.destinationDeliveryFeeDOT + v1Fee.destinationExecutionFeeDOT
        const tx = await context.ethereumProvider.gatewayV1SendToken(
            context.ethereum(),
            context.environment.gatewayContract,
            sourceAccount,
            tokenAddress,
            this.to.id,
            beneficiary,
            totalFeeDot,
            amount,
            value,
        )
        return toV2Transfer({
            input: {
                registry,
                sourceAccount,
                beneficiaryAccount,
                tokenAddress,
                destinationParaId: this.to.id,
                amount,
                fee: v1Fee,
            },
            computed: {
                gatewayAddress: registry.gatewayAddress,
                beneficiaryAddressHex,
                beneficiaryMultiAddress: beneficiary,
                totalValue: value,
                tokenErcMetadata,
                ahAssetMetadata,
                destAssetMetadata,
                minimalBalance,
                destParachain,
                destinationFeeInDOT: totalFeeDot,
            },
            tx,
        })
    }

    async validateTransfer(transfer: ToPolkadotV2Transfer): Promise<ToPolkadotV2ValidationResult> {
        const context = this.context
        const v1Transfer = toV1Transfer(transfer)
        const { tx } = v1Transfer
        const { amount, sourceAccount, tokenAddress, registry, destinationParaId } =
            v1Transfer.input
        const {
            ethereum,
            bridgeHub,
            assetHub,
            destParachain: destParachainApi,
        } = {
            ethereum: context.ethereum(),
            bridgeHub: await context.bridgeHub(),
            assetHub: await context.assetHub(),
            destParachain: await context.parachain(destinationParaId),
        }

        const {
            totalValue,
            minimalBalance,
            destAssetMetadata,
            ahAssetMetadata,
            beneficiaryAddressHex,
        } = v1Transfer.computed

        const logs: ValidationLog[] = []
        if (amount < minimalBalance) {
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.MinimumAmountValidation,
                message: "The amount transferred is less than the minimum amount.",
            })
        }
        const etherBalance = await context.ethereumProvider.getBalance(ethereum, sourceAccount)

        let tokenBalance: { balance: bigint; gatewayAllowance: bigint }
        if (tokenAddress !== ETHER_TOKEN_ADDRESS) {
            tokenBalance = await context.ethereumProvider.erc20Balance(
                ethereum,
                tokenAddress,
                sourceAccount,
                registry.gatewayAddress,
            )
        } else {
            tokenBalance = {
                balance: etherBalance,
                gatewayAllowance: 340282366920938463463374607431768211455n,
            }
        }

        if (tokenBalance.gatewayAllowance < amount && !destAssetMetadata.location) {
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.GatewaySpenderLimitReached,
                message:
                    "The Snowbridge gateway contract needs to approved as a spender for this token and amount.",
            })
        }

        if (tokenBalance.balance < amount) {
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.InsufficientTokenBalance,
                message: "Insufficient token balance to submit transaction.",
            })
        }
        let feeInfo: FeeInfo | undefined
        try {
            const estimatedGas = await context.ethereumProvider.estimateGas(ethereum, tx)
            const feeData = await context.ethereumProvider.getFeeData(ethereum)
            const executionFee =
                feeData.maxFeePerGas !== null
                    ? feeData.maxFeePerGas * estimatedGas
                    : feeData.gasPrice !== null
                      ? feeData.gasPrice * estimatedGas
                      : 0n
            feeInfo = {
                estimatedGas,
                feeData,
                executionFee,
                totalTxCost: executionFee + totalValue,
            }
            if (feeInfo.totalTxCost > etherBalance) {
                logs.push({
                    kind: ValidationKind.Error,
                    reason: ValidationReason.InsufficientEther,
                    message: "Insufficient ether to submit transaction and pay fees.",
                })
            }
        } catch {
            logs.push({
                kind: ValidationKind.Warning,
                reason: ValidationReason.FeeEstimationError,
                message: "Could not estimate transaction fee.",
            })
        }

        const bridgeStatus = await getOperatingStatus({
            gateway: context.gateway(),
            bridgeHub,
        })
        if (
            bridgeStatus.toPolkadot.outbound !== "Normal" ||
            bridgeStatus.toPolkadot.beacon !== "Normal"
        ) {
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.BridgeStatusNotOperational,
                message: "Snowbridge is not currently operational.",
            })
        }

        let assetHubDryRunError: string | undefined
        let destinationParachainDryRunError: string | undefined
        const dryRunAssetHubResult = await dryRunAssetHub(context, assetHub, v1Transfer)
        if (!dryRunAssetHubResult.success) {
            assetHubDryRunError = dryRunAssetHubResult.errorMessage
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.DryRunFailed,
                message: "Dry run call on Asset Hub failed.",
            })
        } else if (destinationParaId !== registry.assetHubParaId) {
            const paraImpl = await this.context.paraImplementation(destParachainApi)
            const dryRunDestinationResult = await dryRunDestination(
                destParachainApi,
                v1Transfer,
                dryRunAssetHubResult.forwardedDestination![1][0],
            )
            if (!dryRunDestinationResult.success) {
                destinationParachainDryRunError = dryRunDestinationResult.errorMessage
                logs.push({
                    kind: ValidationKind.Error,
                    reason: ValidationReason.DryRunFailed,
                    message: "Dry run call on destination parachain failed.",
                })
                if (!destAssetMetadata.isSufficient) {
                    const { accountMaxConsumers, accountExists } = await paraImpl.validateAccount(
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
        }

        const v1Result: ValidationResult = {
            logs,
            success: logs.find((l) => l.kind === ValidationKind.Error) === undefined,
            data: {
                etherBalance,
                tokenBalance,
                feeInfo,
                bridgeStatus,
                assetHubDryRunError,
                destinationParachainDryRunError,
            },
            transfer: v1Transfer,
        }
        return {
            ...v1Result,
            transfer: toV2Transfer(v1Result.transfer),
        } as ToPolkadotV2ValidationResult
    }

    async getMessageReceipt(receipt: TransactionReceipt): Promise<MessageReceipt | null> {
        const context = this.context
        const messageAccepted =
            context.ethereumProvider.scanGatewayV1OutboundMessageAccepted(receipt)
        if (!messageAccepted) return null
        return messageAccepted
    }
}

export function createTransferImplementationV1(
    context: Context<EthersProviderTypes>,
    route: TransferRoute,
    registry: AssetRegistry,
    source: EthereumChain,
    destination: Parachain,
): ToPolkadotTransferInterface {
    return new V1ToPolkadotAdapter(context, registry, route, source, destination)
}

async function dryRunAssetHub(
    context: Context<EthersProviderTypes>,
    assetHub: ApiPromise,
    transfer: Transfer,
) {
    const { registry, amount, tokenAddress, beneficiaryAccount, destinationParaId } = transfer.input
    const { destinationFeeInDOT, destAssetMetadata } = transfer.computed
    const bridgeHubLocation = {
        v4: { parents: 1, interior: { x1: [{ parachain: registry.bridgeHubParaId }] } },
    }
    let xcm: any
    //  taken from chopsticks and based on our exchange rate calculation.
    const baseFee = context.ethereumProvider.parseUnits(
        "0.1",
        transfer.input.registry.relaychain.tokenDecimals,
    )
    const assetHubFee =
        baseFee +
        transfer.input.fee.destinationDeliveryFeeDOT +
        transfer.input.fee.destinationExecutionFeeDOT
    if (destinationParaId !== registry.assetHubParaId) {
        if (destAssetMetadata.location) {
            xcm = buildParachainPNAReceivedXcmOnAssetHub(
                assetHub.registry,
                registry.ethChainId,
                destAssetMetadata.locationOnAH,
                destinationParaId,
                amount,
                assetHubFee,
                destinationFeeInDOT,
                beneficiaryAccount,
                "0x0000000000000000000000000000000000000000000000000000000000000000",
            )
        } else {
            xcm = buildParachainERC20ReceivedXcmOnAssetHub(
                assetHub.registry,
                registry.ethChainId,
                tokenAddress,
                destinationParaId,
                amount,
                assetHubFee,
                destinationFeeInDOT,
                beneficiaryAccount,
                "0x0000000000000000000000000000000000000000000000000000000000000000",
            )
        }
    } else {
        if (destAssetMetadata.location) {
            xcm = buildAssetHubPNAReceivedXcm(
                assetHub.registry,
                registry.ethChainId,
                destAssetMetadata.location,
                amount,
                assetHubFee,
                beneficiaryAccount,
                "0x0000000000000000000000000000000000000000000000000000000000000000",
            )
        } else {
            xcm = buildAssetHubERC20ReceivedXcm(
                assetHub.registry,
                registry.ethChainId,
                tokenAddress,
                amount,
                assetHubFee,
                beneficiaryAccount,
                "0x0000000000000000000000000000000000000000000000000000000000000000",
            )
        }
    }
    const result = await assetHub.call.dryRunApi.dryRunXcm<
        Result<XcmDryRunEffects, XcmDryRunApiError>
    >(bridgeHubLocation, xcm)

    const resultHuman = result.toHuman() as any

    const success = result.isOk && result.asOk.executionResult.isComplete
    let forwardedDestination
    if (!success) {
        console.error("Error during dry run on asset hub:", xcm.toHuman(), result.toHuman())
    } else {
        forwardedDestination = result.asOk.forwardedXcms.find((x) => {
            return (
                x[0].isV4 &&
                x[0].asV4.parents.toNumber() === 1 &&
                x[0].asV4.interior.isX1 &&
                x[0].asV4.interior.asX1[0].isParachain &&
                x[0].asV4.interior.asX1[0].asParachain.toNumber() === destinationParaId
            )
        })
    }
    return {
        success,
        errorMessage: resultHuman.Ok.executionResult.Incomplete?.error,
        forwardedDestination,
    }
}

async function dryRunDestination(destination: ApiPromise, transfer: Transfer, xcm: any) {
    const { registry } = transfer.input
    const assetHubOrigin = {
        v4: { parents: 1, interior: { x1: [{ parachain: registry.assetHubParaId }] } },
    }
    const result = await destination.call.dryRunApi.dryRunXcm<
        Result<XcmDryRunEffects, XcmDryRunApiError>
    >(assetHubOrigin, xcm)

    const resultHuman = result.toHuman() as any

    const success = result.isOk && result.asOk.executionResult.isComplete

    if (!success) {
        console.error("Error during dry run on source parachain:", xcm.toHuman(), result.toHuman())
    }
    return {
        success,
        errorMessage: resultHuman.Ok.executionResult.Incomplete?.error,
    }
}

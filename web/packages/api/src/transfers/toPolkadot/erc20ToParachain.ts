import {
    AssetRegistry,
    ChainId,
    EthereumChain,
    EthereumProviderTypes,
    Parachain,
    TransferRoute,
} from "@snowbridge/base-types"
import { TransferInterface } from "./transferInterface"
import { Context } from "../../index"
import {
    buildMessageId,
    calculateRelayerFee,
    claimerFromBeneficiary,
    claimerLocationToBytes,
    DeliveryFee,
    messageId as getSharedMessageReceipt,
    Transfer,
    ValidationKind,
    ValidatedTransfer,
} from "../../toPolkadotSnowbridgeV2"
import { accountId32Location, DOT_LOCATION, erc20Location, isDOT } from "../../xcmBuilder"
import { ETHER_TOKEN_ADDRESS } from "../../assets_v2"
import { ensureValidationSuccess, padFeeByPercentage, paraIdToSovereignAccount } from "../../utils"
import { FeeInfo, ValidationLog, ValidationReason } from "../../toPolkadot_v2"
import {
    buildAssetHubERC20ReceivedXcm,
    buildParachainERC20ReceivedXcmOnDestWithDOTFee,
    buildParachainERC20ReceivedXcmOnDestination,
    buildParachainERC20ReceivedXcmOnDestinationWithDOTFee,
} from "../../xcmbuilders/toPolkadot/erc20ToParachain"
import {
    sendMessageXCM,
    sendMessageXCMWithDOTDestFee,
} from "../../xcmbuilders/toPolkadot/erc20ToParachain"
import { getOperatingStatus } from "../../status"
import { hexToU8a } from "@polkadot/util"

export class ERC20ToParachain<T extends EthereumProviderTypes> implements TransferInterface<T> {
    constructor(
        public readonly context: Context<T>,
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

    async fee(
        tokenAddress: string,
        options?: {
            paddFeeByPercentage?: bigint
            feeAsset?: any
            customXcm?: any[]
            overrideRelayerFee?: bigint
        },
    ): Promise<DeliveryFee> {
        const context = this.context
        const registry = this.registry
        const assetHub = await context.assetHub()
        const bridgeHub = await context.bridgeHub()
        const destination = await context.parachain(this.to.id)
        const destParachain = this.destination
        const destAssetMetadata = destParachain.assets[tokenAddress.toLowerCase()]
        if (!destAssetMetadata) {
            throw Error(
                `Token ${tokenAddress} not registered on destination parachain ${destParachain.id}.`,
            )
        }

        // AssetHub fees
        let assetHubXcm = buildAssetHubERC20ReceivedXcm(
            assetHub.registry,
            registry.ethChainId,
            tokenAddress,
            3000000000000n,
            1000000000000n,
            1000000000000n,
            accountId32Location(
                "0x0000000000000000000000000000000000000000000000000000000000000000",
            ),
            "0x0000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            this.to.id,
            1000000000000n,
            "0x0000000000000000000000000000000000000000000000000000000000000000",
        )
        const bridgeHubImpl = await this.context.paraImplementation(bridgeHub)
        const assetHubImpl = await this.context.paraImplementation(assetHub)
        let ether = erc20Location(registry.ethChainId, ETHER_TOKEN_ADDRESS)
        const paddFeeByPercentage = options?.paddFeeByPercentage
        const feeAsset = options?.feeAsset || ether

        // Delivery fee BridgeHub to AssetHub
        const deliveryFeeInDOT = await bridgeHubImpl.calculateDeliveryFeeInDOT(
            registry.assetHubParaId,
            assetHubXcm,
        )
        // AssetHub execution fee
        let assetHubExecutionFeeDOT = await assetHubImpl.calculateXcmFee(assetHubXcm, DOT_LOCATION)
        // Swap to ether
        const deliveryFeeInEther = await assetHubImpl.swapAsset1ForAsset2(
            DOT_LOCATION,
            ether,
            deliveryFeeInDOT,
        )
        let assetHubExecutionFeeEther = padFeeByPercentage(
            await assetHubImpl.swapAsset1ForAsset2(DOT_LOCATION, ether, assetHubExecutionFeeDOT),
            paddFeeByPercentage ?? 33n,
        )

        let destinationXcm: any
        // Destination fees
        if (isDOT(feeAsset)) {
            destinationXcm = buildParachainERC20ReceivedXcmOnDestinationWithDOTFee(
                destination.registry,
                registry.ethChainId,
                "0x0000000000000000000000000000000000000000",
                3402823669209384634633746074317682114n,
                3402823669209384634633746074317682114n,
                destParachain.info.accountType === "AccountId32"
                    ? "0x0000000000000000000000000000000000000000000000000000000000000000"
                    : "0x0000000000000000000000000000000000000000",
                "0x0000000000000000000000000000000000000000000000000000000000000000",
                options?.customXcm,
            )
        } else {
            destinationXcm = buildParachainERC20ReceivedXcmOnDestination(
                destination.registry,
                registry.ethChainId,
                "0x0000000000000000000000000000000000000000",
                3402823669209384634633746074317682114n,
                3402823669209384634633746074317682114n,
                destParachain.info.accountType === "AccountId32"
                    ? "0x0000000000000000000000000000000000000000000000000000000000000000"
                    : "0x0000000000000000000000000000000000000000",
                "0x0000000000000000000000000000000000000000000000000000000000000000",
                options?.customXcm,
            )
        }

        const destinationImpl = await this.context.paraImplementation(destination)
        // Delivery fee AssetHub to Destination
        let destinationDeliveryFeeDOT = await assetHubImpl.calculateDeliveryFeeInDOT(
            this.to.id,
            destinationXcm,
        )

        // Swap to ether
        const destinationDeliveryFeeEther = await assetHubImpl.swapAsset1ForAsset2(
            DOT_LOCATION,
            ether,
            destinationDeliveryFeeDOT,
        )

        let destinationExecutionFeeEther
        let destinationExecutionFeeDOT
        if (isDOT(feeAsset)) {
            // Calculate ether fee on AssetHub, because that is where Ether will be swapped for DOT.
            // Destination execution fee
            destinationExecutionFeeDOT = await destinationImpl.calculateXcmFee(
                destinationXcm,
                DOT_LOCATION,
            )
            destinationExecutionFeeEther = padFeeByPercentage(
                await assetHubImpl.swapAsset1ForAsset2(
                    DOT_LOCATION,
                    ether,
                    destinationExecutionFeeDOT,
                ),
                paddFeeByPercentage ?? 33n,
            )
        } else if (feeAsset == ether) {
            destinationExecutionFeeEther = padFeeByPercentage(
                await destinationImpl.calculateXcmFee(destinationXcm, ether),
                paddFeeByPercentage ?? 33n,
            )
        } else {
            throw Error(`Unsupported fee asset`)
        }

        const { relayerFee, extrinsicFeeDot, extrinsicFeeEther } = await calculateRelayerFee(
            assetHubImpl,
            registry.ethChainId,
            options?.overrideRelayerFee,
            deliveryFeeInEther,
        )

        const totalFeeInWei =
            assetHubExecutionFeeEther +
            destinationDeliveryFeeEther +
            destinationExecutionFeeEther +
            relayerFee
        return {
            assetHubDeliveryFeeEther: deliveryFeeInEther,
            assetHubExecutionFeeEther: assetHubExecutionFeeEther,
            destinationDeliveryFeeEther: destinationDeliveryFeeEther,
            destinationExecutionFeeEther: destinationExecutionFeeEther,
            destinationExecutionFeeDOT: destinationExecutionFeeDOT,
            relayerFee: relayerFee,
            extrinsicFeeDot: extrinsicFeeDot,
            extrinsicFeeEther: extrinsicFeeEther,
            totalFeeInWei: totalFeeInWei,
            feeAsset: feeAsset,
        }
    }

    async tx(
        sourceAccount: string,
        beneficiaryAccount: string,
        tokenAddress: string,
        amount: bigint,
        fee: DeliveryFee,
        customXcm?: any[],
    ): Promise<Transfer<T>> {
        const context = this.context
        const registry = this.registry
        const ethereum = context.ethereum()
        const assetHub = await context.assetHub()
        const destination = await context.parachain(this.to.id)

        if (!destination) {
            throw Error(`Unable to connect to destination parachain with ID ${this.to.id}.`)
        }
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
        let value = fee.totalFeeInWei
        let inputAmount = amount
        const assets: string[] = []
        if (tokenAddress === ETHER_TOKEN_ADDRESS) {
            value += amount
            inputAmount += fee.totalFeeInWei
        } else {
            assets.push(context.ethereumProvider.encodeNativeAsset(tokenAddress, amount))
        }
        const accountNonce = await context.ethereumProvider.getTransactionCount(
            ethereum,
            sourceAccount,
            "pending",
        )
        const topic = buildMessageId(
            this.to.id,
            sourceAccount,
            tokenAddress,
            beneficiaryAddressHex,
            amount,
            accountNonce,
        )

        let xcm
        if (isDOT(fee.feeAsset)) {
            xcm = hexToU8a(
                sendMessageXCMWithDOTDestFee(
                    destination.registry,
                    registry.ethChainId,
                    this.to.id,
                    tokenAddress,
                    beneficiaryAddressHex,
                    amount,
                    fee.destinationExecutionFeeEther ?? 0n,
                    fee.destinationExecutionFeeDOT ?? 0n,
                    topic,
                ).toHex(),
            )
        } else {
            xcm = hexToU8a(
                sendMessageXCM(
                    destination.registry,
                    registry.ethChainId,
                    this.to.id,
                    tokenAddress,
                    beneficiaryAddressHex,
                    amount,
                    fee.destinationExecutionFeeEther ?? 0n,
                    topic,
                    customXcm,
                ).toHex(),
            )
        }
        let claimer = claimerFromBeneficiary(assetHub, beneficiaryAddressHex)

        const tx = await context.ethereumProvider.gatewayV2SendMessage(
            context.ethereum(),
            context.environment.gatewayContract,
            sourceAccount,
            xcm,
            assets,
            claimerLocationToBytes(claimer),
            fee.assetHubExecutionFeeEther + fee.destinationDeliveryFeeEther,
            fee.relayerFee,
            value,
        )

        return {
            kind: "ethereum->polkadot",
            input: {
                registry,
                sourceAccount,
                beneficiaryAccount,
                tokenAddress,
                destinationParaId: this.to.id,
                amount,
                fee,
                customXcm,
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
                claimer,
                topic,
                totalInputAmount: inputAmount,
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
                paddFeeByPercentage?: bigint
                feeAsset?: any
                customXcm?: any[]
                overrideRelayerFee?: bigint
            }
            customXcm?: any[]
        },
    ): Promise<ValidatedTransfer<T>> {
        const fee = await this.fee(tokenAddress, options?.fee)
        const transfer = await this.tx(
            sourceAccount,
            beneficiaryAccount,
            tokenAddress,
            amount,
            fee,
            options?.customXcm,
        )
        return ensureValidationSuccess(await this.validate(transfer))
    }

    async validate(transfer: Transfer<T>): Promise<ValidatedTransfer<T>> {
        const context = this.context
        const { tx } = transfer
        const { amount, sourceAccount, tokenAddress, registry, destinationParaId } = transfer.input
        const ethereum = context.ethereum()
        const gateway = context.gateway()
        const bridgeHub = await context.bridgeHub()
        const assetHub = await context.assetHub()
        const destParachainApi = await context.parachain(destinationParaId)

        const {
            totalValue,
            minimalBalance,
            destParachain,
            destAssetMetadata,
            ahAssetMetadata,
            beneficiaryAddressHex,
            claimer,
        } = transfer.computed

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
                // u128 max
                gatewayAllowance: 340282366920938463463374607431768211455n,
            }
        }
        if (tokenBalance.gatewayAllowance < amount) {
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
                message: "The amount transferred is greater than the users token balance.",
            })
        }
        let feeInfo: FeeInfo | undefined
        if (logs.length === 0) {
            const [estimatedGas, feeData] = await Promise.all([
                context.ethereumProvider.estimateGas(ethereum, tx),
                context.ethereumProvider.getFeeData(ethereum),
            ])
            const executionFee = (feeData.gasPrice ?? 0n) * estimatedGas
            if (executionFee === 0n) {
                logs.push({
                    kind: ValidationKind.Error,
                    reason: ValidationReason.FeeEstimationError,
                    message: "Could not get fetch fee details.",
                })
            }
            const totalTxCost = totalValue + executionFee
            if (etherBalance < totalTxCost) {
                logs.push({
                    kind: ValidationKind.Error,
                    reason: ValidationReason.InsufficientEther,
                    message: "Insufficient ether to submit transaction.",
                })
            }
            feeInfo = {
                estimatedGas,
                feeData,
                executionFee,
                totalTxCost,
            }
        }
        const bridgeStatus = await getOperatingStatus({ gateway, bridgeHub })
        if (
            bridgeStatus.toPolkadot.outbound !== "Normal" ||
            bridgeStatus.toPolkadot.beacon !== "Normal"
        ) {
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.BridgeStatusNotOperational,
                message: "Bridge operations have been paused by onchain governance.",
            })
        }

        // Check if asset can be received on asset hub (dry run)
        const ahParachain = registry.parachains[`polkadot_${registry.assetHubParaId}`]
        const assetHubImpl = await this.context.paraImplementation(assetHub)
        let dryRunAhSuccess, forwardedDestination, assetHubDryRunError
        if (!ahParachain.features.hasDryRunApi) {
            logs.push({
                kind: ValidationKind.Warning,
                reason: ValidationReason.DryRunNotSupportedOnDestination,
                message:
                    "Asset Hub does not support dry running of XCM. Transaction success cannot be confirmed.",
            })
        } else {
            // build asset hub packet and dryRun
            const assetHubFee =
                transfer.input.fee.assetHubExecutionFeeEther +
                transfer.input.fee.destinationDeliveryFeeEther

            let xcm
            if (isDOT(transfer.input.fee.feeAsset)) {
                xcm = buildParachainERC20ReceivedXcmOnDestWithDOTFee(
                    assetHub.registry,
                    registry.ethChainId,
                    tokenAddress,
                    transfer.computed.totalValue - assetHubFee,
                    assetHubFee,
                    amount,
                    claimer,
                    transfer.input.sourceAccount,
                    transfer.computed.beneficiaryAddressHex,
                    destinationParaId,
                    transfer.input.fee.destinationExecutionFeeEther ?? 0n,
                    transfer.input.fee.destinationExecutionFeeDOT ?? 0n,
                    "0x0000000000000000000000000000000000000000000000000000000000000000",
                    transfer.input.customXcm,
                )
            } else {
                xcm = buildAssetHubERC20ReceivedXcm(
                    assetHub.registry,
                    registry.ethChainId,
                    tokenAddress,
                    transfer.computed.totalValue - assetHubFee,
                    assetHubFee,
                    amount,
                    claimer,
                    transfer.input.sourceAccount,
                    transfer.computed.beneficiaryAddressHex,
                    destinationParaId,
                    transfer.input.fee.destinationExecutionFeeEther ?? 0n,
                    "0x0000000000000000000000000000000000000000000000000000000000000000",
                    transfer.input.customXcm,
                )
            }
            let result = await assetHubImpl.dryRunXcm(
                registry.bridgeHubParaId,
                xcm,
                destinationParaId,
            )
            dryRunAhSuccess = result.success
            assetHubDryRunError = result.errorMessage
            forwardedDestination = result.forwardedDestination
            if (!dryRunAhSuccess) {
                logs.push({
                    kind: ValidationKind.Error,
                    reason: ValidationReason.DryRunFailed,
                    message: "Dry run on Asset Hub failed.",
                })
            }
        }

        let destinationParachainDryRunError: string | undefined
        // Check if sovereign account balance for token is at 0 and that consumers is maxxed out.
        if (!ahAssetMetadata.isSufficient && !dryRunAhSuccess) {
            const sovereignAccountId = paraIdToSovereignAccount("sibl", destinationParaId)
            const { accountMaxConsumers, accountExists } = await assetHubImpl.validateAccount(
                sovereignAccountId,
                registry.ethChainId,
                tokenAddress,
                ahAssetMetadata,
            )

            if (!accountExists) {
                logs.push({
                    kind: ValidationKind.Error,
                    reason: ValidationReason.MaxConsumersReached,
                    message: "Sovereign account does not exist on Asset Hub.",
                })
            }
            if (accountMaxConsumers) {
                logs.push({
                    kind: ValidationKind.Error,
                    reason: ValidationReason.MaxConsumersReached,
                    message:
                        "Sovereign account for destination parachain has reached the max consumer limit on Asset Hub.",
                })
            }
        }
        if (!destParachainApi) {
            logs.push({
                kind: ValidationKind.Warning,
                reason: ValidationReason.NoDestinationParachainConnection,
                message:
                    "The destination parachain connection was not supplied. Transaction success cannot be confirmed.",
            })
        } else {
            if (destParachain.features.hasDryRunApi) {
                if (!forwardedDestination) {
                    logs.push({
                        kind: ValidationKind.Error,
                        reason: ValidationReason.DryRunFailed,
                        message:
                            "Dry run on Asset Hub did not produce an XCM to be forwarded to the destination parachain.",
                    })
                } else {
                    const xcm = forwardedDestination[1]
                    if (xcm.length !== 1) {
                        logs.push({
                            kind: ValidationKind.Error,
                            reason: ValidationReason.DryRunFailed,
                            message:
                                "Dry run on Asset Hub did not produce an XCM to be forwarded to the destination parachain.",
                        })
                    }
                    const destParachainImpl =
                        await this.context.paraImplementation(destParachainApi)
                    const { success: dryRunDestinationSuccess, errorMessage: destMessage } =
                        await destParachainImpl.dryRunXcm(registry.assetHubParaId, xcm[0])
                    if (!dryRunDestinationSuccess) {
                        logs.push({
                            kind: ValidationKind.Error,
                            reason: ValidationReason.DryRunFailed,
                            message: "Dry run on destination parachain failed.",
                        })
                    }
                    destinationParachainDryRunError = destMessage
                }
            } else {
                logs.push({
                    kind: ValidationKind.Warning,
                    reason: ValidationReason.DryRunNotSupportedOnDestination,
                    message:
                        "The destination parachain does not support dry running of XCM. Transaction success cannot be confirmed.",
                })
            }
            if (
                !destAssetMetadata.isSufficient &&
                ((destParachain.features.hasDryRunApi && destinationParachainDryRunError) ||
                    !destParachain.features.hasDryRunApi)
            ) {
                const destParachainImpl = await this.context.paraImplementation(destParachainApi)
                // Check if the account is created
                const { accountMaxConsumers, accountExists } =
                    await destParachainImpl.validateAccount(
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

        const success = logs.find((l) => l.kind === ValidationKind.Error) === undefined

        return {
            logs,
            success,
            data: {
                etherBalance,
                tokenBalance,
                feeInfo,
                bridgeStatus,
                assetHubDryRunError,
                destinationParachainDryRunError,
            },
            ...transfer,
        }
    }

    async messageId(receipt: T["TransactionReceipt"]) {
        return getSharedMessageReceipt(this.context.ethereumProvider, receipt)
    }
}

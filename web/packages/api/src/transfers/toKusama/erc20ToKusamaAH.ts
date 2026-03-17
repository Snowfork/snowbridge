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
    messageId as getSharedMessageReceipt,
} from "../../toPolkadotSnowbridgeV2"
import { accountId32Location, DOT_LOCATION, erc20Location } from "../../xcmBuilder"
import { ETHER_TOKEN_ADDRESS } from "../../assets_v2"
import {
    ensureValidationSuccess,
    padFeeByPercentage,
    resolveBeneficiary,
} from "../../utils"
import { FeeInfo, ValidationLog, ValidationReason } from "../../toPolkadot_v2"
import {
    buildAssetHubERC20ReceivedXcmForKusama,
    sendMessageXCM,
} from "../../xcmbuilders/toKusama/erc20ToKusamaAH"
import { getOperatingStatus } from "../../status"
import { hexToU8a } from "@polkadot/util"
import {
    DeliveryFee,
    Transfer,
    ValidatedTransfer,
    MessageReceipt,
} from "../../toKusamaSnowbridgeV2"
import { ValidationKind } from "../../toPolkadot_v2"

export class ERC20ToKusamaAH<T extends EthereumProviderTypes> implements TransferInterface<T> {
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
            overrideRelayerFee?: bigint
        },
    ): Promise<DeliveryFee> {
        const context = this.context
        const registry = this.registry

        if (!registry.kusama) {
            throw Error("Kusama config is not set in the registry.")
        }
        const kusamaAHParaId = registry.kusama.assetHubParaId

        const assetHub = await context.assetHub()
        const bridgeHub = await context.bridgeHub()
        const kusamaAssetHub = await context.kusamaAssetHub()

        // Build AssetHub XCM for fee estimation
        let assetHubXcm = buildAssetHubERC20ReceivedXcmForKusama(
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
            kusamaAHParaId,
            1000000000000n,
            "0x0000000000000000000000000000000000000000000000000000000000000000",
        )

        const bridgeHubImpl = await this.context.paraImplementation(bridgeHub)
        const assetHubImpl = await this.context.paraImplementation(assetHub)
        const kusamaAHImpl = await this.context.paraImplementation(kusamaAssetHub)
        let ether = erc20Location(registry.ethChainId, ETHER_TOKEN_ADDRESS)
        const paddFeeByPercentage = options?.paddFeeByPercentage

        // Delivery fee BridgeHub to AssetHub
        const deliveryFeeInDOT = await bridgeHubImpl.calculateDeliveryFeeInDOT(
            registry.assetHubParaId,
            assetHubXcm,
        )
        const deliveryFeeInEther = await assetHubImpl.swapAsset1ForAsset2(
            DOT_LOCATION,
            ether,
            deliveryFeeInDOT,
        )

        // AssetHub execution fee
        let assetHubExecutionFeeDOT = await assetHubImpl.calculateXcmFee(assetHubXcm, DOT_LOCATION)
        let assetHubExecutionFeeEther = padFeeByPercentage(
            await assetHubImpl.swapAsset1ForAsset2(DOT_LOCATION, ether, assetHubExecutionFeeDOT),
            paddFeeByPercentage ?? 33n,
        )

        // Kusama AH delivery fee (via Polkadot↔Kusama bridge)
        // This is the fee for the Polkadot AH → Kusama AH bridge hop
        let kusamaDeliveryFeeDOT = await assetHubImpl.calculateDeliveryFeeInDOT(
            registry.bridgeHubParaId,
            assetHubXcm,
        )
        let kusamaDeliveryFeeEther = padFeeByPercentage(
            await assetHubImpl.swapAsset1ForAsset2(DOT_LOCATION, ether, kusamaDeliveryFeeDOT),
            paddFeeByPercentage ?? 33n,
        )

        // Kusama AH execution fee
        // Estimate from the Kusama AH side
        let kusamaExecutionFeeEther = padFeeByPercentage(
            await kusamaAHImpl.calculateXcmFee(assetHubXcm, ether),
            paddFeeByPercentage ?? 33n,
        )

        const { relayerFee, extrinsicFeeDot, extrinsicFeeEther } = await calculateRelayerFee(
            assetHubImpl,
            registry.ethChainId,
            options?.overrideRelayerFee,
            deliveryFeeInEther,
        )

        const totalFeeInWei =
            assetHubExecutionFeeEther +
            kusamaDeliveryFeeEther +
            kusamaExecutionFeeEther +
            relayerFee

        return {
            kind: "ethereum->kusama",
            assetHubDeliveryFeeEther: deliveryFeeInEther,
            assetHubExecutionFeeEther,
            kusamaDeliveryFeeEther,
            kusamaExecutionFeeEther,
            relayerFee,
            extrinsicFeeDot,
            extrinsicFeeEther,
            totalFeeInWei,
            feeAsset: ether,
        }
    }

    async tx(
        sourceAccount: string,
        beneficiaryAccount: string,
        tokenAddress: string,
        amount: bigint,
        fee: DeliveryFee,
    ): Promise<Transfer<T>> {
        const context = this.context
        const registry = this.registry
        const ethereum = context.ethereum()
        const assetHub = await context.assetHub()

        if (!registry.kusama) {
            throw Error("Kusama config is not set in the registry.")
        }
        const kusamaAHParaId = registry.kusama.assetHubParaId

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
        const kusamaAssetMetadata =
            registry.kusama.parachains[`kusama_${kusamaAHParaId}`]?.assets[
                tokenAddress.toLowerCase()
            ]
        if (!kusamaAssetMetadata) {
            throw Error(`Token ${tokenAddress} not registered on Kusama asset hub.`)
        }
        const minimalBalance =
            ahAssetMetadata.minimumBalance > kusamaAssetMetadata.minimumBalance
                ? ahAssetMetadata.minimumBalance
                : kusamaAssetMetadata.minimumBalance

        const { hexAddress: beneficiaryAddressHex } = resolveBeneficiary(beneficiaryAccount)
        const beneficiary = context.ethereumProvider.beneficiaryMultiAddress(beneficiaryAddressHex)
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
            kusamaAHParaId,
            sourceAccount,
            tokenAddress,
            beneficiaryAddressHex,
            amount,
            accountNonce,
        )

        const kusamaAssetHub = await context.kusamaAssetHub()
        let xcm = hexToU8a(
            sendMessageXCM(
                kusamaAssetHub.registry,
                registry.ethChainId,
                kusamaAHParaId,
                tokenAddress,
                beneficiaryAddressHex,
                amount,
                fee.kusamaExecutionFeeEther,
                topic,
            ).toHex(),
        )
        let claimer = claimerFromBeneficiary(assetHub, beneficiaryAddressHex)

        const tx = await context.ethereumProvider.gatewayV2SendMessage(
            context.ethereum(),
            context.environment.gatewayContract,
            sourceAccount,
            xcm,
            assets,
            claimerLocationToBytes(claimer),
            fee.assetHubExecutionFeeEther + fee.kusamaDeliveryFeeEther,
            fee.relayerFee,
            value,
        )

        return {
            kind: "ethereum->kusama",
            input: {
                registry,
                sourceAccount,
                beneficiaryAccount,
                tokenAddress,
                destinationParaId: kusamaAHParaId,
                amount,
                fee,
            },
            computed: {
                gatewayAddress: registry.gatewayAddress,
                beneficiaryAddressHex,
                beneficiaryMultiAddress: beneficiary,
                totalValue: value,
                tokenErcMetadata,
                ahAssetMetadata,
                kusamaAssetMetadata,
                minimalBalance,
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
                overrideRelayerFee?: bigint
            }
        },
    ): Promise<ValidatedTransfer<T>> {
        const fee = await this.fee(tokenAddress, options?.fee)
        const transfer = await this.tx(
            sourceAccount,
            beneficiaryAccount,
            tokenAddress,
            amount,
            fee,
        )
        return ensureValidationSuccess(await this.validate(transfer))
    }

    async validate(transfer: Transfer<T>): Promise<ValidatedTransfer<T>> {
        const context = this.context
        const { tx } = transfer
        const { amount, sourceAccount, tokenAddress, registry } = transfer.input
        const ethereum = context.ethereum()
        const gateway = context.gateway()
        const bridgeHub = await context.bridgeHub()
        const assetHub = await context.assetHub()

        const { totalValue, minimalBalance, beneficiaryAddressHex, claimer } = transfer.computed

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

        // Dry run on Polkadot AH
        if (!registry.kusama) {
            throw Error("Kusama config is not set in the registry.")
        }
        const kusamaAHParaId = registry.kusama.assetHubParaId
        const assetHubImpl = await this.context.paraImplementation(assetHub)
        let assetHubDryRunError: string | undefined

        const ahParachain = registry.parachains[`polkadot_${registry.assetHubParaId}`]
        if (ahParachain.features.hasDryRunApi) {
            const assetHubFee =
                transfer.input.fee.assetHubExecutionFeeEther +
                transfer.input.fee.kusamaDeliveryFeeEther

            let xcm = buildAssetHubERC20ReceivedXcmForKusama(
                assetHub.registry,
                registry.ethChainId,
                tokenAddress,
                transfer.computed.totalValue - assetHubFee,
                assetHubFee,
                amount,
                claimer,
                transfer.input.sourceAccount,
                transfer.computed.beneficiaryAddressHex,
                kusamaAHParaId,
                transfer.input.fee.kusamaExecutionFeeEther,
                "0x0000000000000000000000000000000000000000000000000000000000000000",
            )
            let result = await assetHubImpl.dryRunXcm(
                registry.bridgeHubParaId,
                xcm,
                kusamaAHParaId,
            )
            if (!result.success) {
                assetHubDryRunError = result.errorMessage
                logs.push({
                    kind: ValidationKind.Error,
                    reason: ValidationReason.DryRunFailed,
                    message: "Dry run on Asset Hub failed.",
                })
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
            },
            ...transfer,
        }
    }

    async messageId(receipt: T["TransactionReceipt"]) {
        return getSharedMessageReceipt(this.context.ethereumProvider, receipt)
    }
}

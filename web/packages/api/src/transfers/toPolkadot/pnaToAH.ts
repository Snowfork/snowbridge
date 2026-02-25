import { AssetRegistry } from "@snowbridge/base-types"
import { TransferInterface } from "./transferInterface"
import { EthersContext } from "../../index"
import {
    buildMessageId,
    calculateRelayerFee,
    claimerFromBeneficiary,
    claimerLocationToBytes,
    DeliveryFee,
    encodeNativeAsset,
    Transfer,
    ValidationKind,
    ValidationResult,
} from "../../toPolkadotSnowbridgeV2"
import { accountId32Location, DOT_LOCATION, erc20Location } from "../../xcmBuilder"
import { paraImplementation } from "../../parachains"
import { ETHER_TOKEN_ADDRESS } from "../../assets_v2"
import { beneficiaryMultiAddress } from "../../EthereumProvider"
import { padFeeByPercentage } from "../../utils"
import { FeeInfo, resolveInputs, ValidationLog, ValidationReason } from "../../toPolkadot_v2"
import { buildAssetHubPNAReceivedXcm, sendMessageXCM } from "../../xcmbuilders/toPolkadot/pnaToAH"
import { getOperatingStatus } from "../../status"
import { hexToU8a } from "@polkadot/util"

export class PNAToAH implements TransferInterface {
    async getDeliveryFee(
        context: EthersContext,
        registry: AssetRegistry,
        tokenAddress: string,
        _destinationParaId: number,
        options?: {
            paddFeeByPercentage?: bigint
            feeAsset?: any
            customXcm?: any[]
            overrideRelayerFee?: bigint
        },
    ): Promise<DeliveryFee> {
        const assetHub = await context.assetHub()
        const bridgeHub = await context.bridgeHub()

        const ahAssetMetadata =
            registry.parachains[`polkadot_${registry.assetHubParaId}`].assets[
                tokenAddress.toLowerCase()
            ]
        if (!ahAssetMetadata) {
            throw Error(`Token ${tokenAddress} not registered on asset hub.`)
        }

        let assetHubXcm = buildAssetHubPNAReceivedXcm(
            assetHub.registry,
            registry.ethChainId,
            ahAssetMetadata.location,
            3000000000000n,
            1000000000000n,
            1000000000000n,
            accountId32Location(
                "0x0000000000000000000000000000000000000000000000000000000000000000",
            ),
            "0x0000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000000000000000000000000000",
        )
        let ether = erc20Location(registry.ethChainId, ETHER_TOKEN_ADDRESS)
        const paddFeeByPercentage = options?.paddFeeByPercentage
        const feeAsset = options?.feeAsset || ether

        if (feeAsset !== ether) {
            throw new Error("only ether is supported as fee asset in this version of the API")
        }

        // Delivery fee BridgeHub to AssetHub
        const bridgeHubImpl = await paraImplementation(bridgeHub)
        const deliveryFeeInDOT = await bridgeHubImpl.calculateDeliveryFeeInDOT(
            registry.assetHubParaId,
            assetHubXcm,
        )

        const assetHubImpl = await paraImplementation(assetHub)
        const deliveryFeeInEther = await assetHubImpl.swapAsset1ForAsset2(
            DOT_LOCATION,
            ether,
            deliveryFeeInDOT,
        )
        // AssetHub Execution fee
        let assetHubExecutionFeeDOT = await assetHubImpl.calculateXcmFee(assetHubXcm, DOT_LOCATION)

        let assetHubExecutionFeeEther = padFeeByPercentage(
            await assetHubImpl.swapAsset1ForAsset2(DOT_LOCATION, ether, assetHubExecutionFeeDOT),
            paddFeeByPercentage ?? 33n,
        )

        const { relayerFee, extrinsicFeeDot, extrinsicFeeEther } = await calculateRelayerFee(
            assetHubImpl,
            registry.ethChainId,
            options?.overrideRelayerFee,
            deliveryFeeInEther,
        )

        const totalFeeInWei = assetHubExecutionFeeEther + relayerFee
        return {
            assetHubDeliveryFeeEther: deliveryFeeInEther,
            assetHubExecutionFeeEther: assetHubExecutionFeeEther,
            destinationDeliveryFeeEther: 0n,
            destinationExecutionFeeEther: 0n,
            relayerFee: relayerFee,
            extrinsicFeeDot: extrinsicFeeDot,
            extrinsicFeeEther: extrinsicFeeEther,
            totalFeeInWei: totalFeeInWei,
            feeAsset: feeAsset,
        }
    }

    async createTransfer(
        context: EthersContext,
        registry: AssetRegistry,
        destinationParaId: number,
        sourceAccount: string,
        beneficiaryAccount: string,
        tokenAddress: string,
        amount: bigint,
        fee: DeliveryFee,
        customXcm?: any[],
    ): Promise<Transfer> {
        const ethereum = context.ethereum()
        const assetHub = await context.assetHub()

        const { tokenErcMetadata, destParachain, ahAssetMetadata, destAssetMetadata } =
            resolveInputs(registry, tokenAddress, destinationParaId)
        const minimalBalance =
            ahAssetMetadata.minimumBalance > destAssetMetadata.minimumBalance
                ? ahAssetMetadata.minimumBalance
                : destAssetMetadata.minimumBalance

        let { address: beneficiary, hexAddress: beneficiaryAddressHex } =
            beneficiaryMultiAddress(beneficiaryAccount)
        let value = fee.assetHubExecutionFeeEther + fee.assetHubDeliveryFeeEther + fee.relayerFee

        const con = context.gatewayV2()

        if (!ahAssetMetadata.foreignId) {
            throw Error("asset foreign ID not set in metadata")
        }

        const accountNonce = await ethereum.getTransactionCount(sourceAccount, "pending")
        const topic = buildMessageId(
            destinationParaId,
            sourceAccount,
            tokenAddress,
            beneficiaryAddressHex,
            amount,
            accountNonce,
        )

        const xcm = hexToU8a(
            sendMessageXCM(assetHub.registry, beneficiaryAddressHex, topic, customXcm).toHex(),
        )
        let assets = [encodeNativeAsset(tokenAddress, amount)]
        let claimer = claimerFromBeneficiary(assetHub, beneficiaryAddressHex)

        const tx = await context.ethereumProvider.populateTransaction(
            con,
            "v2_sendMessage",
            xcm,
            assets,
            claimerLocationToBytes(claimer),
            fee.assetHubExecutionFeeEther,
            fee.relayerFee,
            {
                value,
                from: sourceAccount,
            },
        )

        return {
            input: {
                registry,
                sourceAccount,
                beneficiaryAccount,
                tokenAddress,
                destinationParaId,
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
                totalInputAmount: amount,
            },
            tx,
        }
    }

    async validateTransfer(context: EthersContext, transfer: Transfer): Promise<ValidationResult> {
        const { tx } = transfer
        const { amount, sourceAccount, tokenAddress, registry } = transfer.input
        const ethereum = context.ethereum()
        const gateway = context.gateway()
        const bridgeHub = await context.bridgeHub()
        const assetHub = await context.assetHub()

        const { totalValue, minimalBalance, ahAssetMetadata, beneficiaryAddressHex, claimer } =
            transfer.computed

        const logs: ValidationLog[] = []
        if (amount < minimalBalance) {
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.MinimumAmountValidation,
                message: "The amount transferred is less than the minimum amount.",
            })
        }
        const etherBalance = await ethereum.getBalance(sourceAccount)

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
        const assetHubImpl = await paraImplementation(assetHub)
        let dryRunAhSuccess, assetHubDryRunError
        if (!ahParachain.features.hasDryRunApi) {
            logs.push({
                kind: ValidationKind.Warning,
                reason: ValidationReason.DryRunNotSupportedOnDestination,
                message:
                    "Asset Hub does not support dry running of XCM. Transaction success cannot be confirmed.",
            })
        } else {
            const assetHubFee =
                transfer.input.fee.assetHubDeliveryFeeEther +
                transfer.input.fee.assetHubExecutionFeeEther
            const xcm = buildAssetHubPNAReceivedXcm(
                assetHub.registry,
                registry.ethChainId,
                ahAssetMetadata.location,
                transfer.computed.totalValue - assetHubFee,
                assetHubFee,
                amount,
                claimer,
                transfer.input.sourceAccount,
                transfer.computed.beneficiaryAddressHex,
                "0x0000000000000000000000000000000000000000000000000000000000000000",
                transfer.input.customXcm,
            )

            let result = await assetHubImpl.dryRunXcm(registry.bridgeHubParaId, xcm)
            dryRunAhSuccess = result.success
            assetHubDryRunError = result.errorMessage
            if (!dryRunAhSuccess) {
                logs.push({
                    kind: ValidationKind.Error,
                    reason: ValidationReason.DryRunFailed,
                    message: "Dry run on Asset Hub failed.",
                })
            }
        }

        if (!ahAssetMetadata.isSufficient && !dryRunAhSuccess) {
            const { accountMaxConsumers, accountExists } = await assetHubImpl.validateAccount(
                beneficiaryAddressHex,
                registry.ethChainId,
                tokenAddress,
                ahAssetMetadata,
            )

            if (accountMaxConsumers) {
                logs.push({
                    kind: ValidationKind.Error,
                    reason: ValidationReason.MaxConsumersReached,
                    message: "Beneficiary account has reached the max consumer limit on Asset Hub.",
                })
            }
            if (!accountExists) {
                logs.push({
                    kind: ValidationKind.Error,
                    reason: ValidationReason.AccountDoesNotExist,
                    message: "Beneficiary account does not exist on Asset Hub.",
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
            transfer,
        }
    }
}

import { ApiPromise } from "@polkadot/api"
import { AssetRegistry } from "@snowbridge/base-types"
import {
    RegistrationInterface,
    RegistrationFee,
    TokenRegistration,
    RegistrationValidationResult,
} from "./registrationInterface"
import { Context, EthersProviderTypes } from "../../index"
import {
    getMessageReceipt as getSharedMessageReceipt,
    ValidationKind,
} from "../../toPolkadotSnowbridgeV2"
import { FeeInfo, ValidationLog, ValidationReason } from "../../toPolkadot_v2"
import { getOperatingStatus } from "../../status"
import { DOT_LOCATION, erc20Location } from "../../xcmBuilder"
import { ETHER_TOKEN_ADDRESS } from "../../assets_v2"
import { padFeeByPercentage } from "../../utils"
import {
    buildAssetHubRegisterTokenXcm,
    getBridgeOwnerAccount,
} from "../../xcmbuilders/toPolkadot/registerToken"
import { TransactionReceipt } from "ethers"

const getAssetDeposit = (assetHub: ApiPromise): bigint => {
    return BigInt(assetHub.consts.foreignAssets.assetDeposit.toString())
}

export class RegisterToken implements RegistrationInterface {
    async getRegistrationFee(
        context: Context<EthersProviderTypes>,
        registry: AssetRegistry,
        relayerFee: bigint,
        options?: {
            paddFeeByPercentage?: bigint
        },
    ): Promise<RegistrationFee> {
        const assetHub = await context.assetHub()
        const bridgeHub = await context.bridgeHub()

        const paddFeeByPercentage = options?.paddFeeByPercentage ?? 33n
        const ether = erc20Location(registry.ethChainId, ETHER_TOKEN_ADDRESS)

        const assetDepositDOT = getAssetDeposit(assetHub)
        const assetHubXcm = buildAssetHubRegisterTokenXcm(
            assetHub,
            registry.ethChainId,
            "0x0000000000000000000000000000000000000000", // dummy token address
            1_000_000_000_000n, // dummy total value
            100_000_000_000n, // dummy execution fee
            assetDepositDOT,
            getBridgeOwnerAccount(registry.ethChainId),
        )

        // Delivery fee BridgeHub to AssetHub
        const bridgeHubImpl = await context.paraImplementation(bridgeHub)
        const deliveryFeeInDOT = await bridgeHubImpl.calculateDeliveryFeeInDOT(
            registry.assetHubParaId,
            assetHubXcm,
        )

        // AssetHub Execution fee
        const assetHubImpl = await context.paraImplementation(assetHub)

        const deliveryFeeInEther = await assetHubImpl.swapAsset1ForAsset2(
            DOT_LOCATION,
            ether,
            deliveryFeeInDOT,
        )

        const assetHubExecutionFeeDOT = await assetHubImpl.calculateXcmFee(
            assetHubXcm,
            DOT_LOCATION,
        )

        const assetHubExecutionFeeEther = padFeeByPercentage(
            await assetHubImpl.swapAsset1ForAsset2(DOT_LOCATION, ether, assetHubExecutionFeeDOT),
            paddFeeByPercentage,
        )

        // Convert asset deposit from DOT to Ether
        const assetDepositEther = padFeeByPercentage(
            await assetHubImpl.swapAsset1ForAsset2(DOT_LOCATION, ether, assetDepositDOT),
            10n,
        )

        const totalFeeInWei =
            deliveryFeeInEther + assetHubExecutionFeeEther + assetDepositEther + relayerFee

        return {
            assetHubDeliveryFeeEther: deliveryFeeInEther,
            assetHubExecutionFeeEther: assetHubExecutionFeeEther,
            assetDepositEther: assetDepositEther,
            assetDepositDOT: assetDepositDOT,
            relayerFee: relayerFee,
            totalFeeInWei: totalFeeInWei,
        }
    }

    async createRegistration(
        context: Context<EthersProviderTypes>,
        registry: AssetRegistry,
        sourceAccount: string,
        tokenAddress: string,
        fee: RegistrationFee,
    ): Promise<TokenRegistration> {
        const totalValue = fee.totalFeeInWei

        const network = 0

        const tx = await context.ethereumProvider.gatewayV2RegisterToken(
            context.ethereum(),
            context.environment.gatewayContract,
            sourceAccount,
            tokenAddress,
            network,
            fee.assetHubExecutionFeeEther,
            fee.relayerFee,
            totalValue,
        )

        return {
            input: {
                registry,
                sourceAccount,
                tokenAddress,
                fee,
            },
            computed: {
                gatewayAddress: registry.gatewayAddress,
                totalValue,
            },
            tx,
        }
    }

    async validateRegistration(
        context: Context<EthersProviderTypes>,
        registration: TokenRegistration,
    ): Promise<RegistrationValidationResult> {
        const { tx } = registration
        const { sourceAccount, tokenAddress, registry } = registration.input
        const ethereum = context.ethereum()
        const gateway = context.gatewayV2()
        const bridgeHub = await context.bridgeHub()
        const assetHub = await context.assetHub()

        const { totalValue } = registration.computed
        const logs: ValidationLog[] = []

        const isTokenAlreadyRegistered = await gateway.isTokenRegistered(tokenAddress)
        if (isTokenAlreadyRegistered) {
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.MinimumAmountValidation,
                message: "Token is already registered on the bridge.",
            })
        }

        const etherBalance = await context.ethereumProvider.getBalance(ethereum, sourceAccount)

        let feeInfo: FeeInfo | undefined
        if (logs.length === 0 || !isTokenAlreadyRegistered) {
            const [estimatedGas, feeData] = await Promise.all([
                context.ethereumProvider.estimateGas(ethereum, tx),
                context.ethereumProvider.getFeeData(ethereum),
            ])
            const executionFee = (feeData.gasPrice ?? 0n) * estimatedGas
            if (executionFee === 0n) {
                logs.push({
                    kind: ValidationKind.Error,
                    reason: ValidationReason.FeeEstimationError,
                    message: "Could not fetch fee details.",
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

        // Check bridge status
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

        // Dry run on AssetHub
        const ahParachain = registry.parachains[`polkadot_${registry.assetHubParaId}`]
        let assetHubDryRunError: string | undefined
        if (!ahParachain.features.hasDryRunApi) {
            logs.push({
                kind: ValidationKind.Warning,
                reason: ValidationReason.DryRunNotSupportedOnDestination,
                message:
                    "Asset Hub does not support dry running of XCM. Transaction success cannot be confirmed.",
            })
        } else {
            try {
                const xcm = buildAssetHubRegisterTokenXcm(
                    assetHub,
                    registry.ethChainId,
                    tokenAddress,
                    totalValue,
                    registration.input.fee.assetHubDeliveryFeeEther,
                    registration.input.fee.assetDepositDOT,
                    getBridgeOwnerAccount(registry.ethChainId),
                )

                const assetHubImpl = await context.paraImplementation(assetHub)
                const result = await assetHubImpl.dryRunXcm(registry.bridgeHubParaId, xcm)

                if (!result.success) {
                    assetHubDryRunError = result.errorMessage
                    logs.push({
                        kind: ValidationKind.Error,
                        reason: ValidationReason.DryRunFailed,
                        message: `Dry run on Asset Hub failed: ${result.errorMessage}`,
                    })
                }
            } catch (error: any) {
                assetHubDryRunError = error.message
                logs.push({
                    kind: ValidationKind.Warning,
                    reason: ValidationReason.DryRunNotSupportedOnDestination,
                    message: `Failed to perform dry run: ${error.message}`,
                })
            }
        }

        const success = logs.find((l) => l.kind === ValidationKind.Error) === undefined

        return {
            logs,
            success,
            data: {
                etherBalance,
                feeInfo,
                bridgeStatus,
                isTokenAlreadyRegistered,
                assetHubDryRunError,
            },
            registration,
        }
    }

    async getMessageReceipt(context: Context<EthersProviderTypes>, receipt: TransactionReceipt) {
        return getSharedMessageReceipt(context.ethereumProvider, receipt)
    }
}

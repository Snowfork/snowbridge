import { ApiPromise } from "@polkadot/api"
import { AssetRegistry } from "@snowbridge/base-types"
import {
    Connections,
    RegistrationInterface,
    RegistrationFee,
    TokenRegistration,
    RegistrationValidationResult,
} from "./registrationInterface"
import { IGATEWAY_V2_ABI } from "../../contracts"
import { Context } from "../../index"
import { ValidationKind } from "../../toPolkadotSnowbridgeV2"
import { FeeInfo, ValidationLog, ValidationReason } from "../../toPolkadot_v2"
import { AbstractProvider, Contract, Interface } from "ethers"
import { getOperatingStatus } from "../../status"
import { DOT_LOCATION, erc20Location } from "../../xcmBuilder"
import { ETHER_TOKEN_ADDRESS } from "../../assets_v2"
import { padFeeByPercentage } from "../../utils"
import { paraImplementation } from "../../parachains"
import {
    buildAssetHubRegisterTokenXcm,
    getBridgeOwnerAccount,
} from "../../xcmbuilders/toPolkadot/registerToken"

const getAssetDeposit = (assetHub: ApiPromise): bigint => {
    return BigInt(assetHub.consts.foreignAssets.assetDeposit.toString())
}

export class RegisterToken implements RegistrationInterface {
    async getRegistrationFee(
        context:
            | Context
            | {
                  assetHub: ApiPromise
                  bridgeHub: ApiPromise
              },
        registry: AssetRegistry,
        relayerFee: bigint,
        options?: {
            paddFeeByPercentage?: bigint
        },
    ): Promise<RegistrationFee> {
        const { assetHub, bridgeHub } =
            context instanceof Context
                ? {
                      assetHub: await context.assetHub(),
                      bridgeHub: await context.bridgeHub(),
                  }
                : context

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
        const bridgeHubImpl = await paraImplementation(bridgeHub)
        const deliveryFeeInDOT = await bridgeHubImpl.calculateDeliveryFeeInDOT(
            registry.assetHubParaId,
            assetHubXcm,
        )

        // AssetHub Execution fee
        const assetHubImpl = await paraImplementation(assetHub)

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
        context:
            | Context
            | {
                  ethereum: AbstractProvider
              },
        registry: AssetRegistry,
        sourceAccount: string,
        tokenAddress: string,
        fee: RegistrationFee,
    ): Promise<TokenRegistration> {
        const ifce = new Interface(IGATEWAY_V2_ABI)
        const con = new Contract(registry.gatewayAddress, ifce)

        const totalValue = fee.totalFeeInWei

        const network = 0

        const tx = await con
            .getFunction("v2_registerToken")
            .populateTransaction(
                tokenAddress,
                network,
                fee.assetHubExecutionFeeEther,
                fee.relayerFee,
                {
                    value: totalValue,
                    from: sourceAccount,
                },
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
        context: Context | Connections,
        registration: TokenRegistration,
    ): Promise<RegistrationValidationResult> {
        const { tx } = registration
        const { sourceAccount, tokenAddress, registry } = registration.input
        const { ethereum, gateway, bridgeHub, assetHub } =
            context instanceof Context
                ? {
                      ethereum: context.ethereum(),
                      gateway: context.gatewayV2(),
                      bridgeHub: await context.bridgeHub(),
                      assetHub: await context.assetHub(),
                  }
                : { ...context, assetHub: context.assetHub }

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

        const etherBalance = await ethereum.getBalance(sourceAccount)

        let feeInfo: FeeInfo | undefined
        if (logs.length === 0 || !isTokenAlreadyRegistered) {
            const [estimatedGas, feeData] = await Promise.all([
                ethereum.estimateGas(tx),
                ethereum.getFeeData(),
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

                const assetHubImpl = await paraImplementation(assetHub)
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
}

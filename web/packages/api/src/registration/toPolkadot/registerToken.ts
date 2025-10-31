import { ApiPromise } from "@polkadot/api"
import { AssetRegistry } from "@snowbridge/base-types"
import {
    Connections,
    RegistrationInterface,
    RegistrationFee,
    TokenRegistration,
    RegistrationValidationResult,
} from "./registrationInterface"
import { IGatewayV2__factory as IGateway__factory } from "@snowbridge/contract-types"
import { Context } from "../../index"
import { ValidationKind } from "../../toPolkadotSnowbridgeV2"
import { FeeInfo, ValidationLog, ValidationReason } from "../../toPolkadot_v2"
import { AbstractProvider, Contract } from "ethers"
import { getOperatingStatus } from "../../status"
import { DOT_LOCATION, erc20Location } from "../../xcmBuilder"
import { swapAsset1ForAsset2, ETHER_TOKEN_ADDRESS } from "../../assets_v2"
import { padFeeByPercentage } from "../../utils"
import { paraImplementation } from "../../parachains"
import { claimerFromBeneficiary, claimerLocationToBytes } from "../../toPolkadotSnowbridgeV2"
import {
    buildRegisterTokenXcm,
    buildAssetHubRegisterTokenXcm,
    getBridgeOwnerAccount,
} from "../../xcmbuilders/toPolkadot/registerToken"
/**
 * Fetch the asset deposit value from AssetHub runtime constants
 * This is the DOT deposit required to create a new foreign asset
 */
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
        }
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

        // Fetch the asset deposit required for creating a new asset on AssetHub
        const assetDepositDOT = getAssetDeposit(assetHub)

        // Build a sample XCM message to estimate fees
        // Use dummy values for claimer and token address since we only need the structure
        const claimer = claimerFromBeneficiary(
            assetHub,
            "0x0000000000000000000000000000000000000000000000000000000000000000"
        )
        const bridgeOwner = getBridgeOwnerAccount(assetHub, registry.ethChainId)

        const remoteXcm = buildRegisterTokenXcm(
            assetHub,
            registry.ethChainId,
            "0x0000000000000000000000000000000000000000",
            1_000_000_000_000n, // 1000 ETH dummy amount
            claimer,
            bridgeOwner,
            assetDepositDOT
        )

        const assetHubXcm = buildAssetHubRegisterTokenXcm(
            assetHub,
            registry.ethChainId,
            1_000_000_000_000n,
            100_000_000_000n,
            claimer,
            "0x0000000000000000000000000000000000000000",
            remoteXcm.toU8a()
        )

        // Delivery fee BridgeHub to AssetHub
        const bridgeHubImpl = await paraImplementation(bridgeHub)
        const deliveryFeeInDOT = await bridgeHubImpl.calculateDeliveryFeeInDOT(
            registry.assetHubParaId,
            assetHubXcm
        )

        const deliveryFeeInEther = await swapAsset1ForAsset2(
            assetHub,
            DOT_LOCATION,
            ether,
            deliveryFeeInDOT
        )

        // AssetHub Execution fee
        const assetHubImpl = await paraImplementation(assetHub)
        const assetHubExecutionFeeDOT = await assetHubImpl.calculateXcmFee(assetHubXcm, DOT_LOCATION)

        const assetHubExecutionFeeEther = padFeeByPercentage(
            await swapAsset1ForAsset2(assetHub, DOT_LOCATION, ether, assetHubExecutionFeeDOT),
            paddFeeByPercentage
        )

        // Convert asset deposit from DOT to Ether
        const assetDepositEther = await swapAsset1ForAsset2(
            assetHub,
            DOT_LOCATION,
            ether,
            assetDepositDOT
        )

        const totalFeeInWei =
            deliveryFeeInEther + assetHubExecutionFeeEther + assetDepositEther + relayerFee

        return {
            assetHubDeliveryFeeEther: deliveryFeeInEther,
            assetHubExecutionFeeEther: assetHubExecutionFeeEther,
            assetDepositEther: assetDepositEther,
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
        fee: RegistrationFee
    ): Promise<TokenRegistration> {
        const ifce = IGateway__factory.createInterface()
        const con = new Contract(registry.gatewayAddress, ifce)

        const totalValue = fee.totalFeeInWei

        // network = 0 for Polkadot (as per Gateway.sol line 470)
        const network = 0

        // The execution fee passed to the gateway includes both delivery and execution
        const executionFee = fee.assetHubDeliveryFeeEther + fee.assetHubExecutionFeeEther

        const tx = await con
            .getFunction("v2_registerToken")
            .populateTransaction(tokenAddress, network, executionFee, fee.relayerFee, {
                value: totalValue,
                from: sourceAccount,
            })

        return {
            input: {
                registry,
                sourceAccount,
                tokenAddress,
                assetHubDeliveryFeeEther: fee.assetHubDeliveryFeeEther,
                assetHubExecutionFeeEther: fee.assetHubExecutionFeeEther,
                relayerFee: fee.relayerFee,
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
        registration: TokenRegistration
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

        // Check if token is already registered
        const isTokenAlreadyRegistered = await gateway.isTokenRegistered(tokenAddress)
        if (isTokenAlreadyRegistered) {
            logs.push({
                kind: ValidationKind.Warning,
                reason: ValidationReason.MinimumAmountValidation,
                message: "Token is already registered on the bridge.",
            })
        }

        // Check ether balance
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
        const ahParachain = registry.parachains[registry.assetHubParaId]
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
                // Build the remote XCM that will be executed on AssetHub
                const claimer = claimerFromBeneficiary(assetHub, sourceAccount)
                const bridgeOwner = getBridgeOwnerAccount(assetHub, registry.ethChainId)

                // Fetch the asset deposit required for creating a new asset on AssetHub
                const assetDepositDOT = getAssetDeposit(assetHub)

                // First build the remote XCM (the registration instructions)
                const executionFee =
                    registration.input.assetHubDeliveryFeeEther +
                    registration.input.assetHubExecutionFeeEther
                console.log("total value", totalValue)
                console.log("executionFee", executionFee)
                const remoteXcm = buildRegisterTokenXcm(
                    assetHub,
                    registry.ethChainId,
                    tokenAddress,
                    totalValue - executionFee,
                    claimer,
                    bridgeOwner,
                    assetDepositDOT
                )

                // Then build the full AssetHub XCM (with BridgeHub wrapper)
                const xcm = buildAssetHubRegisterTokenXcm(
                    assetHub,
                    registry.ethChainId,
                    totalValue,
                    executionFee,
                    claimer,
                    sourceAccount,
                    remoteXcm.toU8a()
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

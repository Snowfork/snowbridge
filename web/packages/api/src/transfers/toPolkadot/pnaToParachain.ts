import { ApiPromise } from "@polkadot/api"
import { AssetRegistry } from "@snowbridge/base-types"
import { Connections, TransferInterface } from "./transferInterface"
import {
    IGatewayV2__factory as IGateway__factory,
    IGatewayV2 as IGateway,
} from "@snowbridge/contract-types"
import { Context } from "../../index"
import {
    buildMessageId,
    claimerFromBeneficiary,
    DeliveryFee,
    dryRunAssetHub,
    dryRunDestination,
    encodeForeignAsset,
    erc20Balance,
    hexToBytes,
    Transfer,
    validateAccount,
    ValidationKind,
    ValidationResult,
} from "../../toPolkadotSnowbridgeV2"
import { accountId32Location, DOT_LOCATION, erc20Location } from "../../xcmBuilder"
import { paraImplementation } from "../../parachains"
import { ETHER_TOKEN_ADDRESS, swapAsset1ForAsset2 } from "../../assets_v2"
import { beneficiaryMultiAddress, padFeeByPercentage, paraIdToSovereignAccount } from "../../utils"
import { FeeInfo, resolveInputs, ValidationLog, ValidationReason } from "../../toPolkadot_v2"
import {
    buildAssetHubPNAReceivedXcm,
    buildParachainPNAReceivedXcmOnDestination,
    sendMessageXCM,
} from "../../xcmbuilders/toPolkadot/pnaToParachain"
import { Contract } from "ethers"
import { getOperatingStatus } from "../../status"

export class PNAToParachain implements TransferInterface {
    async getDeliveryFee(
        context:
            | Context
            | {
                  gateway: IGateway
                  assetHub: ApiPromise
                  bridgeHub: ApiPromise
                  destination: ApiPromise
              },
        registry: AssetRegistry,
        tokenAddress: string,
        destinationParaId: number,
        relayerFee: bigint,
        paddFeeByPercentage?: bigint
    ): Promise<DeliveryFee> {
        const { assetHub, bridgeHub, destination } =
            context instanceof Context
                ? {
                      assetHub: await context.assetHub(),
                      bridgeHub: await context.bridgeHub(),
                      destination: await context.parachain(destinationParaId),
                  }
                : context

        const { destParachain, destAssetMetadata } = resolveInputs(
            registry,
            tokenAddress,
            destinationParaId
        )
        // AssetHub fees
        let assetHubXcm = buildAssetHubPNAReceivedXcm(
            assetHub.registry,
            registry.ethChainId,
            destAssetMetadata.location,
            1000000000000n,
            1000000000000n,
            1000000000000n,
            1000000000000n,
            accountId32Location(
                "0x0000000000000000000000000000000000000000000000000000000000000000"
            ),
            "0x0000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            destinationParaId,
            "0x0000000000000000000000000000000000000000000000000000000000000000"
        )
        const bridgeHubImpl = await paraImplementation(bridgeHub)
        const assetHubImpl = await paraImplementation(assetHub)
        let ether = erc20Location(registry.ethChainId, ETHER_TOKEN_ADDRESS)

        // Delivery fee BridgeHub to AssetHub
        const deliveryFeeInDOT = await bridgeHubImpl.calculateDeliveryFeeInDOT(
            registry.assetHubParaId,
            assetHubXcm
        )
        // AssetHub execution fee
        let assetHubExecutionFeeDOT = await assetHubImpl.calculateXcmFee(assetHubXcm, DOT_LOCATION)
        // Swap to ether
        const deliveryFeeInEther = await swapAsset1ForAsset2(
            assetHub,
            DOT_LOCATION,
            ether,
            deliveryFeeInDOT
        )
        let assetHubExecutionFeeEther = padFeeByPercentage(
            await swapAsset1ForAsset2(assetHub, DOT_LOCATION, ether, assetHubExecutionFeeDOT),
            paddFeeByPercentage ?? 33n
        )

        // Destination fees
        let destinationXcm = buildParachainPNAReceivedXcmOnDestination(
            destination.registry,
            registry.ethChainId,
            destAssetMetadata.location,
            340282366920938463463374607431768211455n,
            340282366920938463463374607431768211455n,
            destParachain.info.accountType === "AccountId32"
                ? "0x0000000000000000000000000000000000000000000000000000000000000000"
                : "0x0000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000000000000000000000000000"
        )
        const destinationImpl = await paraImplementation(destination)
        // Delivery fee AssetHub to Destination
        let destinationDeliveryFeeDOT = await assetHubImpl.calculateDeliveryFeeInDOT(
            destinationParaId,
            destinationXcm
        )
        // Destination execution fee
        let destinationExecutionFeeDOT = await destinationImpl.calculateXcmFee(
            destinationXcm,
            DOT_LOCATION
        )

        // Swap to ether
        const destinationDeliveryFeeEther = await swapAsset1ForAsset2(
            assetHub,
            DOT_LOCATION,
            ether,
            destinationDeliveryFeeDOT
        )
        let destinationExecutionFeeEther = padFeeByPercentage(
            await swapAsset1ForAsset2(assetHub, DOT_LOCATION, ether, destinationExecutionFeeDOT),
            paddFeeByPercentage ?? 33n
        )

        const totalFeeInWei = deliveryFeeInEther + assetHubExecutionFeeEther + relayerFee
        return {
            assetHubDeliveryFeeEther: deliveryFeeInEther,
            assetHubExecutionFeeEther: assetHubExecutionFeeEther,
            destinationDeliveryFeeEther: destinationDeliveryFeeEther,
            destinationExecutionFeeEther: destinationExecutionFeeEther,
            relayerFee: relayerFee,
            totalFeeInWei: totalFeeInWei,
        }
    }

    async createTransfer(
        context:
            | Context
            | {
                  assetHub: ApiPromise
                  destination: ApiPromise
              },
        registry: AssetRegistry,
        destinationParaId: number,
        sourceAccount: string,
        beneficiaryAccount: string,
        tokenAddress: string,
        amount: bigint,
        fee: DeliveryFee
    ): Promise<Transfer> {
        const { assetHub, destination } =
            context instanceof Context
                ? {
                      assetHub: await context.assetHub(),
                      destination: await context.parachain(destinationParaId),
                  }
                : context
        if (!destination) {
            throw Error(`Unable to connect to destination parachain with ID ${destinationParaId}.`)
        }
        const { tokenErcMetadata, destParachain, ahAssetMetadata, destAssetMetadata } =
            resolveInputs(registry, tokenAddress, destinationParaId)
        const minimalBalance =
            ahAssetMetadata.minimumBalance > destAssetMetadata.minimumBalance
                ? ahAssetMetadata.minimumBalance
                : destAssetMetadata.minimumBalance

        let { address: beneficiary, hexAddress: beneficiaryAddressHex } =
            beneficiaryMultiAddress(beneficiaryAccount)
        let value = fee.totalFeeInWei

        const ifce = IGateway__factory.createInterface()
        const con = new Contract(registry.gatewayAddress, ifce)

        if (!ahAssetMetadata.foreignId) {
            throw Error("asset foreign ID not set in metadata")
        }

        const topic = buildMessageId(
            destinationParaId,
            sourceAccount,
            tokenAddress,
            beneficiaryAddressHex,
            amount
        )

        const xcm = hexToBytes(
            sendMessageXCM(
                destination.registry,
                registry.ethChainId,
                destinationParaId,
                destAssetMetadata.location,
                beneficiaryAddressHex,
                amount,
                fee.destinationExecutionFeeEther,
                topic
            ).toHex()
        )
        let assets = [encodeForeignAsset(ahAssetMetadata.foreignId, amount)]
        let claimer = claimerFromBeneficiary(assetHub, beneficiaryAddressHex)

        const tx = await con
            .getFunction("v2_sendMessage")
            .populateTransaction(
                xcm,
                assets,
                claimer,
                fee.assetHubExecutionFeeEther,
                fee.relayerFee,
                {
                    value,
                    from: sourceAccount,
                }
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
                topic,
            },
            tx,
        }
    }

    async validateTransfer(
        context: Context | Connections,
        transfer: Transfer
    ): Promise<ValidationResult> {
        const { tx } = transfer
        const { amount, sourceAccount, tokenAddress, registry, destinationParaId } = transfer.input
        const {
            ethereum,
            gateway,
            bridgeHub,
            assetHub,
            destination: destParachainApi,
        } = context instanceof Context
            ? {
                  ethereum: context.ethereum(),
                  gateway: context.gateway(),
                  bridgeHub: await context.bridgeHub(),
                  assetHub: await context.assetHub(),
                  destination: await context.parachain(destinationParaId),
              }
            : context

        const {
            totalValue,
            minimalBalance,
            destParachain,
            destAssetMetadata,
            ahAssetMetadata,
            beneficiaryAddressHex,
        } = transfer.computed

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
            tokenBalance = await erc20Balance(
                ethereum,
                tokenAddress,
                sourceAccount,
                registry.gatewayAddress
            )
        } else {
            tokenBalance = {
                balance: etherBalance,
                // u128 max
                gatewayAllowance: 340282366920938463463374607431768211455n,
            }
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
                ethereum.estimateGas(tx),
                ethereum.getFeeData(),
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
        const ahParachain = registry.parachains[registry.assetHubParaId]
        let dryRunAhSuccess, forwardedDestination, assetHubDryRunError
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
                destAssetMetadata.location,
                transfer.computed.totalValue - assetHubFee,
                assetHubFee,
                transfer.input.fee.destinationExecutionFeeEther,
                amount,
                accountId32Location(
                    "0x0000000000000000000000000000000000000000000000000000000000000000"
                ),
                transfer.input.sourceAccount,
                transfer.computed.beneficiaryAddressHex,
                destinationParaId,
                "0x0000000000000000000000000000000000000000000000000000000000000000"
            )
            let result = await dryRunAssetHub(
                assetHub,
                registry.bridgeHubParaId,
                destinationParaId,
                xcm
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

        const assetHubImpl = await paraImplementation(assetHub)
        let destinationParachainDryRunError: string | undefined
        // Check if sovereign account balance for token is at 0 and that consumers is maxxed out.
        if (!ahAssetMetadata.isSufficient && !dryRunAhSuccess) {
            const sovereignAccountId = paraIdToSovereignAccount("sibl", destinationParaId)
            const { accountMaxConumers, accountExists } = await validateAccount(
                assetHubImpl,
                sovereignAccountId,
                registry.ethChainId,
                tokenAddress,
                ahAssetMetadata
            )

            if (!accountExists) {
                logs.push({
                    kind: ValidationKind.Error,
                    reason: ValidationReason.MaxConsumersReached,
                    message: "Sovereign account does not exist on Asset Hub.",
                })
            }
            if (accountMaxConumers) {
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
                    const { success: dryRunDestinationSuccess, errorMessage: destMessage } =
                        await dryRunDestination(destParachainApi, transfer, xcm[0])
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
                const destParachainImpl = await paraImplementation(destParachainApi)
                // Check if the account is created
                const { accountMaxConumers, accountExists } = await validateAccount(
                    destParachainImpl,
                    beneficiaryAddressHex,
                    registry.ethChainId,
                    tokenAddress,
                    destAssetMetadata
                )
                if (accountMaxConumers) {
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
            transfer,
        }
    }
}

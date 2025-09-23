import { ApiPromise } from "@polkadot/api"
import { AssetRegistry } from "@snowbridge/base-types"
import { Connections, TransferInterface } from "./transferInterface"
import {
    IGatewayV2__factory as IGateway__factory,
    IGatewayV2 as IGateway,
} from "@snowbridge/contract-types"
import { Context } from "../../index"
import {
    claimerFromBeneficiary,
    claimerLocationToBytes,
    DeliveryFee,
    dryRunAssetHub,
    encodeNativeAsset,
    ValidationKind,
} from "../../toPolkadotSnowbridgeV2"
import {
    sendMessageXCM,
    buildAssetHubERC20ReceivedXcm,
} from "../../xcmbuilders/toPolkadot/erc20ToAH"
import { accountId32Location, DOT_LOCATION, erc20Location } from "../../xcmBuilder"
import { paraImplementation } from "../../parachains"
import {
    erc20Balance,
    ETHER_TOKEN_ADDRESS,
    swapAsset1ForAsset2,
    validateAccount,
} from "../../assets_v2"
import { beneficiaryMultiAddress, padFeeByPercentage } from "../../utils"
import { Contract } from "ethers"
import { FeeInfo, resolveInputs, ValidationLog, ValidationReason } from "../../toPolkadot_v2"
import { buildMessageId, Transfer, ValidationResult } from "../../toPolkadotSnowbridgeV2"
import { getOperatingStatus } from "../../status"
import { hexToU8a } from "@polkadot/util"

export class ERC20ToAH implements TransferInterface {
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
        _destinationParaId: number,
        relayerFee: bigint,
        options?: {
            paddFeeByPercentage?: bigint
            feeAsset?: any
        }
    ): Promise<DeliveryFee> {
        const { assetHub, bridgeHub } =
            context instanceof Context
                ? {
                      assetHub: await context.assetHub(),
                      bridgeHub: await context.bridgeHub(),
                  }
                : context

        let assetHubXcm = buildAssetHubERC20ReceivedXcm(
            assetHub.registry,
            registry.ethChainId,
            tokenAddress,
            2000000000000n,
            1000000000000n,
            1000000000000n,
            accountId32Location(
                "0x0000000000000000000000000000000000000000000000000000000000000000"
            ),
            "0x0000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000000000000000000000000000"
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
            assetHubXcm
        )

        const assetHubImpl = await paraImplementation(assetHub)
        const deliveryFeeInEther = await swapAsset1ForAsset2(
            assetHub,
            DOT_LOCATION,
            ether,
            deliveryFeeInDOT
        )
        // AssetHub Execution fee
        let assetHubExecutionFeeDOT = await assetHubImpl.calculateXcmFee(assetHubXcm, DOT_LOCATION)

        let assetHubExecutionFeeEther = padFeeByPercentage(
            await swapAsset1ForAsset2(assetHub, DOT_LOCATION, ether, assetHubExecutionFeeDOT),
            paddFeeByPercentage ?? 33n
        )

        registry.environment
        const totalFeeInWei = deliveryFeeInEther + assetHubExecutionFeeEther + relayerFee
        return {
            assetHubDeliveryFeeEther: deliveryFeeInEther,
            assetHubExecutionFeeEther: assetHubExecutionFeeEther,
            destinationDeliveryFeeEther: 0n,
            destinationExecutionFeeEther: 0n,
            relayerFee: relayerFee,
            totalFeeInWei: totalFeeInWei,
            feeAsset: feeAsset,
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
        const { assetHub } =
            context instanceof Context
                ? {
                      assetHub: await context.assetHub(),
                  }
                : context

        const { tokenErcMetadata, destParachain, ahAssetMetadata, destAssetMetadata } =
            resolveInputs(registry, tokenAddress, destinationParaId)
        const minimalBalance =
            ahAssetMetadata.minimumBalance > destAssetMetadata.minimumBalance
                ? ahAssetMetadata.minimumBalance
                : destAssetMetadata.minimumBalance

        let { address: beneficiary, hexAddress: beneficiaryAddressHex } =
            beneficiaryMultiAddress(beneficiaryAccount)
        let value = fee.totalFeeInWei
        let assets: any = []
        if (tokenAddress === ETHER_TOKEN_ADDRESS) {
            value += amount
        } else {
            assets = [encodeNativeAsset(tokenAddress, amount)]
        }
        const ifce = IGateway__factory.createInterface()
        const con = new Contract(registry.gatewayAddress, ifce)

        const topic = buildMessageId(
            destinationParaId,
            sourceAccount,
            tokenAddress,
            beneficiaryAddressHex,
            amount
        )

        const xcm = hexToU8a(
            sendMessageXCM(assetHub.registry, beneficiaryAddressHex, topic).toHex()
        )
        let claimer = claimerFromBeneficiary(assetHub, beneficiaryAddressHex)

        const tx = await con
            .getFunction("v2_sendMessage")
            .populateTransaction(
                xcm,
                assets,
                claimerLocationToBytes(claimer),
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
                claimer,
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
        const { amount, sourceAccount, tokenAddress, registry } = transfer.input
        const { ethereum, gateway, bridgeHub, assetHub } =
            context instanceof Context
                ? {
                      ethereum: context.ethereum(),
                      gateway: context.gateway(),
                      bridgeHub: await context.bridgeHub(),
                      assetHub: await context.assetHub(),
                  }
                : context

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
        let dryRunAhSuccess, assetHubDryRunError
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
                transfer.input.fee.assetHubDeliveryFeeEther +
                transfer.input.fee.assetHubExecutionFeeEther
            const xcm = buildAssetHubERC20ReceivedXcm(
                assetHub.registry,
                registry.ethChainId,
                tokenAddress,
                transfer.computed.totalValue - assetHubFee,
                assetHubFee,
                amount,
                claimer,
                transfer.input.sourceAccount,
                transfer.computed.beneficiaryAddressHex,
                "0x0000000000000000000000000000000000000000000000000000000000000000"
            )

            let result = await dryRunAssetHub(assetHub, registry.bridgeHubParaId, 0, xcm)
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

        const assetHubImpl = await paraImplementation(assetHub)

        if (!ahAssetMetadata.isSufficient && !dryRunAhSuccess) {
            const { accountMaxConsumers, accountExists } = await validateAccount(
                assetHubImpl,
                beneficiaryAddressHex,
                registry.ethChainId,
                tokenAddress,
                ahAssetMetadata
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

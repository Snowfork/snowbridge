import { AssetRegistry } from "@snowbridge/base-types"
import { TransferInterface } from "./transferInterface"
import { Context } from "../../index"
import {
    buildSwapCallData,
    calculateRelayerFee,
    claimerFromBeneficiary,
    claimerLocationToBytes,
    DeliveryFee,
    encodeNativeAsset,
    ValidationKind,
} from "../../toPolkadotSnowbridgeV2"
import {
    sendMessageXCM,
    buildAssetHubERC20ReceivedXcm,
} from "../../xcmbuilders/toPolkadot/erc20ToAH"
import { accountId32Location, DOT_LOCATION, erc20Location } from "../../xcmBuilder"
import { paraImplementation } from "../../parachains"
import { erc20Balance, ETHER_TOKEN_ADDRESS } from "../../assets_v2"
import { beneficiaryMultiAddress, padFeeByPercentage } from "../../utils"
import { FeeInfo, resolveInputs, ValidationLog, ValidationReason } from "../../toPolkadot_v2"
import { buildMessageId, Transfer, ValidationResult } from "../../toPolkadotSnowbridgeV2"
import { getOperatingStatus } from "../../status"
import { hexToU8a } from "@polkadot/util"
import {
    DepositParamsStruct,
    SendParamsStruct,
    SwapParamsStruct,
} from "@snowbridge/contract-types/dist/SnowbridgeL2Adaptor"
import { estimateFees } from "../../across/api"
import { ContractTransaction } from "ethers/lib.commonjs/contract/types"
import { ISwapQuoter } from "@snowbridge/contract-types"

export class ERC20ToAH implements TransferInterface {
    async getDeliveryFee(
        context: Context,
        registry: AssetRegistry,
        l2ChainId: number,
        l2TokenAddress: string,
        amount: bigint,
        _destinationParaId: number,
        options?: {
            paddFeeByPercentage?: bigint
            feeAsset?: any
            customXcm?: any[]
            overrideRelayerFee?: bigint
            l2PadFeeByPercentage?: bigint
            fillDeadlineBuffer?: bigint
        },
    ): Promise<DeliveryFee> {
        const { assetHub, bridgeHub } = {
            assetHub: await context.assetHub(),
            bridgeHub: await context.bridgeHub(),
        }
        if (registry.ethereumChains?.[`ethereum_l2_${l2ChainId}`] == undefined) {
            throw new Error(`L2 Chain ID ${l2ChainId} is not supported in the provided registry`)
        }
        if (
            registry.ethereumChains?.[`ethereum_l2_${l2ChainId}`]?.assets[l2TokenAddress] ==
            undefined
        ) {
            throw new Error(
                `L2 Token Address ${l2TokenAddress} is not supported in the provided registry for L2 Chain ID ${l2ChainId}`,
            )
        }
        let tokenAddress =
            registry.ethereumChains?.[`ethereum_l2_${l2ChainId}`]?.assets[l2TokenAddress]
                ?.swapTokenAddress
        if (!tokenAddress) {
            throw new Error("Token is not registered on Ethereum")
        }
        let assetHubXcm = buildAssetHubERC20ReceivedXcm(
            assetHub.registry,
            registry.ethChainId,
            tokenAddress,
            2000000000000n,
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

        // Calculate fee with Across SDK
        let bridgeFeeInL2Token = 0n,
            swapFeeInL1Token = 0n
        let totalFeeInWei = assetHubExecutionFeeEther + relayerFee
        let l2FeeTokenAddress = context.l2FeeTokenAddress(l2ChainId)
        if (l2TokenAddress == ETHER_TOKEN_ADDRESS || l2TokenAddress == l2FeeTokenAddress) {
            const l1FeeTokenAddress =
                registry.ethereumChains?.[`ethereum_l2_${l2ChainId}`]?.assets[l2FeeTokenAddress]
                    ?.swapTokenAddress
            if (!l1FeeTokenAddress) {
                throw new Error("Fee token is not registered on Ethereum")
            }
            try {
                bridgeFeeInL2Token = await estimateFees(
                    context.acrossApiUrl(),
                    l2FeeTokenAddress,
                    l1FeeTokenAddress,
                    l2ChainId,
                    registry.ethChainId,
                    assetHubExecutionFeeEther + relayerFee + amount,
                )
            } catch (e) {
                throw new Error("Failed to estimate Across bridge fees: " + (e as Error).message)
            }
            bridgeFeeInL2Token = padFeeByPercentage(
                bridgeFeeInL2Token,
                options?.l2PadFeeByPercentage ?? 33n,
            )
            totalFeeInWei += bridgeFeeInL2Token
        } else {
            let swapFee =
                registry.ethereumChains?.[`ethereum_l2_${l2ChainId}`]?.assets[l2TokenAddress]
                    ?.swapFee
            let swapQuoter = context.l1SwapQuoter()
            let params: ISwapQuoter.QuoteExactOutputSingleParamsStruct = {
                tokenIn: tokenAddress,
                tokenOut: context.l1FeeTokenAddress(),
                amount: assetHubExecutionFeeEther + relayerFee,
                fee: swapFee ?? 500, // 0.05% pool fee
                sqrtPriceLimitX96: 0, // no price limit
            }
            let result = await swapQuoter.quoteExactOutputSingle.staticCall(params)
            swapFeeInL1Token = result[0] as bigint
            swapFeeInL1Token = padFeeByPercentage(
                swapFeeInL1Token,
                options?.l2PadFeeByPercentage ?? 33n,
            )
            try {
                bridgeFeeInL2Token = await estimateFees(
                    context.acrossApiUrl(),
                    l2TokenAddress,
                    tokenAddress,
                    l2ChainId,
                    registry.ethChainId,
                    amount + swapFeeInL1Token,
                )
            } catch (e) {
                throw new Error("Failed to estimate Across bridge fees: " + (e as Error).message)
            }
            bridgeFeeInL2Token = padFeeByPercentage(
                bridgeFeeInL2Token,
                options?.l2PadFeeByPercentage ?? 33n,
            )
        }

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
            swapFeeInL1Token,
            bridgeFeeInL2Token,
        }
    }

    async createTransfer(
        context: Context,
        registry: AssetRegistry,
        l2ChainId: number,
        l2TokenAddress: string,
        amount: bigint,
        destinationParaId: number,
        sourceAccount: string,
        beneficiaryAccount: string,
        fee: DeliveryFee,
        options?: {
            customXcm?: any[]
            fillDeadlineBuffer?: bigint
        },
    ): Promise<Transfer> {
        const assetHub = await context.assetHub()
        const l2Chain = context.ethChain(l2ChainId)

        let tokenAddress =
            registry.ethereumChains?.[`ethereum_l2_${l2ChainId}`]?.assets[l2TokenAddress]
                ?.swapTokenAddress
        if (!tokenAddress) {
            throw new Error("Token is not registered on Ethereum")
        }

        const { tokenErcMetadata, destParachain, ahAssetMetadata, destAssetMetadata } =
            resolveInputs(registry, tokenAddress, destinationParaId)
        const minimalBalance =
            ahAssetMetadata.minimumBalance > destAssetMetadata.minimumBalance
                ? ahAssetMetadata.minimumBalance
                : destAssetMetadata.minimumBalance

        let { address: beneficiary, hexAddress: beneficiaryAddressHex } =
            beneficiaryMultiAddress(beneficiaryAccount)

        let assets: any = []
        let value: bigint, inputAmount: bigint

        const l2Adapter = context.l2Adapter(l2ChainId)
        const accountNonce = await l2Chain.getTransactionCount(sourceAccount, "pending")

        const topic = buildMessageId(
            destinationParaId,
            sourceAccount,
            l2TokenAddress,
            beneficiaryAddressHex,
            amount,
            accountNonce,
        )

        const xcm = hexToU8a(
            sendMessageXCM(
                assetHub.registry,
                beneficiaryAddressHex,
                topic,
                options?.customXcm,
            ).toHex(),
        )
        let claimer = claimerFromBeneficiary(assetHub, beneficiaryAddressHex)

        let depositParams: DepositParamsStruct, tx: ContractTransaction

        let sendParams: SendParamsStruct = {
            xcm: xcm,
            assets: assets,
            claimer: claimerLocationToBytes(claimer),
            executionFee: fee.assetHubExecutionFeeEther,
            relayerFee: fee.relayerFee,
        }
        let l2FeeTokenAddress = context.l2FeeTokenAddress(l2ChainId)
        if (l2TokenAddress === ETHER_TOKEN_ADDRESS || l2TokenAddress === l2FeeTokenAddress) {
            value = fee.totalFeeInWei + amount
            depositParams = {
                inputToken: l2TokenAddress,
                outputToken: tokenAddress,
                inputAmount: value,
                outputAmount: amount,
                destinationChainId: BigInt(registry.ethChainId),
                fillDeadlineBuffer: options?.fillDeadlineBuffer ?? 600n,
            }
            let txOptions: any = { from: sourceAccount }
            if (l2TokenAddress === ETHER_TOKEN_ADDRESS) {
                txOptions = { ...txOptions, value: value }
            }
            tx = await l2Adapter
                .getFunction("sendEtherAndCall")
                .populateTransaction(depositParams, sendParams, sourceAccount, topic, txOptions)
        } else {
            value = fee.totalFeeInWei
            inputAmount = amount + fee.bridgeFeeInL2Token! + fee.swapFeeInL1Token!
            assets = [encodeNativeAsset(tokenAddress, amount)]
            sendParams.assets = assets
            depositParams = {
                inputToken: l2TokenAddress,
                outputToken: tokenAddress,
                inputAmount,
                outputAmount: amount,
                destinationChainId: BigInt(registry.ethChainId),
                fillDeadlineBuffer: options?.fillDeadlineBuffer ?? 600n,
            }
            let swapCalldata = await buildSwapCallData(
                context,
                registry,
                l2ChainId,
                l2TokenAddress,
                fee.assetHubExecutionFeeEther + fee.relayerFee,
                fee.swapFeeInL1Token!,
            )
            let swapParams: SwapParamsStruct = {
                inputAmount: fee.swapFeeInL1Token!,
                router: context.l1SwapRouterAddress(),
                callData: swapCalldata,
            }
            tx = await l2Adapter
                .getFunction("sendTokenAndCall")
                .populateTransaction(depositParams, swapParams, sendParams, sourceAccount, topic, {
                    from: sourceAccount,
                })
        }

        return {
            input: {
                registry,
                sourceAccount,
                beneficiaryAccount,
                tokenAddress,
                destinationParaId,
                amount,
                fee,
                customXcm: options?.customXcm,
                l2TokenAddress,
                sourceChainId: l2ChainId,
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
                l2AdapterAddress: l2Adapter.target.toString(),
            },
            tx,
        }
    }

    async validateTransfer(context: Context, transfer: Transfer): Promise<ValidationResult> {
        const { tx } = transfer
        const { amount, sourceAccount, tokenAddress, registry, l2TokenAddress, sourceChainId } =
            transfer.input
        const { gateway, bridgeHub, assetHub, l2Chain } = {
            gateway: context.gateway(),
            bridgeHub: await context.bridgeHub(),
            assetHub: await context.assetHub(),
            l2Chain: context.ethChain(sourceChainId!),
        }

        const {
            totalValue,
            minimalBalance,
            ahAssetMetadata,
            beneficiaryAddressHex,
            claimer,
            l2AdapterAddress,
        } = transfer.computed

        const logs: ValidationLog[] = []
        if (amount < minimalBalance) {
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.MinimumAmountValidation,
                message: "The amount transferred is less than the minimum amount.",
            })
        }
        const etherBalance = await l2Chain.getBalance(sourceAccount)

        let tokenBalance: { balance: bigint; gatewayAllowance: bigint }
        if (tokenAddress !== ETHER_TOKEN_ADDRESS) {
            tokenBalance = await erc20Balance(
                l2Chain,
                l2TokenAddress!,
                sourceAccount,
                l2AdapterAddress!,
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
                message: "The amount transferred is greater than the users token balance.",
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
        let l2BridgeDryRunError: string | undefined
        if (logs.length === 0) {
            let estimatedGas: bigint
            try {
                estimatedGas = await l2Chain.estimateGas(tx)
            } catch (e) {
                l2BridgeDryRunError =
                    "Could not estimate gas for l2 transaction." + (e as Error).message
                logs.push({
                    kind: ValidationKind.Error,
                    reason: ValidationReason.FeeEstimationError,
                    message: l2BridgeDryRunError,
                })
                estimatedGas = 0n
            }
            const feeData = await l2Chain.getFeeData()
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

        const assetHubImpl = await paraImplementation(assetHub)

        // Check if asset can be received on asset hub (dry run)
        const ahParachain = registry.parachains[`polkadot_${registry.assetHubParaId}`]
        let dryRunAhSuccess, assetHubDryRunError
        if (!ahParachain.features.hasDryRunApi) {
            logs.push({
                kind: ValidationKind.Error,
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
                l2BridgeDryRunError,
            },
            transfer,
        }
    }
}

import {
    AssetRegistry,
    ChainId,
    DepositParamsStruct,
    EthereumChain,
    EthereumProviderTypes,
    ISwapQuoter,
    Parachain,
    QuoteExactOutputSingleParamsStruct,
    SendParamsStruct,
    SwapParamsStruct,
    TransferRoute,
} from "@snowbridge/base-types"
import { TransferInterface } from "./transferInterface"
import { Context } from "../../index"
import {
    buildSwapCallData,
    calculateRelayerFee,
    claimerFromBeneficiary,
    claimerLocationToBytes,
    DeliveryFee,
    messageId as getSharedMessageReceipt,
    ValidationKind,
} from "../../toPolkadotSnowbridgeV2"
import {
    sendMessageXCM,
    buildAssetHubERC20ReceivedXcm,
} from "../../xcmbuilders/toPolkadot/erc20ToAH"
import { accountId32Location, erc20Location } from "../../xcmBuilder"
import {
    DOT_LOCATION,
    ETHER_TOKEN_ADDRESS,
    getAssetHubEtherMinBalance,
} from "../../assets_v2"
import { ensureValidationSuccess, padFeeByPercentage } from "../../utils"
import { resolveBeneficiary } from "../../crypto"
import { FeeInfo, ValidationLog, ValidationReason } from "../../types/toPolkadot"
import { buildMessageId, Transfer, ValidatedTransfer } from "../../toPolkadotSnowbridgeV2"
import { getOperatingStatus } from "../../status"
import { hexToU8a } from "@polkadot/util"
import { estimateFees } from "../../across/api"

export class ERC20ToAH<T extends EthereumProviderTypes> implements TransferInterface<T> {
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
        l2TokenAddress: string,
        amount: bigint,
        options?: {
            padFeeByPercentage?: bigint
            feeAsset?: any
            customXcm?: any[]
            overrideRelayerFee?: bigint
            l2PadFeeByPercentage?: bigint
            fillDeadlineBuffer?: bigint
        },
    ): Promise<DeliveryFee> {
        const context = this.context
        const registry = this.registry
        const { assetHub, bridgeHub } = {
            assetHub: await context.assetHub(),
            bridgeHub: await context.bridgeHub(),
        }
        if (registry.ethereumChains?.[`ethereum_l2_${this.from.id}`] == undefined) {
            throw new Error(`L2 Chain ID ${this.from.id} is not supported in the provided registry`)
        }
        if (
            registry.ethereumChains?.[`ethereum_l2_${this.from.id}`]?.assets[l2TokenAddress] ==
            undefined
        ) {
            throw new Error(
                `L2 Token Address ${l2TokenAddress} is not supported in the provided registry for L2 Chain ID ${this.from.id}`,
            )
        }
        let tokenAddress =
            registry.ethereumChains?.[`ethereum_l2_${this.from.id}`]?.assets[l2TokenAddress]
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
        const feePadPercentage = options?.padFeeByPercentage
        const feeAsset = options?.feeAsset || ether

        if (feeAsset !== ether) {
            throw new Error("only ether is supported as fee asset in this version of the API")
        }

        // Delivery fee BridgeHub to AssetHub
        const bridgeHubImpl = await this.context.paraImplementation(bridgeHub)
        const deliveryFeeInDOT = await bridgeHubImpl.calculateDeliveryFeeInDOT(
            registry.assetHubParaId,
            assetHubXcm,
        )

        const assetHubImpl = await this.context.paraImplementation(assetHub)
        const deliveryFeeInEther = await assetHubImpl.swapAsset1ForAsset2(
            DOT_LOCATION,
            ether,
            deliveryFeeInDOT,
        )
        // AssetHub Execution fee
        let assetHubExecutionFeeDOT = await assetHubImpl.calculateXcmFee(assetHubXcm, DOT_LOCATION)

        let assetHubExecutionFeeEther = padFeeByPercentage(
            await assetHubImpl.swapAsset1ForAsset2(DOT_LOCATION, ether, assetHubExecutionFeeDOT),
            feePadPercentage ?? 33n,
        )
        // For non-ether transfers, oversize executionFee by AH bridged-ether
        // min_balance: the post-PayFees surplus then naturally lands at the
        // recipient via RefundSurplus → DepositAsset, satisfying
        // `Token::BelowMinimum` on a fresh asset account.
        if (tokenAddress !== ETHER_TOKEN_ADDRESS) {
            assetHubExecutionFeeEther += getAssetHubEtherMinBalance(registry)
        }

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
        const acrossApiUrl = context.environment.l2Bridge?.acrossAPIUrl
        const l2FeeTokenAddress =
            context.environment.l2Bridge?.l2Chains[this.from.id]?.feeTokenAddress
        if (!acrossApiUrl || !l2FeeTokenAddress) {
            throw new Error("L2 bridge configuration is missing.")
        }
        if (l2TokenAddress == ETHER_TOKEN_ADDRESS || l2TokenAddress == l2FeeTokenAddress) {
            const l1FeeTokenAddress =
                registry.ethereumChains?.[`ethereum_l2_${this.from.id}`]?.assets[l2FeeTokenAddress]
                    ?.swapTokenAddress
            if (!l1FeeTokenAddress) {
                throw new Error("Fee token is not registered on Ethereum")
            }
            bridgeFeeInL2Token = await estimateFees(
                acrossApiUrl,
                l2FeeTokenAddress,
                l1FeeTokenAddress,
                this.from.id,
                registry.ethChainId,
                assetHubExecutionFeeEther + relayerFee + amount,
            )
            bridgeFeeInL2Token = padFeeByPercentage(
                bridgeFeeInL2Token,
                options?.l2PadFeeByPercentage ?? 33n,
            )
            totalFeeInWei += bridgeFeeInL2Token
        } else {
            let swapFee =
                registry.ethereumChains?.[`ethereum_l2_${this.from.id}`]?.assets[l2TokenAddress]
                    ?.swapFee
            const swapQuoter = context.l1SwapQuoter() as T["Contract"] & ISwapQuoter
            const l1FeeTokenAddress = context.environment.l2Bridge?.l1FeeTokenAddress
            if (!l1FeeTokenAddress) {
                throw new Error("L2 bridge configuration is missing.")
            }
            let params: QuoteExactOutputSingleParamsStruct = {
                tokenIn: tokenAddress,
                tokenOut: l1FeeTokenAddress,
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
            bridgeFeeInL2Token = await estimateFees(
                acrossApiUrl,
                l2TokenAddress,
                tokenAddress,
                this.from.id,
                registry.ethChainId,
                amount + swapFeeInL1Token,
            )
            bridgeFeeInL2Token = padFeeByPercentage(
                bridgeFeeInL2Token,
                options?.l2PadFeeByPercentage ?? 33n,
            )
        }

        return {
            kind: "ethereum_l2->polkadot",
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

    async tx(
        sourceAccount: string,
        beneficiaryAccount: string,
        l2TokenAddress: string,
        amount: bigint,
        fee: DeliveryFee,
        options?: {
            customXcm?: any[]
            fillDeadlineBuffer?: bigint
        },
    ): Promise<Transfer<T>> {
        const context = this.context
        const registry = this.registry
        const assetHub = await context.assetHub()
        const l2Chain = context.ethChain(this.from.id)

        let tokenAddress =
            registry.ethereumChains?.[`ethereum_l2_${this.from.id}`]?.assets[l2TokenAddress]
                ?.swapTokenAddress
        if (!tokenAddress) {
            throw new Error("Token is not registered on Ethereum")
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

        const { hexAddress: beneficiaryAddressHex } = resolveBeneficiary(beneficiaryAccount)
        const beneficiary = context.ethereumProvider.beneficiaryMultiAddress(beneficiaryAddressHex)

        let assets: any = []
        let value: bigint
        let inputAmount: bigint = amount

        const accountNonce = await context.ethereumProvider.getTransactionCount(
            l2Chain,
            sourceAccount,
            "pending",
        )

        const topic = buildMessageId(
            this.to.id,
            sourceAccount,
            l2TokenAddress,
            beneficiaryAddressHex,
            amount,
            accountNonce,
        )

        // For ether transfers there's only one asset in holding so no split is needed.
        const userAssetLocation =
            tokenAddress === ETHER_TOKEN_ADDRESS
                ? undefined
                : erc20Location(registry.ethChainId, tokenAddress)
        const xcm = hexToU8a(
            sendMessageXCM(
                assetHub.registry,
                beneficiaryAddressHex,
                topic,
                options?.customXcm,
                userAssetLocation,
            ).toHex(),
        )
        let claimer = claimerFromBeneficiary(assetHub, beneficiaryAddressHex, registry.environment)

        let depositParams: DepositParamsStruct, tx: T["ContractTransaction"]

        let sendParams: SendParamsStruct = {
            xcm: xcm,
            assets: assets,
            claimer: claimerLocationToBytes(claimer),
            executionFee: fee.assetHubExecutionFeeEther,
            relayerFee: fee.relayerFee,
        }
        const l2FeeTokenAddress =
            context.environment.l2Bridge?.l2Chains[this.from.id]?.feeTokenAddress
        const l1SwapRouterAddress = context.environment.l2Bridge?.l1SwapRouterAddress
        if (!l2FeeTokenAddress || !l1SwapRouterAddress) {
            throw new Error("L2 chain configuration is missing.")
        }
        if (l2TokenAddress === ETHER_TOKEN_ADDRESS || l2TokenAddress === l2FeeTokenAddress) {
            value = fee.totalFeeInWei + amount
            inputAmount =
                amount +
                (l2TokenAddress === l2FeeTokenAddress ? (fee.bridgeFeeInL2Token ?? 0n) : 0n)
            depositParams = {
                inputToken: l2TokenAddress,
                outputToken: tokenAddress,
                inputAmount: value,
                outputAmount: amount,
                destinationChainId: BigInt(registry.ethChainId),
                fillDeadlineBuffer: options?.fillDeadlineBuffer ?? 600n,
            }
            tx = await context.ethereumProvider.l2AdapterSendEtherAndCall(
                context.ethChain(this.from.id),
                context.environment.l2Bridge!.l2Chains[this.from.id].adapterAddress,
                sourceAccount,
                depositParams,
                sendParams,
                sourceAccount,
                topic,
                l2TokenAddress === ETHER_TOKEN_ADDRESS ? value : undefined,
            )
        } else {
            value = fee.totalFeeInWei
            inputAmount = amount + fee.bridgeFeeInL2Token! + fee.swapFeeInL1Token!
            assets = [context.ethereumProvider.encodeNativeAsset(tokenAddress, amount)]
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
                this.from.id,
                l2TokenAddress,
                fee.assetHubExecutionFeeEther + fee.relayerFee,
                fee.swapFeeInL1Token!,
            )
            let swapParams: SwapParamsStruct = {
                inputAmount: fee.swapFeeInL1Token!,
                router: l1SwapRouterAddress,
                callData: swapCalldata,
            }
            tx = await context.ethereumProvider.l2AdapterSendTokenAndCall(
                context.ethChain(this.from.id),
                context.environment.l2Bridge!.l2Chains[this.from.id].adapterAddress,
                sourceAccount,
                depositParams,
                swapParams,
                sendParams,
                sourceAccount,
                topic,
            )
        }

        return {
            kind: "ethereum_l2->polkadot",
            input: {
                registry,
                sourceAccount,
                beneficiaryAccount,
                tokenAddress,
                destinationParaId: this.to.id,
                amount,
                fee,
                customXcm: options?.customXcm,
                l2TokenAddress,
                sourceChainId: this.from.id,
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
                l2AdapterAddress:
                    context.environment.l2Bridge!.l2Chains[this.from.id].adapterAddress,
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
                padFeeByPercentage?: bigint
                feeAsset?: any
                customXcm?: any[]
                overrideRelayerFee?: bigint
                l2PadFeeByPercentage?: bigint
                fillDeadlineBuffer?: bigint
            }
            tx?: {
                customXcm?: any[]
                fillDeadlineBuffer?: bigint
            }
        },
    ): Promise<ValidatedTransfer<T>> {
        const fee = await this.fee(tokenAddress, amount, options?.fee)
        const transfer = await this.tx(
            sourceAccount,
            beneficiaryAccount,
            tokenAddress,
            amount,
            fee,
            options?.tx,
        )
        return ensureValidationSuccess(await this.validate(transfer))
    }

    async validate(transfer: Transfer<T>): Promise<ValidatedTransfer<T>> {
        const context = this.context
        const { tx } = transfer
        const { amount, sourceAccount, tokenAddress, registry, l2TokenAddress, sourceChainId } =
            transfer.input
        const { totalInputAmount } = transfer.computed
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
        const etherBalance = await context.ethereumProvider.getBalance(l2Chain, sourceAccount)

        let tokenBalance: { balance: bigint; gatewayAllowance: bigint }
        if (tokenAddress !== ETHER_TOKEN_ADDRESS) {
            tokenBalance = await context.ethereumProvider.erc20Balance(
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
        if (tokenBalance.gatewayAllowance < totalInputAmount) {
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.GatewaySpenderLimitReached,
                message:
                    "The Snowbridge L2 wrapper contract needs to approved as a spender for this token and amount.",
            })
        }

        if (tokenBalance.balance < totalInputAmount) {
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
                estimatedGas = await context.ethereumProvider.estimateGas(l2Chain, tx)
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
            const feeData = await context.ethereumProvider.getFeeData(l2Chain)
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
        const bridgeStatus = await getOperatingStatus({
            ethereumProvider: context.ethereumProvider,
            gateway,
            bridgeHub,
        })
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

        const assetHubImpl = await this.context.paraImplementation(assetHub)

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
            const executionFee = transfer.input.fee.assetHubExecutionFeeEther
            const payloadValue =
                transfer.computed.totalValue - executionFee - transfer.input.fee.relayerFee
            const xcm = buildAssetHubERC20ReceivedXcm(
                assetHub.registry,
                registry.ethChainId,
                tokenAddress,
                payloadValue,
                executionFee,
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
                totalInputAmount,
                etherBalance,
                tokenBalance,
                feeInfo,
                bridgeStatus,
                assetHubDryRunError,
                l2BridgeDryRunError,
            },
            ...transfer,
        }
    }

    async messageId(receipt: T["TransactionReceipt"]) {
        return getSharedMessageReceipt(this.context.ethereumProvider, receipt)
    }
}

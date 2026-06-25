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
    ValidationKind,
} from "../../toPolkadotSnowbridgeV2"
import {
    sendMessageXCM,
    buildAssetHubERC20ReceivedXcm,
} from "../../xcmbuilders/toPolkadot/erc20ToAH"
import { accountId32Location, erc20Location } from "../../xcmBuilder"
import { DOT_LOCATION, ETHER_TOKEN_ADDRESS, getAssetHubEtherMinBalance } from "../../assets_v2"
import { ensureValidationSuccess, padFeeByPercentage } from "../../utils"
import { resolveBeneficiary } from "../../crypto"
import { FeeInfo, ValidationLog, ValidationReason } from "../../types/toPolkadot"
import { buildMessageId, Transfer, ValidatedTransfer } from "../../toPolkadotSnowbridgeV2"
import { getOperatingStatus } from "../../status"
import { hexToU8a } from "@polkadot/util"
import { estimateFees } from "../../across/api"
import { VolumeFeeParams, calculateVolumeTipInWei } from "../../feeSchedule"
import {
    addBreakdown,
    computeTotals,
    findInBreakdown,
    findInBreakdownOrZero,
    findTotal,
} from "../../fees"
import { checkDotEthPoolLiquidityForEthereumToPolkadot } from "../../poolReserves"

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

    // Reconstructs the Across `message` (the on-chain `Instructions` the L2
    // adaptor emits) so the `/suggested-fees` estimate includes the
    // destination message-execution gas. Uses placeholder beneficiary/topic —
    // the relayer gas does not depend on those, only on the message shape.
    private buildAcrossMessage(
        assetHub: any,
        tokenAddress: string,
        amount: bigint,
        executionFee: bigint,
        relayerFee: bigint,
        customXcm?: any[],
        swap?: { router: string; inputAmount: bigint; callData: string },
    ): string {
        const { context, registry } = this
        const placeholder = "0x0000000000000000000000000000000000000000000000000000000000000000"
        const l1Weth = context.environment.l2Bridge?.l1FeeTokenAddress
        if (!l1Weth) {
            throw new Error("L2 bridge configuration is missing.")
        }
        const userAssetLocation =
            tokenAddress === ETHER_TOKEN_ADDRESS
                ? undefined
                : erc20Location(registry.ethChainId, tokenAddress)
        const xcm = hexToU8a(
            sendMessageXCM(
                assetHub.registry,
                placeholder,
                placeholder,
                customXcm,
                userAssetLocation,
            ).toHex(),
        )
        const claimer = claimerLocationToBytes(
            claimerFromBeneficiary(assetHub, placeholder, registry.environment),
        )
        const assets = swap
            ? [context.ethereumProvider.encodeNativeAsset(tokenAddress, amount)]
            : []
        return context.ethereumProvider.buildAcrossDepositMessage({
            outputToken: tokenAddress,
            gateway: registry.gatewayAddress,
            l1Weth,
            fallbackRecipient: "0x0000000000000000000000000000000000000001",
            xcm,
            assets,
            claimer,
            executionFee,
            relayerFee,
            destinationExecutionFee: 0n,
            outputAmount: amount,
            swap,
        })
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
            volumeFee?: VolumeFeeParams
        },
    ): Promise<DeliveryFee> {
        if (options?.volumeFee && options?.overrideRelayerFee !== undefined) {
            throw new Error("Cannot specify both volumeFee and overrideRelayerFee")
        }
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
        // Quote in the runtime swap direction (ETH->DOT) so the LP fee lands
        // on the ETH input side, matching what AH's SwapFirstAssetTrader does.
        const deliveryFeeInEther = await assetHubImpl.getAssetHubConversionPalletSwap(
            ether,
            DOT_LOCATION,
            deliveryFeeInDOT,
        )
        // AssetHub Execution fee. Always compute the DOT figure for the fee
        // breakdown. Prefer AH's direct weight->ether quote when available,
        // fall back to the manual ETH->DOT swap.
        let assetHubExecutionFeeDOT = await assetHubImpl.calculateXcmFee(assetHubXcm, DOT_LOCATION)

        let assetHubExecutionFeeEtherRaw: bigint
        try {
            assetHubExecutionFeeEtherRaw = await assetHubImpl.calculateXcmFee(assetHubXcm, ether)
        } catch {
            assetHubExecutionFeeEtherRaw = await assetHubImpl.getAssetHubConversionPalletSwap(
                ether,
                DOT_LOCATION,
                assetHubExecutionFeeDOT,
            )
        }
        let assetHubExecutionFeeEther = padFeeByPercentage(
            assetHubExecutionFeeEtherRaw,
            feePadPercentage ?? 50n,
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

        let volumeTip: bigint | undefined
        let finalRelayerFee = relayerFee
        if (options?.volumeFee) {
            volumeTip = calculateVolumeTipInWei(options.volumeFee)
            finalRelayerFee += volumeTip
        }

        // Calculate fee with Across SDK
        let bridgeFeeInL2Token = 0n,
            swapFeeInL1Token = 0n
        let totalFeeInWei = assetHubExecutionFeeEther + finalRelayerFee
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
            const l1HandlerAddress = context.environment.l2Bridge?.l1HandlerAddress
            if (!l1HandlerAddress) {
                throw new Error("L2 bridge configuration is missing.")
            }
            const message = this.buildAcrossMessage(
                assetHub,
                tokenAddress,
                amount,
                assetHubExecutionFeeEther,
                finalRelayerFee,
                options?.customXcm,
            )
            bridgeFeeInL2Token = await estimateFees(
                acrossApiUrl,
                l2FeeTokenAddress,
                l1FeeTokenAddress,
                this.from.id,
                registry.ethChainId,
                assetHubExecutionFeeEther + finalRelayerFee + amount,
                { recipient: l1HandlerAddress, message },
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
                amount: assetHubExecutionFeeEther + finalRelayerFee,
                fee: swapFee ?? 500, // 0.05% pool fee
                sqrtPriceLimitX96: 0, // no price limit
            }
            let result = await swapQuoter.quoteExactOutputSingle.staticCall(params)
            swapFeeInL1Token = result[0] as bigint
            swapFeeInL1Token = padFeeByPercentage(
                swapFeeInL1Token,
                options?.l2PadFeeByPercentage ?? 33n,
            )
            const l1SwapRouterAddress = context.environment.l2Bridge?.l1SwapRouterAddress
            const l1HandlerAddress = context.environment.l2Bridge?.l1HandlerAddress
            if (!l1SwapRouterAddress || !l1HandlerAddress) {
                throw new Error("L2 bridge configuration is missing.")
            }
            const swapCallData = await buildSwapCallData(
                context,
                registry,
                this.from.id,
                l2TokenAddress,
                assetHubExecutionFeeEther + finalRelayerFee,
                swapFeeInL1Token,
            )
            const message = this.buildAcrossMessage(
                assetHub,
                tokenAddress,
                amount,
                assetHubExecutionFeeEther,
                finalRelayerFee,
                options?.customXcm,
                {
                    router: l1SwapRouterAddress,
                    inputAmount: swapFeeInL1Token,
                    callData: swapCallData,
                },
            )
            bridgeFeeInL2Token = await estimateFees(
                acrossApiUrl,
                l2TokenAddress,
                tokenAddress,
                this.from.id,
                registry.ethChainId,
                amount + swapFeeInL1Token,
                { recipient: l1HandlerAddress, message },
            )
            bridgeFeeInL2Token = padFeeByPercentage(
                bridgeFeeInL2Token,
                options?.l2PadFeeByPercentage ?? 33n,
            )
        }

        const l2TokenMeta =
            registry.ethereumChains?.[`ethereum_l2_${this.from.id}`]?.assets[l2TokenAddress]
        const l2FeeTokenMeta = l2FeeTokenAddress
            ? registry.ethereumChains?.[`ethereum_l2_${this.from.id}`]?.assets[l2FeeTokenAddress]
            : undefined
        const l2FeeSymbol = l2FeeTokenMeta?.symbol ?? l2FeeTokenAddress ?? "L2_FEE"
        const l1TokenMeta = l2TokenMeta?.swapTokenAddress
            ? registry.ethereumChains?.[`ethereum_${registry.ethChainId}`]?.assets[
                  l2TokenMeta.swapTokenAddress.toLowerCase()
              ]
            : undefined
        const l1TokenSymbol = l1TokenMeta?.symbol ?? "L1_TOKEN"

        const breakdown: DeliveryFee["breakdown"] = {}
        addBreakdown(breakdown, "assetHubDelivery", { amount: deliveryFeeInEther, symbol: "ETH" })
        addBreakdown(breakdown, "assetHubDelivery", { amount: deliveryFeeInDOT, symbol: "DOT" })
        addBreakdown(breakdown, "assetHubExecution", {
            amount: assetHubExecutionFeeEther,
            symbol: "ETH",
        })
        addBreakdown(breakdown, "assetHubExecution", {
            amount: assetHubExecutionFeeDOT,
            symbol: "DOT",
        })
        addBreakdown(breakdown, "relayer", { amount: finalRelayerFee, symbol: "ETH" })
        addBreakdown(breakdown, "extrinsic", { amount: extrinsicFeeDot, symbol: "DOT" })
        addBreakdown(breakdown, "extrinsic", { amount: extrinsicFeeEther, symbol: "ETH" })
        if (bridgeFeeInL2Token > 0n) {
            addBreakdown(breakdown, "l2Bridge", { amount: bridgeFeeInL2Token, symbol: l2FeeSymbol })
        }
        if (swapFeeInL1Token > 0n) {
            addBreakdown(breakdown, "l1Swap", { amount: swapFeeInL1Token, symbol: l1TokenSymbol })
        }

        const summary: DeliveryFee["summary"] = [
            {
                description: "XCM execution fees",
                amount: assetHubExecutionFeeEther,
                symbol: "ETH",
            },
            { description: "Bridge fees", amount: deliveryFeeInEther, symbol: "ETH" },
            {
                description: "Relayer tip",
                amount: finalRelayerFee - deliveryFeeInEther,
                symbol: "ETH",
            },
        ]
        if (bridgeFeeInL2Token > 0n) {
            summary.push({
                description: "Across fee",
                amount: bridgeFeeInL2Token,
                symbol: l2FeeSymbol,
            })
        }
        if (swapFeeInL1Token > 0n) {
            summary.push({
                description: "Swap fee",
                amount: swapFeeInL1Token,
                symbol: l1TokenSymbol,
            })
        }

        return {
            kind: "ethereum_l2->polkadot",
            feeAsset,
            breakdown,
            summary,
            totals: computeTotals(summary),
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
            executionFee: findInBreakdown(fee.breakdown, "assetHubExecution", "ETH"),
            relayerFee: findInBreakdown(fee.breakdown, "relayer", "ETH"),
            destinationExecutionFee: 0n,
        }
        const l2FeeTokenAddress =
            context.environment.l2Bridge?.l2Chains[this.from.id]?.feeTokenAddress
        const l1SwapRouterAddress = context.environment.l2Bridge?.l1SwapRouterAddress
        if (!l2FeeTokenAddress || !l1SwapRouterAddress) {
            throw new Error("L2 chain configuration is missing.")
        }
        const totalFeeInWei = findTotal(fee, "ETH")
        const bridgeFeeInL2Token = (fee.breakdown.l2Bridge ?? []).reduce((s, a) => s + a.amount, 0n)
        const swapFeeInL1Token = (fee.breakdown.l1Swap ?? []).reduce((s, a) => s + a.amount, 0n)
        if (l2TokenAddress === ETHER_TOKEN_ADDRESS || l2TokenAddress === l2FeeTokenAddress) {
            // bridgeFeeInL2Token is denominated in the L2 fee token (WETH on
            // OP/Base/Arbitrum), which is 1:1 with ETH wei but lives in its own
            // symbol bucket in fee.totals. Add it explicitly so msg.value /
            // depositParams.inputAmount cover the Across leg.
            value = totalFeeInWei + amount + bridgeFeeInL2Token
            // The adaptor pulls exactly depositParams.inputAmount (= value) from the
            // user — as msg.value for native ETH, or via WETH safeTransferFrom on the
            // fee-token path. totalInputAmount must equal that so validation checks the
            // real balance/allowance requirement; otherwise a WETH send passes
            // validation and then reverts in sendEtherAndCall.
            inputAmount = value
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
            value = totalFeeInWei
            inputAmount = amount + bridgeFeeInL2Token + swapFeeInL1Token
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
                findInBreakdown(fee.breakdown, "assetHubExecution", "ETH") +
                    findInBreakdown(fee.breakdown, "relayer", "ETH"),
                swapFeeInL1Token,
            )
            let swapParams: SwapParamsStruct = {
                inputAmount: swapFeeInL1Token,
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
                volumeFee?: VolumeFeeParams
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

        const requiredDotOut =
            findInBreakdownOrZero(transfer.input.fee.breakdown, "assetHubDelivery", "DOT") +
            findInBreakdownOrZero(transfer.input.fee.breakdown, "assetHubExecution", "DOT")
        if (requiredDotOut > 0n) {
            const reserveCheck = await checkDotEthPoolLiquidityForEthereumToPolkadot(
                assetHubImpl,
                registry.ethChainId,
                requiredDotOut,
            )
            if (!reserveCheck.ok) {
                logs.push({
                    kind: ValidationKind.Error,
                    reason: ValidationReason.InsufficientPoolReserves,
                    message:
                        reserveCheck.reason === "pool-missing"
                            ? `${reserveCheck.pool} pool does not exist on Asset Hub.`
                            : `${reserveCheck.pool} pool on Asset Hub has insufficient liquidity (need ${reserveCheck.requiredOut}, have ${reserveCheck.reserveOut}).`,
                })
            }
        }

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
            const executionFee = findInBreakdown(
                transfer.input.fee.breakdown,
                "assetHubExecution",
                "ETH",
            )
            const relayerFee = findInBreakdown(transfer.input.fee.breakdown, "relayer", "ETH")
            // On the ETH/WETH fee-token path tx() folds the Across origin fee
            // (bridgeFeeInL2Token) into totalValue. That spread is kept by the Across
            // relayer and never reaches the gateway, so the Ether actually delivered to
            // Asset Hub is totalValue - bridgeFee. Exclude it when reconstructing the
            // dry-run payload; the swap path does not add it to totalValue.
            const l2FeeTokenAddress =
                context.environment.l2Bridge?.l2Chains[this.from.id]?.feeTokenAddress
            const bridgeFeeInL2Token = (transfer.input.fee.breakdown.l2Bridge ?? []).reduce(
                (s, a) => s + a.amount,
                0n,
            )
            const deliveredValue =
                l2TokenAddress === ETHER_TOKEN_ADDRESS || l2TokenAddress === l2FeeTokenAddress
                    ? transfer.computed.totalValue - bridgeFeeInL2Token
                    : transfer.computed.totalValue
            const payloadValue = deliveredValue - executionFee - relayerFee
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
        return this.context.ethereumProvider.scanL2WrapperDepositCallInvoked(receipt)
    }
}

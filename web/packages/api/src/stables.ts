import { ApiPromise } from "@polkadot/api"
import { AddressOrPair, SignerOptions, SubmittableExtrinsic } from "@polkadot/api/types"
import { ISubmittableResult } from "@polkadot/types/types"
import { Result } from "@polkadot/types"
import { CallDryRunEffects, EventRecord, XcmDryRunApiError } from "@polkadot/types/interfaces"
import { isHex, stringToU8a, u8aConcat, u8aToHex } from "@polkadot/util"
import { blake2AsU8a, decodeAddress } from "@polkadot/util-crypto"
import { BridgeInfo, EthereumProviderTypes, TransferRoute } from "@snowbridge/base-types"
import { Context } from "."
import { DOT_LOCATION } from "./assets_v2"
import { dryRunTx } from "./forInterParachain"
import { runEthereumDryRun } from "./dryRunEthereum"
import { findTotalOrUndefined } from "./fees"
import { getOperatingStatus } from "./status"
import {
    dryRunAssetHub,
    dryRunOnSourceParachain,
    TransferToEthereum,
} from "./toEthereumSnowbridgeV2"
import { dryRunBridgeHub } from "./toEthereum_v2"
import type { DeliveryFee, Transfer, ValidationLog } from "./types/toEthereum"
import { ValidationKind, ValidationReason } from "./types/toEthereum"
import type { VolumeFeeParams } from "./feeSchedule"

// Stables moved from Asset Hub to Hydration, swapped, then bridged to Ethereum.
export const HYDRATION_PARA_ID = 2034
const ASSET_HUB_ASSETS_PALLET = 50
const HYDRATION_DOT_ASSET_ID = 5
const HYDRATION_NATIVE_ASSET_ID = 0

export type HydrationStableSymbol = "HOLLAR" | "USDT" | "USDC"
export type EthereumStableSymbol = "USDT" | "USDC"

export type HydrationStable = {
    symbol: HydrationStableSymbol
    name: string
    decimals: number
    hydrationAssetId: number
    locationOnAssetHub: any
    // Unset for foreign assets (HOLLAR).
    assetHubAssetId?: number
    // Quote source when the user holds none; not a route pool, or quoting moves its price.
    quotePoolId: number
}

export type EthereumStable = {
    symbol: EthereumStableSymbol
    token: string
    hydrationAssetId: number
    decimals: number
}

export type Trade = { pool: any; assetIn: number; assetOut: number }

const HOLLAR_ID = 222
const AH_USDT_ID = 10
const AH_USDC_ID = 22
const AUSDT_ID = 1002
const ETH_USDT_ID = 1000767
const ETH_USDC_ID = 1000766
const POOL_HOLLAR_AUSDT = 111
const POOL_HOLLAR_AUSDC = 110
const POOL_AUSDT_ETH_STABLES = 103
const POOL_AH_USDT_USDC = 102

export const HYDRATION_STABLES: Record<HydrationStableSymbol, HydrationStable> = {
    HOLLAR: {
        symbol: "HOLLAR",
        name: "Hydrated Dollar",
        decimals: 18,
        hydrationAssetId: HOLLAR_ID,
        locationOnAssetHub: {
            parents: 1,
            interior: { X2: [{ Parachain: HYDRATION_PARA_ID }, { GeneralIndex: HOLLAR_ID }] },
        },
        quotePoolId: POOL_HOLLAR_AUSDC,
    },
    USDT: {
        symbol: "USDT",
        name: "Tether USD (Asset Hub)",
        decimals: 6,
        hydrationAssetId: AH_USDT_ID,
        assetHubAssetId: 1984,
        locationOnAssetHub: {
            parents: 0,
            interior: { X2: [{ PalletInstance: ASSET_HUB_ASSETS_PALLET }, { GeneralIndex: 1984 }] },
        },
        quotePoolId: POOL_AH_USDT_USDC,
    },
    USDC: {
        symbol: "USDC",
        name: "USD Coin (Asset Hub)",
        decimals: 6,
        hydrationAssetId: AH_USDC_ID,
        assetHubAssetId: 1337,
        locationOnAssetHub: {
            parents: 0,
            interior: { X2: [{ PalletInstance: ASSET_HUB_ASSETS_PALLET }, { GeneralIndex: 1337 }] },
        },
        quotePoolId: POOL_AH_USDT_USDC,
    },
}

export const ETHEREUM_STABLES: Record<EthereumStableSymbol, EthereumStable> = {
    USDT: {
        symbol: "USDT",
        token: "0xdac17f958d2ee523a2206206994597c13d831ec7",
        hydrationAssetId: ETH_USDT_ID,
        decimals: 6,
    },
    USDC: {
        symbol: "USDC",
        token: "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
        hydrationAssetId: ETH_USDC_ID,
        decimals: 6,
    },
}

// Explicit router trades; Hydration has no on-chain routes for these pairs.
const toEthStables = (via: Trade[]): Record<EthereumStableSymbol, Trade[]> => ({
    USDT: [
        ...via,
        { pool: { Stableswap: POOL_AUSDT_ETH_STABLES }, assetIn: AUSDT_ID, assetOut: ETH_USDT_ID },
    ],
    USDC: [
        ...via,
        { pool: { Stableswap: POOL_AUSDT_ETH_STABLES }, assetIn: AUSDT_ID, assetOut: ETH_USDC_ID },
    ],
})
export const SWAP_ROUTES: Record<HydrationStableSymbol, Record<EthereumStableSymbol, Trade[]>> = {
    HOLLAR: toEthStables([
        { pool: { Stableswap: POOL_HOLLAR_AUSDT }, assetIn: HOLLAR_ID, assetOut: AUSDT_ID },
    ]),
    USDT: toEthStables([{ pool: { Aave: null }, assetIn: AH_USDT_ID, assetOut: AUSDT_ID }]),
    USDC: toEthStables([
        { pool: { Stableswap: POOL_AH_USDT_USDC }, assetIn: AH_USDC_ID, assetOut: AH_USDT_ID },
        { pool: { Aave: null }, assetIn: AH_USDT_ID, assetOut: AUSDT_ID },
    ]),
}

export type StableBalances = {
    assetHub: Record<HydrationStableSymbol, bigint>
    hydration: Record<HydrationStableSymbol, bigint>
    assetHubDot: bigint
    hydrationDot: bigint
    hydrationNative: bigint
}

export type MoveToHydrationTransfer = {
    kind: "stables:assethub->hydration"
    stable: HydrationStable
    sourceAccount: string
    sourceAccountHex: string
    amount: bigint
    tx: SubmittableExtrinsic<"promise", ISubmittableResult>
}

export type ValidatedMoveToHydration = MoveToHydrationTransfer & {
    success: boolean
    logs: ValidationLog[]
    data: {
        sourceExecutionFee: bigint
        assetHubDryRunError?: any
        hydrationDryRunError?: any
    }
}

export type SwapQuote = {
    from: HydrationStable
    to: EthereumStable
    route: Trade[]
    amountIn: bigint
    amountOut: bigint
}

export type SwapAndBridgeTransfer = {
    kind: "stables:hydration->ethereum"
    from: HydrationStable
    to: EthereumStable
    sourceAccount: string
    sourceAccountHex: string
    beneficiaryAccount: string
    amountIn: bigint
    minAmountOut: bigint
    fee: DeliveryFee
    messageId: string
    bridge: Transfer
    // utility.batchAll([router.sell, bridge.tx])
    tx: SubmittableExtrinsic<"promise", ISubmittableResult>
}

export type ValidatedSwapAndBridge = SwapAndBridgeTransfer & {
    success: boolean
    logs: ValidationLog[]
    data: {
        sourceExecutionFee: bigint
        sourceDryRunError?: any
        assetHubDryRunError?: any
        bridgeHubDryRunError?: any
        ethereumDryRunError?: any
    }
}

export type SubmitReceipt = {
    txHash: string
    txIndex: number
    blockNumber: number
    blockHash: string
    success: boolean
    dispatchError?: any
    messageId?: string
    events: EventRecord[]
}

function toHexAccount(account: string): string {
    return isHex(account) ? account : u8aToHex(decodeAddress(account))
}

// pallet_stableswap pool account: blake2_256("sts" ++ pool_id LE).
export function stableswapPoolAccountHex(poolId: number): string {
    const le = new Uint8Array(4)
    new DataView(le.buffer).setUint32(0, poolId, true)
    return u8aToHex(blake2AsU8a(u8aConcat(stringToU8a("sts"), le), 256))
}

// Pads the fallback price to cover drift from the oracle price.
const FEE_CURRENCY_PRICE_PAD = 2n

async function hydrationFeeInCurrency(
    hydration: ApiPromise,
    currency: number,
    hdxFee: bigint,
): Promise<bigint> {
    const price = (
        await hydration.query.multiTransactionPayment.acceptedCurrencies(currency)
    ).toPrimitive()
    if (price === null || price === undefined) {
        throw new Error(`Hydration does not accept currency ${currency} for fees.`)
    }
    return (hdxFee * BigInt(price.toString()) * FEE_CURRENCY_PRICE_PAD) / 10n ** 18n
}

const balanceOf = (x: any): bigint => {
    const p = x?.toPrimitive?.() ?? x
    if (p === null || p === undefined) return 0n
    if (typeof p === "bigint") return p
    return BigInt(p.balance ?? p.free ?? p.data?.free ?? 0)
}

export class StablesTransfer<T extends EthereumProviderTypes> {
    readonly info: BridgeInfo
    readonly context: Context<T>

    constructor(info: BridgeInfo, context: Context<T>) {
        this.info = info
        this.context = context
    }

    #bridgeRoute(): TransferRoute {
        const { ethChainId } = this.info.registry
        const route = this.info.routes.find(
            (r) =>
                r.from.kind === "polkadot" &&
                r.from.id === HYDRATION_PARA_ID &&
                r.to.kind === "ethereum" &&
                r.to.id === ethChainId,
        )
        if (!route) {
            throw Error(`No Hydration to Ethereum route in the registry.`)
        }
        return route
    }

    #bridgeSender(): TransferToEthereum<T> {
        const { registry } = this.info
        const route = this.#bridgeRoute()
        const hydration = registry.parachains[`polkadot_${HYDRATION_PARA_ID}`]
        const ethereum = registry.ethereumChains[`ethereum_${registry.ethChainId}`]
        return new TransferToEthereum(this.context, route, registry, hydration, ethereum)
    }

    async balances(account: string): Promise<StableBalances> {
        const accountHex = toHexAccount(account)
        const [assetHub, hydration] = await Promise.all([
            this.context.assetHub(),
            this.context.parachain(HYDRATION_PARA_ID),
        ])
        const [assetHubImpl, hydrationImpl] = await Promise.all([
            this.context.paraImplementation(assetHub),
            this.context.paraImplementation(hydration),
        ])
        const symbols = Object.keys(HYDRATION_STABLES) as HydrationStableSymbol[]
        const [ahBalances, hyBalances, ahAccount, hyDot, hyAccount] = await Promise.all([
            Promise.all(
                symbols.map((s) => {
                    const stable = HYDRATION_STABLES[s]
                    return stable.assetHubAssetId !== undefined
                        ? assetHub.query.assets.account(stable.assetHubAssetId, accountHex)
                        : assetHub.query.foreignAssets.account(
                              stable.locationOnAssetHub,
                              accountHex,
                          )
                }),
            ),
            Promise.all(
                symbols.map((s) =>
                    hydration.call.currenciesApi.account(
                        HYDRATION_STABLES[s].hydrationAssetId,
                        accountHex,
                    ),
                ),
            ),
            assetHubImpl.getNativeBalance(accountHex, true),
            hydration.query.tokens.accounts(accountHex, HYDRATION_DOT_ASSET_ID),
            hydrationImpl.getNativeBalance(accountHex, true),
        ])
        const record = (values: any[]) =>
            Object.fromEntries(symbols.map((s, i) => [s, balanceOf(values[i])])) as Record<
                HydrationStableSymbol,
                bigint
            >
        return {
            assetHub: record(ahBalances),
            hydration: record(hyBalances),
            assetHubDot: balanceOf(ahAccount),
            hydrationDot: balanceOf(hyDot),
            hydrationNative: balanceOf(hyAccount),
        }
    }

    // Leg 1: Asset Hub -> Hydration; Hydration execution is paid in the stable.
    async moveToHydrationTx(
        sourceAccount: string,
        symbol: HydrationStableSymbol,
        amount: bigint,
    ): Promise<MoveToHydrationTransfer> {
        const stable = HYDRATION_STABLES[symbol]
        const assetHub = await this.context.assetHub()
        const sourceAccountHex = toHexAccount(sourceAccount)
        const tx = assetHub.tx.polkadotXcm.transferAssets(
            { v4: { parents: 1, interior: { X1: [{ Parachain: HYDRATION_PARA_ID }] } } },
            {
                v4: {
                    parents: 0,
                    interior: { X1: [{ AccountId32: { network: null, id: sourceAccountHex } }] },
                },
            },
            { v4: [{ id: stable.locationOnAssetHub, fun: { Fungible: amount } }] },
            0,
            "Unlimited",
        )
        return {
            kind: "stables:assethub->hydration",
            stable,
            sourceAccount,
            sourceAccountHex,
            amount,
            tx,
        }
    }

    async validateMoveToHydration(
        transfer: MoveToHydrationTransfer,
    ): Promise<ValidatedMoveToHydration> {
        const { registry } = this.info
        const { stable } = transfer
        const logs: ValidationLog[] = []
        const [assetHub, hydration] = await Promise.all([
            this.context.assetHub(),
            this.context.parachain(HYDRATION_PARA_ID),
        ])
        const balances = await this.balances(transfer.sourceAccountHex)
        const paymentInfo = await transfer.tx.paymentInfo(transfer.sourceAccountHex)
        const sourceExecutionFee = paymentInfo["partialFee"].toBigInt()

        const assetInfo = (
            stable.assetHubAssetId !== undefined
                ? await assetHub.query.assets.asset(stable.assetHubAssetId)
                : await assetHub.query.foreignAssets.asset(stable.locationOnAssetHub)
        ).toPrimitive() as any
        const minBalance = BigInt(assetInfo?.minBalance ?? 0)
        if (transfer.amount < minBalance) {
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.InsufficientTokenBalance,
                message: `Amount is below the ${stable.symbol} minimum balance of ${minBalance}.`,
            })
        }
        if (transfer.amount > balances.assetHub[stable.symbol]) {
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.InsufficientTokenBalance,
                message: `Insufficient ${stable.symbol} balance on Asset Hub.`,
            })
        }
        if (sourceExecutionFee > balances.assetHubDot) {
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.InsufficientNativeFee,
                message: "Insufficient DOT on Asset Hub to pay the transaction fee.",
            })
        }

        let assetHubDryRunError
        let hydrationDryRunError
        const dryRunSource = await dryRunTx(
            assetHub,
            HYDRATION_PARA_ID,
            transfer.tx,
            transfer.sourceAccountHex,
        )
        if (!dryRunSource.success) {
            assetHubDryRunError = dryRunSource.error
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.DryRunFailed,
                message: "Dry run on Asset Hub failed.",
            })
        } else {
            const hydrationImpl = await this.context.paraImplementation(hydration)
            const dryRunDest = await hydrationImpl.dryRunXcm(
                registry.assetHubParaId,
                dryRunSource.forwardedXcm,
            )
            if (!dryRunDest.success) {
                hydrationDryRunError = dryRunDest.errorMessage
                logs.push({
                    kind: ValidationKind.Error,
                    reason: ValidationReason.DryRunFailed,
                    message: "Dry run on Hydration failed.",
                })
            }
        }

        return {
            ...transfer,
            success: logs.find((l) => l.kind === ValidationKind.Error) === undefined,
            logs,
            data: { sourceExecutionFee, assetHubDryRunError, hydrationDryRunError },
        }
    }

    async signAndSendMoveToHydration(
        transfer: MoveToHydrationTransfer,
        account: AddressOrPair,
        options: Partial<SignerOptions>,
    ): Promise<SubmitReceipt> {
        const assetHub = await this.context.assetHub()
        return signAndSend(assetHub, transfer.tx, account, options)
    }

    // Dry-runs router.sell, from the quote pool account if the user holds none.
    async quote(
        sourceAccount: string,
        fromSymbol: HydrationStableSymbol,
        toSymbol: EthereumStableSymbol,
        amountIn: bigint,
    ): Promise<SwapQuote> {
        const from = HYDRATION_STABLES[fromSymbol]
        const to = ETHEREUM_STABLES[toSymbol]
        const route = SWAP_ROUTES[fromSymbol][toSymbol]
        const hydration = await this.context.parachain(HYDRATION_PARA_ID)
        const sell = hydration.tx.router.sell(
            from.hydrationAssetId,
            to.hydrationAssetId,
            amountIn,
            1n,
            route,
        )
        const candidates = [toHexAccount(sourceAccount), stableswapPoolAccountHex(from.quotePoolId)]

        let lastError: any
        for (const who of candidates) {
            const result = await hydration.call.dryRunApi.dryRunCall<
                Result<CallDryRunEffects, XcmDryRunApiError>
            >({ system: { signed: who } }, sell, 4)
            if (!result.isOk || !result.asOk.executionResult.isOk) {
                lastError = result.toHuman()
                continue
            }
            // The last Swapped3 has the final amount.
            let amountOut: bigint | undefined
            for (const event of result.asOk.emittedEvents) {
                if (!hydration.events.broadcast.Swapped3.is(event)) continue
                const names = (event.data as any).names as string[] | undefined
                const index = names?.indexOf("outputs") ?? -1
                const outputs = (event.data.toPrimitive() as any[])[index >= 0 ? index : 5] ?? []
                for (const out of outputs) {
                    if (Number(out.asset) === to.hydrationAssetId) {
                        amountOut = BigInt(out.amount)
                    }
                }
            }
            if (amountOut === undefined) {
                throw Error("Swap dry run emitted no output for the target asset.")
            }
            return { from, to, route, amountIn, amountOut }
        }
        throw Error(`Could not quote ${fromSymbol} to ${toSymbol}: ${JSON.stringify(lastError)}`)
    }

    async swapAndBridgeFee(
        toSymbol: EthereumStableSymbol,
        options?: {
            padFeeByPercentage?: bigint
            slippagePadPercentage?: bigint
            volumeFee?: VolumeFeeParams
        },
    ): Promise<DeliveryFee> {
        return this.#bridgeSender().fee(ETHEREUM_STABLES[toSymbol].token, {
            feeTokenLocation: DOT_LOCATION,
            padFeeByPercentage: options?.padFeeByPercentage,
            slippagePadPercentage: options?.slippagePadPercentage ?? 20n,
            volumeFee: options?.volumeFee,
        })
    }

    // Leg 2: swap and bridge `minAmountOut` in one batch; any surplus stays on Hydration.
    async swapAndBridgeTx(
        sourceAccount: string,
        beneficiaryAccount: string,
        quote: SwapQuote,
        slippageBps: bigint,
        fee: DeliveryFee,
    ): Promise<SwapAndBridgeTransfer> {
        const { from, to, route, amountIn, amountOut } = quote
        const minAmountOut = amountOut - (amountOut * slippageBps) / 10_000n
        const hydration = await this.context.parachain(HYDRATION_PARA_ID)
        const bridge = await this.#bridgeSender().tx(
            sourceAccount,
            beneficiaryAccount,
            to.token,
            minAmountOut,
            fee,
        )
        const sell = hydration.tx.router.sell(
            from.hydrationAssetId,
            to.hydrationAssetId,
            amountIn,
            minAmountOut,
            route,
        )
        const tx = hydration.tx.utility.batchAll([sell, bridge.tx])
        return {
            kind: "stables:hydration->ethereum",
            from,
            to,
            sourceAccount,
            sourceAccountHex: toHexAccount(sourceAccount),
            beneficiaryAccount,
            amountIn,
            minAmountOut,
            fee,
            messageId: bridge.computed.messageId!,
            bridge,
            tx,
        }
    }

    async validateSwapAndBridge(transfer: SwapAndBridgeTransfer): Promise<ValidatedSwapAndBridge> {
        const { registry } = this.info
        const logs: ValidationLog[] = []
        const [hydration, assetHub, bridgeHub] = await Promise.all([
            this.context.parachain(HYDRATION_PARA_ID),
            this.context.assetHub(),
            this.context.bridgeHub(),
        ])
        const balances = await this.balances(transfer.sourceAccountHex)
        const paymentInfo = await transfer.tx.paymentInfo(transfer.sourceAccountHex)
        const sourceExecutionFee = paymentInfo["partialFee"].toBigInt()

        // The tx fee may be paid in a non-HDX currency.
        const feeCurrencyRaw = (
            await hydration.query.multiTransactionPayment.accountCurrencyMap(
                transfer.sourceAccountHex,
            )
        ).toPrimitive()
        const feeCurrency =
            feeCurrencyRaw === null || feeCurrencyRaw === undefined ? 0 : Number(feeCurrencyRaw)
        const txFee =
            feeCurrency === HYDRATION_NATIVE_ASSET_ID
                ? sourceExecutionFee
                : await hydrationFeeInCurrency(hydration, feeCurrency, sourceExecutionFee)

        const amountIn =
            transfer.amountIn + (feeCurrency === transfer.from.hydrationAssetId ? txFee : 0n)
        if (amountIn > balances.hydration[transfer.from.symbol]) {
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.InsufficientTokenBalance,
                message: `Insufficient ${transfer.from.symbol} balance on Hydration.`,
            })
        }
        const requiredDot =
            (findTotalOrUndefined(transfer.fee, "DOT") ?? 0n) +
            (feeCurrency === HYDRATION_DOT_ASSET_ID ? txFee : 0n)
        if (requiredDot > balances.hydrationDot) {
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.InsufficientDotFee,
                message: "Insufficient DOT on Hydration to pay the bridge fee.",
            })
        }
        if (feeCurrency === HYDRATION_NATIVE_ASSET_ID) {
            if (txFee > balances.hydrationNative) {
                logs.push({
                    kind: ValidationKind.Error,
                    reason: ValidationReason.InsufficientNativeFee,
                    message: "Insufficient HDX on Hydration to pay the transaction fee.",
                })
            }
        } else if (
            feeCurrency !== transfer.from.hydrationAssetId &&
            feeCurrency !== HYDRATION_DOT_ASSET_ID
        ) {
            const feeBalance = balanceOf(
                await hydration.call.currenciesApi.account(feeCurrency, transfer.sourceAccountHex),
            )
            if (txFee > feeBalance) {
                logs.push({
                    kind: ValidationKind.Error,
                    reason: ValidationReason.InsufficientNativeFee,
                    message: `Insufficient balance of fee currency ${feeCurrency} on Hydration to pay the transaction fee.`,
                })
            }
        }

        // The Ethereum stable only exists after the swap, so dry run the whole batch.
        let sourceDryRunError
        let assetHubDryRunError
        let bridgeHubDryRunError
        let ethereumDryRunError: string | undefined
        const dryRunSource = await dryRunOnSourceParachain(
            hydration,
            registry.assetHubParaId,
            registry.bridgeHubParaId,
            transfer.tx,
            transfer.sourceAccountHex,
        )
        if (!dryRunSource.success || !dryRunSource.assetHubForwarded) {
            sourceDryRunError = dryRunSource.error
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.DryRunFailed,
                message: "Dry run on Hydration failed.",
            })
        } else {
            const dryRunAH = await dryRunAssetHub(
                assetHub,
                HYDRATION_PARA_ID,
                registry.bridgeHubParaId,
                dryRunSource.assetHubForwarded[1][0],
            )
            if (!dryRunAH.success || !dryRunAH.bridgeHubForwarded) {
                assetHubDryRunError = dryRunAH.errorMessage
                logs.push({
                    kind: ValidationKind.Error,
                    reason: ValidationReason.DryRunFailed,
                    message: "Dry run on Asset Hub failed.",
                })
            } else {
                const dryRunBH = await dryRunBridgeHub(
                    bridgeHub,
                    registry.assetHubParaId,
                    dryRunAH.bridgeHubForwarded[1][0],
                )
                if (!dryRunBH.success) {
                    bridgeHubDryRunError = dryRunBH.errorMessage
                    logs.push({
                        kind: ValidationKind.Error,
                        reason: ValidationReason.DryRunFailed,
                        message: "Dry run on Bridge Hub failed.",
                    })
                } else {
                    const ethResult = await runEthereumDryRun(
                        this.context,
                        HYDRATION_PARA_ID,
                        transfer.sourceAccountHex,
                        transfer.bridge,
                        logs,
                    )
                    ethereumDryRunError = ethResult.ethereumDryRunError
                }
            }
        }

        const bridgeStatus = await getOperatingStatus({
            ethereumProvider: this.context.ethereumProvider,
            gateway: this.context.gateway(),
            bridgeHub,
        })
        if (bridgeStatus.toEthereum.outbound !== "Normal") {
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.BridgeStatusNotOperational,
                message: "Bridge operations have been paused by onchain governance.",
            })
        }

        return {
            ...transfer,
            success: logs.find((l) => l.kind === ValidationKind.Error) === undefined,
            logs,
            data: {
                sourceExecutionFee,
                sourceDryRunError,
                assetHubDryRunError,
                bridgeHubDryRunError,
                ethereumDryRunError,
            },
        }
    }

    async signAndSendSwapAndBridge(
        transfer: SwapAndBridgeTransfer,
        account: AddressOrPair,
        options: Partial<SignerOptions>,
    ): Promise<SubmitReceipt> {
        const hydration = await this.context.parachain(HYDRATION_PARA_ID)
        const receipt = await signAndSend(hydration, transfer.tx, account, options)
        receipt.messageId = transfer.messageId
        return receipt
    }
}

// Success means no ExtrinsicFailed, as Hydration emits no polkadotXcm.Sent.
async function signAndSend(
    api: ApiPromise,
    tx: SubmittableExtrinsic<"promise", ISubmittableResult>,
    account: AddressOrPair,
    options: Partial<SignerOptions>,
): Promise<SubmitReceipt> {
    return new Promise<SubmitReceipt>((resolve, reject) => {
        tx.signAndSend(account, options, (c) => {
            if (c.isError) {
                reject(c.internalError || c.dispatchError || c)
                return
            }
            if (!c.isInBlock && !c.isFinalized) return
            const failed = c.events.find((e) => api.events.system.ExtrinsicFailed.is(e.event))
            resolve({
                txHash: u8aToHex(c.txHash),
                txIndex: c.txIndex || 0,
                blockNumber: Number((c as any).blockNumber),
                blockHash: c.isInBlock ? c.status.asInBlock.toHex() : c.status.asFinalized.toHex(),
                success: failed === undefined,
                dispatchError: failed
                    ? (failed.event.data.toHuman(true) as any)?.dispatchError
                    : undefined,
                events: c.events,
            })
        }).catch(reject)
    })
}

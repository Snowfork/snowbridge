import { ApiPromise, SubmittableResult } from "@polkadot/api"
import { AddressOrPair, SignerOptions, SubmittableExtrinsic } from "@polkadot/api/types"
import { ISubmittableResult } from "@polkadot/types/types"
import { Result } from "@polkadot/types"
import { CallDryRunEffects, EventRecord, XcmDryRunApiError } from "@polkadot/types/interfaces"
import { isHex, stringToU8a, u8aConcat, u8aToHex } from "@polkadot/util"
import { blake2AsU8a, decodeAddress } from "@polkadot/util-crypto"
import { BridgeInfo, EthereumProviderTypes, TransferRoute } from "@snowbridge/base-types"
import { Context } from "."
import { DOT_LOCATION } from "./assets_v2"
import { runEthereumDryRun } from "./dryRunEthereum"
import { ACCOUNT_ID_32, resolveBeneficiary } from "./crypto"
import { findTotalOrUndefined } from "./fees"
import { getOperatingStatus } from "./status"
import { padFeeByPercentage } from "./utils"
import {
    dryRunAssetHub,
    dryRunOnSourceParachain,
    TransferToEthereum,
} from "./toEthereumSnowbridgeV2"
import { dryRunBridgeHub } from "./toEthereum_v2"
import type { DeliveryFee, Transfer, ValidationLog } from "./types/toEthereum"
import { ValidationKind, ValidationReason } from "./types/toEthereum"
import type { VolumeFeeParams } from "./feeSchedule"
import type { ServiceFee } from "./types/fee"
import type { ParachainBase } from "./parachains/parachainBase"

// Stables moved from Asset Hub to Hydration, swapped, then bridged to Ethereum.
export const HYDRATION_PARA_ID = 2034
const ASSET_HUB_PARA_ID = 1000
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
    // Pool account the quote is dry-run from when the user holds none. It may be a
    // route pool: stableswap prices from reserves read before the self-transfer.
    quotePoolId: number
}

export type EthereumStable = {
    symbol: EthereumStableSymbol
    token: string
    hydrationAssetId: number
    decimals: number
}

export type Trade = { pool: any; assetIn: number; assetOut: number }

// Hydration asset and pool ids. Asset ids are keys of `assetRegistry.assets` on
// Hydration; stableswap pool ids are keys of `stableswap.pools`, with their assets
// listed in the value. Both are browsable on polkadot.js apps (chain state) or Subscan.
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
// Every route ends in pool 103, which pairs the Ethereum stables with aUSDT, so each
// source is first turned into aUSDT. `{ Aave: null }` is not a liquidity pool: it
// supplies USDT to Hydration's Aave money market and receives aUSDT 1:1.
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
    // DOT sent alongside, for step 2 fees on Hydration.
    dotTopUp: bigint
    serviceFee?: ServiceFee
    tx: SubmittableExtrinsic<"promise", ISubmittableResult>
}

// Step 1 fees. The DOT fees are paid on Asset Hub, Hydration execution in the stable.
// Parts that need a successful Asset Hub dry run are undefined when it fails.
export type MoveToHydrationFees = {
    assetHubExecution: bigint
    assetHubDelivery?: bigint
    // Hydration execution of the stable transfer, in the stable.
    hydrationExecution?: bigint
    // Hydration execution of the DOT top-up, in DOT.
    hydrationDotExecution?: bigint
}

// Step 1 service fee inputs; the fee is paid in DOT on Asset Hub.
export type MoveToHydrationFeeParams = {
    dotToUsdNumerator: bigint
    dotToUsdDenominator: bigint
    serviceFeeRecipient: string
}

// Step 1 charges a fixed service fee; step 2 charges the full volume fee.
export const MOVE_TO_HYDRATION_FEE_USD = 1n

export type ValidatedMoveToHydration = MoveToHydrationTransfer & {
    success: boolean
    logs: ValidationLog[]
    data: {
        sourceExecutionFee: bigint
        fees: MoveToHydrationFees
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
    amountOut: bigint
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
        // The Hydration tx fee in the account's fee currency; amount is unset when
        // that currency cannot be priced.
        txFee: { amount?: bigint; assetId: number; symbol: string; decimals: number }
        sourceDryRunError?: any
        assetHubDryRunError?: any
        bridgeHubDryRunError?: any
        ethereumDryRunError?: any
    }
}

export type SubmitReceipt = {
    txHash: string
    txIndex: number
    blockNumber?: number
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

// What Hydration charges, in the stable, to execute the XCM from Asset Hub.
// Asset Hub assets are addressed from Hydration through Asset Hub; HOLLAR's
// Asset Hub location is already global.
function hydrationFeeLocation(stable: HydrationStable): any {
    return stable.assetHubAssetId !== undefined
        ? {
              parents: 1,
              interior: {
                  X3: [
                      { Parachain: ASSET_HUB_PARA_ID },
                      { PalletInstance: ASSET_HUB_ASSETS_PALLET },
                      { GeneralIndex: stable.assetHubAssetId },
                  ],
              },
          }
        : stable.locationOnAssetHub
}

const DOT_ON_HYDRATION = { parents: 1, interior: { Here: null } }

// The DOT top-up as Hydration receives it, for weighing. Weight does not depend on amounts.
function dotTopUpXcm(beneficiaryHex: string): any {
    const dot = { id: DOT_ON_HYDRATION, fun: { Fungible: 1n } }
    return {
        V4: [
            { ReserveAssetDeposited: [dot] },
            { ClearOrigin: null },
            { BuyExecution: { fees: dot, weightLimit: "Unlimited" } },
            {
                DepositAsset: {
                    assets: { Wild: { AllCounted: 1 } },
                    beneficiary: {
                        parents: 0,
                        interior: { X1: [{ AccountId32: { network: null, id: beneficiaryHex } }] },
                    },
                },
            },
            { SetTopic: "0x" + "00".repeat(32) },
        ],
    }
}

// What Hydration charges, in the asset at `location`, to execute an XCM.
async function hydrationXcmExecutionFee(
    hydration: ApiPromise,
    xcm: any,
    location: any,
): Promise<bigint> {
    const weight = await hydration.call.xcmPaymentApi.queryXcmWeight(
        hydration.createType("XcmVersionedXcm", xcm),
    )
    const fee = await hydration.call.xcmPaymentApi.queryWeightToAssetFee((weight as any).asOk, {
        V4: location,
    })
    return BigInt((fee as any).asOk.toString())
}

// Dry runs `call` on XCM v5, falling back for runtimes whose dryRunCall takes no version.
async function dryRunCall(
    api: ApiPromise,
    origin: any,
    call: any,
): Promise<Result<CallDryRunEffects, XcmDryRunApiError>> {
    try {
        return await api.call.dryRunApi.dryRunCall<Result<CallDryRunEffects, XcmDryRunApiError>>(
            origin,
            call,
            5,
        )
    } catch {
        return await api.call.dryRunApi.dryRunCall<Result<CallDryRunEffects, XcmDryRunApiError>>(
            origin,
            call,
        )
    }
}

// Dry runs `tx` on Asset Hub and returns every message it forwards to Hydration,
// in batch order.
async function dryRunAssetHubToHydration(
    assetHub: ApiPromise,
    tx: SubmittableExtrinsic<"promise", ISubmittableResult>,
    account: string,
): Promise<{ success: boolean; error?: any; messages: any[] }> {
    const result = await dryRunCall(assetHub, { system: { signed: account } }, tx)
    if (result.isErr) {
        return { success: false, error: result.asErr.toJSON(), messages: [] }
    }
    if (result.asOk.executionResult.isErr) {
        return { success: false, error: result.asOk.executionResult.asErr.toJSON(), messages: [] }
    }
    const entry = result.asOk.forwardedXcms.find(([dest]) => {
        const loc = dest.isV5 ? dest.asV5 : dest.isV4 ? dest.asV4 : undefined
        return (
            loc !== undefined &&
            loc.parents.toNumber() === 1 &&
            loc.interior.isX1 &&
            loc.interior.asX1[0].isParachain &&
            loc.interior.asX1[0].asParachain.toNumber() === HYDRATION_PARA_ID
        )
    })
    const messages: any[] = entry ? (entry.toPrimitive() as any[])[1] : []
    return { success: messages.length > 0, messages }
}

// Pads the fallback price to cover drift from the oracle price.
const FEE_CURRENCY_PRICE_PAD = 2n

// Undefined for a currency without a fallback price: Hydration prices it from the
// oracle, or swaps it to DOT when the asset is insufficient.
async function hydrationFeeInCurrency(
    hydration: ApiPromise,
    currency: number,
    hdxFee: bigint,
): Promise<bigint | undefined> {
    const price = (
        await hydration.query.multiTransactionPayment.acceptedCurrencies(currency)
    ).toPrimitive()
    if (price === null || price === undefined) return undefined
    return (hdxFee * BigInt(price.toString()) * FEE_CURRENCY_PRICE_PAD) / 10n ** 18n
}

async function hydrationExistentialDeposit(
    hydration: ApiPromise,
    assetId: number,
): Promise<bigint> {
    const asset = (await hydration.query.assetRegistry.assets(assetId)).toPrimitive() as any
    return BigInt(asset?.existentialDeposit ?? 0)
}

// What an account can spend and stay alive. Frozen funds and the existential deposit
// overlap, so only the larger of the two is held back.
async function spendableKeepAlive(
    impl: ParachainBase,
    account: string,
    existentialDeposit: bigint,
): Promise<bigint> {
    const { free, frozen, reserved } = (await impl.getNativeAccount(account)).data
    const locked = frozen > reserved ? frozen - reserved : 0n
    const held = locked > existentialDeposit ? locked : existentialDeposit
    return free > held ? free - held : 0n
}

// A remainder that is neither zero nor at least `minimum` is reaped as dust.
function leavesDust(balance: bigint, spent: bigint, minimum: bigint): boolean {
    const remaining = balance - spent
    return remaining > 0n && remaining < minimum
}

function formatAmount(value: bigint, decimals: number): string {
    const base = 10n ** BigInt(decimals)
    const fraction = (value % base).toString().padStart(decimals, "0").replace(/0+$/, "")
    return fraction ? `${value / base}.${fraction}` : `${value / base}`
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
        // Asset, pool and token ids are Polkadot mainnet values.
        if (info.environment.name !== "polkadot_mainnet") {
            throw Error(
                `Stables are only supported on polkadot_mainnet, not ${info.environment.name}.`,
            )
        }
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

    // Leg 1: Asset Hub -> Hydration. Each asset pays its own Hydration execution.
    // HOLLAR's reserve is Hydration and DOT's is Asset Hub, so they cannot share a
    // transferAssets call; the stable, the DOT top-up and the service fee are batched.
    async moveToHydrationTx(
        sourceAccount: string,
        symbol: HydrationStableSymbol,
        amount: bigint,
        options?: { volumeFee?: MoveToHydrationFeeParams; dotTopUp?: bigint },
    ): Promise<MoveToHydrationTransfer> {
        const stable = HYDRATION_STABLES[symbol]
        const dotTopUp = options?.dotTopUp ?? 0n
        if (amount === 0n && dotTopUp === 0n) {
            throw new Error("Nothing to send: set an amount or DOT to send.")
        }
        const assetHub = await this.context.assetHub()
        const sourceAccountHex = toHexAccount(sourceAccount)
        const serviceFee = await this.moveToHydrationServiceFee(options?.volumeFee)
        const transferAssets = (id: any, value: bigint) =>
            assetHub.tx.polkadotXcm.transferAssets(
                { v4: { parents: 1, interior: { X1: [{ Parachain: HYDRATION_PARA_ID }] } } },
                {
                    v4: {
                        parents: 0,
                        interior: {
                            X1: [{ AccountId32: { network: null, id: sourceAccountHex } }],
                        },
                    },
                },
                { v4: [{ id, fun: { Fungible: value } }] },
                0,
                "Unlimited",
            )
        // pallet_xcm cannot infer DOT's reserve for Hydration, so name it: Asset Hub
        // is the reserve, and the DOT pays its own execution there.
        const dotTransfer = (value: bigint) =>
            assetHub.tx.polkadotXcm.transferAssetsUsingTypeAndThen(
                { v4: { parents: 1, interior: { X1: [{ Parachain: HYDRATION_PARA_ID }] } } },
                { v4: [{ id: DOT_LOCATION, fun: { Fungible: value } }] },
                "LocalReserve",
                { v4: DOT_LOCATION },
                "LocalReserve",
                {
                    v4: [
                        {
                            depositAsset: {
                                assets: { Wild: { AllCounted: 1 } },
                                beneficiary: {
                                    parents: 0,
                                    interior: { X1: [{ AccountId32: { id: sourceAccountHex } }] },
                                },
                            },
                        },
                    ],
                },
                "Unlimited",
            )
        const calls = [
            ...(amount > 0n ? [transferAssets(stable.locationOnAssetHub, amount)] : []),
            ...(dotTopUp > 0n ? [dotTransfer(dotTopUp)] : []),
            ...(serviceFee
                ? [assetHub.tx.balances.transferKeepAlive(serviceFee.recipient, serviceFee.amount)]
                : []),
        ]
        const tx = calls.length === 1 ? calls[0] : assetHub.tx.utility.batchAll(calls)
        return {
            kind: "stables:assethub->hydration",
            stable,
            sourceAccount,
            sourceAccountHex,
            amount,
            dotTopUp,
            serviceFee,
            tx,
        }
    }

    // Step 1 service fee: MOVE_TO_HYDRATION_FEE_USD in DOT, floored at the Asset Hub
    // existential deposit so the transfer to an empty recipient cannot fail.
    async moveToHydrationServiceFee(
        params?: MoveToHydrationFeeParams,
    ): Promise<ServiceFee | undefined> {
        if (!params) return undefined
        const { hexAddress, kind } = resolveBeneficiary(params.serviceFeeRecipient)
        if (kind !== ACCOUNT_ID_32) {
            throw new Error("Service fee recipient must be a 32-byte Asset Hub account.")
        }
        if (params.dotToUsdNumerator <= 0n || params.dotToUsdDenominator <= 0n) {
            throw new Error("DOT to USD rate must be positive.")
        }
        const fee =
            (MOVE_TO_HYDRATION_FEE_USD * 10n ** 10n * params.dotToUsdDenominator) /
            params.dotToUsdNumerator
        const assetHub = await this.context.assetHub()
        const ed = BigInt(assetHub.consts.balances.existentialDeposit.toString())
        return { recipient: hexAddress, amount: fee > ed ? fee : ed }
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
        const [balances, { fees, dryRun }] = await Promise.all([
            this.balances(transfer.sourceAccountHex),
            this.#moveToHydrationCosts(transfer),
        ])
        const sourceExecutionFee = fees.assetHubExecution

        const assetInfo = (
            stable.assetHubAssetId !== undefined
                ? await assetHub.query.assets.asset(stable.assetHubAssetId)
                : await assetHub.query.foreignAssets.asset(stable.locationOnAssetHub)
        ).toPrimitive() as any
        const minBalance = BigInt(assetInfo?.minBalance ?? 0)
        const { decimals, symbol } = stable
        const sendsStable = transfer.amount > 0n
        if (sendsStable && transfer.amount < minBalance) {
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.InsufficientTokenBalance,
                message: `Amount is below the ${symbol} minimum balance of ${formatAmount(minBalance, decimals)}.`,
            })
        }
        if (transfer.amount > balances.assetHub[symbol]) {
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.InsufficientTokenBalance,
                message: `Insufficient ${symbol} balance on Asset Hub.`,
            })
        } else if (
            sendsStable &&
            leavesDust(balances.assetHub[symbol], transfer.amount, minBalance)
        ) {
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.InsufficientTokenBalance,
                message: `This leaves less than the ${formatAmount(minBalance, decimals)} ${symbol} minimum on Asset Hub. Send the full balance or leave at least the minimum.`,
            })
        }
        // Hydration execution is paid in the stable, so an account that holds none there
        // needs more than the existential deposit plus that fee.
        const hydrationEd = await hydrationExistentialDeposit(hydration, stable.hydrationAssetId)
        const stableMinimum = hydrationEd + (fees.hydrationExecution ?? 0n)
        if (sendsStable && balances.hydration[symbol] === 0n && transfer.amount <= stableMinimum) {
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.InsufficientTokenBalance,
                message: `Send more than ${formatAmount(stableMinimum, decimals)} ${symbol}, the Hydration existential deposit plus execution fee.`,
            })
        }

        // The top-up pays its own Hydration execution in DOT, so an account that holds
        // none there needs more than the existential deposit plus that fee.
        if (transfer.dotTopUp > 0n && balances.hydrationDot === 0n) {
            const dotMinimum =
                (await hydrationExistentialDeposit(hydration, HYDRATION_DOT_ASSET_ID)) +
                (fees.hydrationDotExecution ?? 0n)
            if (transfer.dotTopUp <= dotMinimum) {
                logs.push({
                    kind: ValidationKind.Error,
                    reason: ValidationReason.InsufficientDotFee,
                    message: `Send more than ${formatAmount(dotMinimum, 10)} DOT, the Hydration existential deposit plus execution fee.`,
                })
            }
        }

        // DOT must cover the top-up, tx fee, service fee and delivery of every
        // message, and keep the Asset Hub existential deposit.
        const dotEd = BigInt(assetHub.consts.balances.existentialDeposit.toString())
        const requiredDot =
            sourceExecutionFee +
            (transfer.serviceFee?.amount ?? 0n) +
            (fees.assetHubDelivery ?? 0n) +
            transfer.dotTopUp
        const spendableDot = await spendableKeepAlive(
            await this.context.paraImplementation(assetHub),
            transfer.sourceAccountHex,
            dotEd,
        )
        if (requiredDot > spendableDot) {
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.InsufficientNativeFee,
                message: `Insufficient DOT on Asset Hub: ${
                    transfer.dotTopUp > 0n ? "the DOT and fees" : "fees"
                } need ${formatAmount(requiredDot, 10)} DOT, and ${formatAmount(dotEd, 10)} DOT or any locked DOT must stay in the account.`,
            })
        }

        let assetHubDryRunError
        let hydrationDryRunError
        if (!dryRun.success) {
            assetHubDryRunError = dryRun.error
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.DryRunFailed,
                message: "Dry run on Asset Hub failed.",
            })
        } else {
            // Every message the batch forwards must execute on Hydration.
            const hydrationImpl = await this.context.paraImplementation(hydration)
            for (const message of dryRun.messages) {
                const dryRunDest = await hydrationImpl.dryRunXcm(registry.assetHubParaId, message)
                if (!dryRunDest.success) {
                    hydrationDryRunError = dryRunDest.errorMessage
                    logs.push({
                        kind: ValidationKind.Error,
                        reason: ValidationReason.DryRunFailed,
                        message: "Dry run on Hydration failed.",
                    })
                    break
                }
            }
        }

        return {
            ...transfer,
            success: logs.find((l) => l.kind === ValidationKind.Error) === undefined,
            logs,
            data: { sourceExecutionFee, fees, assetHubDryRunError, hydrationDryRunError },
        }
    }

    async moveToHydrationFees(transfer: MoveToHydrationTransfer): Promise<MoveToHydrationFees> {
        return (await this.#moveToHydrationCosts(transfer)).fees
    }

    // Step 1 fees and the Asset Hub dry run they come from.
    async #moveToHydrationCosts(transfer: MoveToHydrationTransfer): Promise<{
        fees: MoveToHydrationFees
        dryRun: Awaited<ReturnType<typeof dryRunAssetHubToHydration>>
    }> {
        const [assetHub, hydration] = await Promise.all([
            this.context.assetHub(),
            this.context.parachain(HYDRATION_PARA_ID),
        ])
        const [paymentInfo, dryRun] = await Promise.all([
            transfer.tx.paymentInfo(transfer.sourceAccountHex),
            dryRunAssetHubToHydration(assetHub, transfer.tx, transfer.sourceAccountHex),
        ])
        const assetHubExecution = paymentInfo["partialFee"].toBigInt()
        if (!dryRun.success) return { fees: { assetHubExecution }, dryRun }
        const assetHubImpl = await this.context.paraImplementation(assetHub)
        // Messages follow batch order: the stable first, then the DOT top-up.
        const [stableMessage, dotMessage] =
            transfer.amount > 0n
                ? [dryRun.messages[0], dryRun.messages[1]]
                : [undefined, dryRun.messages[0]]
        const deliveries = await Promise.all(
            dryRun.messages.map((m) =>
                assetHubImpl.calculateDeliveryFeeInDOT(HYDRATION_PARA_ID, m),
            ),
        )
        const [hydrationExecution, hydrationDotExecution] = await Promise.all([
            stableMessage &&
                hydrationXcmExecutionFee(
                    hydration,
                    stableMessage,
                    hydrationFeeLocation(transfer.stable),
                ),
            dotMessage && hydrationXcmExecutionFee(hydration, dotMessage, DOT_ON_HYDRATION),
        ])
        return {
            fees: {
                assetHubExecution,
                assetHubDelivery: deliveries.reduce((a, b) => a + b, 0n),
                hydrationExecution: hydrationExecution || undefined,
                hydrationDotExecution: hydrationDotExecution || undefined,
            },
            dryRun,
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
            const result = await dryRunCall(hydration, { system: { signed: who } }, sell)
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
            accelerated?: boolean
        },
    ): Promise<DeliveryFee> {
        return this.#bridgeSender().fee(ETHEREUM_STABLES[toSymbol].token, {
            feeTokenLocation: DOT_LOCATION,
            padFeeByPercentage: options?.padFeeByPercentage,
            slippagePadPercentage: options?.slippagePadPercentage ?? 20n,
            volumeFee: options?.volumeFee,
            accelerated: options?.accelerated,
        })
    }

    // DOT to send in leg 1 so leg 2 can pay `fee` on Hydration: the padded bridge fee,
    // `txFee` when the account pays Hydration fees in DOT, and the Hydration DOT
    // existential deposit, less what the account already holds, plus the top-up's own
    // Hydration execution. `txFee` is an estimate the caller supplies, as leg 2's tx
    // does not exist yet.
    async dotTopUp(
        account: string,
        fee: DeliveryFee,
        options?: { padPercentage?: bigint; txFee?: bigint },
    ): Promise<bigint> {
        const accountHex = toHexAccount(account)
        const hydration = await this.context.parachain(HYDRATION_PARA_ID)
        const [held, dotEd, execution, feeCurrency] = await Promise.all([
            hydration.query.tokens.accounts(accountHex, HYDRATION_DOT_ASSET_ID).then(balanceOf),
            hydrationExistentialDeposit(hydration, HYDRATION_DOT_ASSET_ID),
            hydrationXcmExecutionFee(hydration, dotTopUpXcm(accountHex), DOT_ON_HYDRATION),
            hydration.query.multiTransactionPayment
                .accountCurrencyMap(accountHex)
                .then((c) => c.toPrimitive()),
        ])
        const bridgeDot = padFeeByPercentage(
            findTotalOrUndefined(fee, "DOT") ?? 0n,
            options?.padPercentage ?? 10n,
        )
        const txFeeInDot =
            Number(feeCurrency ?? HYDRATION_NATIVE_ASSET_ID) === HYDRATION_DOT_ASSET_ID
                ? (options?.txFee ?? 0n)
                : 0n
        const needed = bridgeDot + txFeeInDot + dotEd
        return needed > held ? needed - held + execution : 0n
    }

    // Leg 2: swap and bridge `minAmountOut` in one batch; any surplus stays on Hydration.
    async swapAndBridgeTx(
        sourceAccount: string,
        beneficiaryAccount: string,
        quote: SwapQuote,
        slippageBps: bigint,
        fee: DeliveryFee,
    ): Promise<SwapAndBridgeTransfer> {
        if (slippageBps < 0n || slippageBps >= 10_000n) {
            throw Error(`slippageBps ${slippageBps} not in range of 0 to 9999.`)
        }
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
            amountOut,
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

        const feeAssetInfo = (
            await hydration.query.assetRegistry.assets(feeCurrency)
        ).toPrimitive() as any
        const feeAsset = {
            symbol: String(feeAssetInfo?.symbol ?? `asset ${feeCurrency}`),
            decimals: Number(feeAssetInfo?.decimals ?? 12),
        }
        const { from, to } = transfer
        const [fromEd, toEd, dotEd, hdxEd] = await Promise.all([
            hydrationExistentialDeposit(hydration, from.hydrationAssetId),
            hydrationExistentialDeposit(hydration, to.hydrationAssetId),
            hydrationExistentialDeposit(hydration, HYDRATION_DOT_ASSET_ID),
            hydrationExistentialDeposit(hydration, HYDRATION_NATIVE_ASSET_ID),
        ])

        // The source stable and DOT always have a fallback price, so txFee is set for them.
        const amountIn =
            transfer.amountIn + (feeCurrency === from.hydrationAssetId ? (txFee ?? 0n) : 0n)
        if (amountIn > balances.hydration[from.symbol]) {
            // The amount alone may fit; the fee paid in the same stable pushes it over.
            const feeInSource =
                feeCurrency === from.hydrationAssetId &&
                transfer.amountIn <= balances.hydration[from.symbol]
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.InsufficientTokenBalance,
                message: feeInSource
                    ? `The transaction fee (about ${formatAmount(txFee ?? 0n, from.decimals)} ${from.symbol}) is paid in ${from.symbol}, so lower the amount to leave that on Hydration.`
                    : `Insufficient ${from.symbol} balance on Hydration.`,
            })
        } else if (leavesDust(balances.hydration[from.symbol], amountIn, fromEd)) {
            // With the fee paid in the source stable, the full balance cannot be swapped.
            const advice =
                feeCurrency === from.hydrationAssetId
                    ? `The transaction fee is paid in ${from.symbol}, so lower the amount to leave at least that.`
                    : "Swap the full balance or leave at least that."
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.InsufficientTokenBalance,
                message: `This leaves less than the ${formatAmount(fromEd, from.decimals)} ${from.symbol} existential deposit on Hydration. ${advice}`,
            })
        }
        const requiredDot =
            (findTotalOrUndefined(transfer.fee, "DOT") ?? 0n) +
            (feeCurrency === HYDRATION_DOT_ASSET_ID ? (txFee ?? 0n) : 0n)
        if (requiredDot > balances.hydrationDot) {
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.InsufficientDotFee,
                message: "Insufficient DOT on Hydration to pay the bridge fee.",
            })
        } else if (leavesDust(balances.hydrationDot, requiredDot, dotEd)) {
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.InsufficientDotFee,
                message: `The DOT fees leave less than the ${formatAmount(dotEd, 10)} DOT existential deposit on Hydration.`,
            })
        }
        // Surplus above minAmountOut stays on Hydration; below the existential
        // deposit it is reaped.
        const toBalance = balanceOf(
            await hydration.call.currenciesApi.account(
                to.hydrationAssetId,
                transfer.sourceAccountHex,
            ),
        )
        if (leavesDust(toBalance + transfer.amountOut, transfer.minAmountOut, toEd)) {
            logs.push({
                kind: ValidationKind.Warning,
                reason: ValidationReason.InsufficientTokenBalance,
                message: `Swap surplus under ${formatAmount(toEd, to.decimals)} ${to.symbol} would be lost as dust on Hydration.`,
            })
        }
        if (feeCurrency === HYDRATION_NATIVE_ASSET_ID) {
            const spendableHdx = await spendableKeepAlive(
                await this.context.paraImplementation(hydration),
                transfer.sourceAccountHex,
                hdxEd,
            )
            if (sourceExecutionFee > spendableHdx) {
                logs.push({
                    kind: ValidationKind.Error,
                    reason: ValidationReason.InsufficientNativeFee,
                    message: `Insufficient HDX on Hydration for the transaction fee; ${formatAmount(hdxEd, 12)} HDX or any locked HDX must stay in the account.`,
                })
            }
        } else if (
            feeCurrency !== transfer.from.hydrationAssetId &&
            feeCurrency !== HYDRATION_DOT_ASSET_ID
        ) {
            // The batch does not spend this currency, so an unpriced fee cannot break it:
            // too low a balance makes Hydration reject the transaction before inclusion.
            if (txFee === undefined) {
                logs.push({
                    kind: ValidationKind.Warning,
                    reason: ValidationReason.InsufficientNativeFee,
                    message: `The Hydration transaction fee is paid in ${feeAsset.symbol}, which could not be priced. If your ${feeAsset.symbol} balance is too low, the transaction is rejected and nothing is sent.`,
                })
            } else {
                const [feeBalance, feeEd] = await Promise.all([
                    hydration.call.currenciesApi
                        .account(feeCurrency, transfer.sourceAccountHex)
                        .then(balanceOf),
                    hydrationExistentialDeposit(hydration, feeCurrency),
                ])
                if (txFee > feeBalance) {
                    logs.push({
                        kind: ValidationKind.Error,
                        reason: ValidationReason.InsufficientNativeFee,
                        message: `Insufficient ${feeAsset.symbol} on Hydration to pay the transaction fee.`,
                    })
                } else if (leavesDust(feeBalance, txFee, feeEd)) {
                    // Fees are withdrawn allowing death, so a remainder under the
                    // existential deposit is reaped.
                    logs.push({
                        kind: ValidationKind.Warning,
                        reason: ValidationReason.InsufficientNativeFee,
                        message: `The transaction fee may leave less than the ${formatAmount(feeEd, feeAsset.decimals)} ${feeAsset.symbol} existential deposit on Hydration, which would be lost.`,
                    })
                }
            }
        }

        // The Ethereum stable only exists after the swap, so dry run the whole batch.
        let sourceDryRunError
        let assetHubDryRunError
        let bridgeHubDryRunError
        let ethereumDryRunError: string | undefined
        const dryRunSource =
            (txFee !== undefined
                ? await dryRunWithTxFee(
                      hydration,
                      registry.assetHubParaId,
                      transfer.sourceAccountHex,
                      transfer.tx,
                      feeCurrency,
                      txFee,
                  )
                : undefined) ??
            (await dryRunOnSourceParachain(
                hydration,
                registry.assetHubParaId,
                registry.bridgeHubParaId,
                transfer.tx,
                transfer.sourceAccountHex,
            ))
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
                txFee: { amount: txFee, assetId: feeCurrency, ...feeAsset },
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

// Dry runs `tx` after charging its fee: as Root, deduct the fee, then dispatch `tx`
// as the account. Returns undefined when the fee currency cannot be debited that way.
async function dryRunWithTxFee(
    hydration: ApiPromise,
    assetHubParaId: number,
    account: string,
    tx: SubmittableExtrinsic<"promise", ISubmittableResult>,
    feeCurrency: number,
    txFee: bigint,
): Promise<{ success: boolean; error?: any; assetHubForwarded?: any } | undefined> {
    const call = hydration.tx.utility.batchAll([
        hydration.tx.currencies.updateBalance(account, feeCurrency, -txFee),
        hydration.tx.utility.dispatchAs({ system: { Signed: account } }, tx),
    ])
    const result = await dryRunCall(hydration, { system: "Root" }, call.inner.toHex())
    if (result.isErr) {
        return { success: false, error: result.asErr.toJSON() }
    }
    const effects = result.asOk
    if (effects.executionResult.isErr) {
        const err = (effects.executionResult.asErr as any).error
        if (err.isModule) {
            const meta = hydration.registry.findMetaError(err.asModule)
            if (meta.section === "currencies" && meta.name === "NotSupported") return undefined
        }
        return { success: false, error: effects.executionResult.asErr.toJSON() }
    }
    // dispatchAs succeeds regardless; the batch's own result is in DispatchedAs.
    const dispatched = effects.emittedEvents.find(
        (e) => e.section === "utility" && e.method === "DispatchedAs",
    )
    const inner = dispatched?.data[0] as any
    if (!inner || inner.isErr) {
        return { success: false, error: inner?.asErr.toJSON() }
    }
    const assetHubForwarded = effects.forwardedXcms.find(([dest]) => {
        const loc = dest.isV5 ? dest.asV5 : dest.isV4 ? dest.asV4 : undefined
        return (
            loc !== undefined &&
            loc.parents.toNumber() === 1 &&
            loc.interior.isX1 &&
            loc.interior.asX1[0].isParachain &&
            loc.interior.asX1[0].asParachain.toNumber() === assetHubParaId
        )
    })
    return { success: assetHubForwarded !== undefined, assetHubForwarded }
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
                blockNumber: (c as SubmittableResult).blockNumber?.toNumber(),
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

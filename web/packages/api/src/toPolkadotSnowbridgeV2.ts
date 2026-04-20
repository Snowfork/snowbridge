import { TransferInterface } from "./transfers/toPolkadot/transferInterface"
import { TransferInterface as L2TransferInterface } from "./transfers/l2ToPolkadot/transferInterface"
import { ERC20ToAH } from "./transfers/toPolkadot/erc20ToAH"
import { ERC20ToAH as ERC20FromL2ToAH } from "./transfers/l2ToPolkadot/erc20ToAH"
import { RegisterToken } from "./registration/toPolkadot/registerToken"
import {
    AssetRegistry,
    ChainId,
    EthereumChain,
    EthereumProvider,
    EthereumProviderTypes,
    Parachain,
    TransferRoute,
} from "@snowbridge/base-types"
import { PNAToAH } from "./transfers/toPolkadot/pnaToAH"
import { ERC20ToParachain } from "./transfers/toPolkadot/erc20ToParachain"
import { PNAToParachain } from "./transfers/toPolkadot/pnaToParachain"
import { hexToU8a, stringToU8a } from "@polkadot/util"
import { blake2AsHex } from "@polkadot/util-crypto"
import { ApiPromise } from "@polkadot/api"
import { accountToLocationWithNetwork, erc20Location } from "./xcmBuilder"
import { Codec } from "@polkadot/types/types"
import { DOT_LOCATION, ETHER_TOKEN_ADDRESS } from "./assets_v2"
import { ensureValidationSuccess, padFeeByPercentage } from "./utils"
import { Context } from "./index"
export { ValidationKind } from "./types/toPolkadot"
import { ParachainBase } from "./parachains/parachainBase"
import type {
    DeliveryFee,
    MessageReceipt,
    Transfer,
    ValidatedTransfer,
} from "./types/toPolkadotSnowbridgeV2"
export type {
    DeliveryFee,
    MessageReceipt,
    Transfer,
    ValidatedTransfer,
} from "./types/toPolkadotSnowbridgeV2"

// Re-export registration types for convenience
export type {
    TokenRegistration,
    ValidatedRegisterToken,
    RegistrationFee,
    RegistrationInterface,
} from "./types/registration/toPolkadot"

export class TransferToPolkadot<T extends EthereumProviderTypes> implements TransferInterface<T> {
    #pnaImpl?: TransferInterface<T>
    #erc20Impl?: TransferInterface<T>

    constructor(
        public readonly context: Context<T>,
        private readonly route: TransferRoute,
        private readonly registry: AssetRegistry,
        private readonly source: EthereumChain,
        private readonly destination: Parachain,
    ) {}

    get from(): ChainId {
        return this.route.from
    }

    get to(): ChainId {
        return this.route.to
    }

    #resolveByTokenAddress(tokenAddress: string): TransferInterface<T> {
        const destinationParaId = this.route.to.id
        const ahAssetMetadata =
            this.registry.parachains[`polkadot_${this.registry.assetHubParaId}`].assets[
                tokenAddress.toLowerCase()
            ]
        if (!ahAssetMetadata) {
            throw Error(`Token ${tokenAddress} not registered on asset hub.`)
        }

        if (ahAssetMetadata.location) {
            this.#pnaImpl ??=
                destinationParaId == this.registry.assetHubParaId
                    ? new PNAToAH(
                          this.context,
                          this.registry,
                          this.route,
                          this.source,
                          this.destination,
                      )
                    : new PNAToParachain(
                          this.context,
                          this.registry,
                          this.route,
                          this.source,
                          this.destination,
                      )
            return this.#pnaImpl
        }

        this.#erc20Impl ??=
            destinationParaId == this.registry.assetHubParaId
                ? new ERC20ToAH(
                      this.context,
                      this.registry,
                      this.route,
                      this.source,
                      this.destination,
                  )
                : new ERC20ToParachain(
                      this.context,
                      this.registry,
                      this.route,
                      this.source,
                      this.destination,
                  )
        return this.#erc20Impl
    }

    async fee(
        tokenAddress: string,
        options?: {
            padFeeByPercentage?: bigint
            feeAsset?: any
            customXcm?: any[]
            overrideRelayerFee?: bigint
            volumeFee?: import("./feeSchedule").VolumeFeeParams
        },
    ): Promise<DeliveryFee> {
        return this.#resolveByTokenAddress(tokenAddress).fee(tokenAddress, options)
    }

    async tx(
        sourceAccount: string,
        beneficiaryAccount: string,
        tokenAddress: string,
        amount: bigint,
        fee: DeliveryFee,
        customXcm?: any[],
    ): Promise<Transfer<T>> {
        return this.#resolveByTokenAddress(tokenAddress).tx(
            sourceAccount,
            beneficiaryAccount,
            tokenAddress,
            amount,
            fee,
            customXcm,
        )
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
                volumeFee?: import("./feeSchedule").VolumeFeeParams
            }
            customXcm?: any[]
        },
    ): Promise<ValidatedTransfer<T>> {
        const fee = await this.fee(tokenAddress, options?.fee)
        const transfer = await this.tx(
            sourceAccount,
            beneficiaryAccount,
            tokenAddress,
            amount,
            fee,
            options?.customXcm,
        )
        return ensureValidationSuccess(await this.validate(transfer))
    }

    async validate(transfer: Transfer<T>): Promise<ValidatedTransfer<T>> {
        return this.#resolveByTokenAddress(transfer.input.tokenAddress).validate(transfer)
    }

    async messageId(receipt: T["TransactionReceipt"]): Promise<MessageReceipt | null> {
        return messageId(this.context.ethereumProvider, receipt)
    }
}

export function buildMessageId(
    destParaId: number,
    sourceAccountHex: string,
    tokenAddress: string,
    beneficiaryAccount: string,
    amount: bigint,
    accountNonce: number,
    timestamp?: number,
) {
    const entropy = new Uint8Array([
        ...stringToU8a(destParaId.toString()),
        ...hexToU8a(sourceAccountHex),
        ...hexToU8a(tokenAddress),
        ...stringToU8a(beneficiaryAccount),
        ...stringToU8a(amount.toString()),
        ...stringToU8a(accountNonce.toString()),
        ...stringToU8a((timestamp || Date.now()).toString()),
    ])
    return blake2AsHex(entropy)
}

export async function messageId<T extends EthereumProviderTypes>(
    ethereumProvider: EthereumProvider<T>,
    receipt: T["TransactionReceipt"],
): Promise<MessageReceipt | null> {
    const messageAccepted = ethereumProvider.scanGatewayV2OutboundMessageAccepted(receipt)
    if (!messageAccepted) return null
    return messageAccepted
}

export function claimerFromBeneficiary(
    assetHub: ApiPromise,
    beneficiaryAddressHex: string,
    envName: string,
) {
    // The `network` field MUST match what AH's signed-origin converter produces
    // (e.g. `Some(Polkadot)` on polkadot_mainnet) so a trapped-funds claim from
    // the beneficiary's signed origin hashes to the same key as the trap entry.
    const accountLocation = {
        parents: 0,
        interior: { x1: [accountToLocationWithNetwork(beneficiaryAddressHex, envName)] },
    }
    return assetHub.registry.createType("StagingXcmV5Location", accountLocation)
}

export function claimerLocationToBytes(claimerLocation: Codec) {
    return hexToU8a(claimerLocation.toHex())
}

export async function inboundMessageExtrinsicFee(
    assetHub: ParachainBase,
    ethChainId: number,
): Promise<{ extrinsicFeeDot: bigint; extrinsicFeeEther: bigint }> {
    // Hardcoded because the EthereumInboundQueueV2::submit() extrinsic
    // requires a valid proof to get an accurate weight. Sending an
    // invalid proof underestimates the cost by 80%. Constructing a proof is
    // complex and requires the message to be finalized, so not fit for purpose
    // here. Consequently, DOT fee is hardcoded for now.
    const extrinsicFeeDot = 250_000_000n

    const etherLocation = erc20Location(ethChainId, ETHER_TOKEN_ADDRESS)
    const extrinsicFeeEther = await assetHub.swapAsset1ForAsset2(
        DOT_LOCATION,
        etherLocation,
        extrinsicFeeDot,
    )

    return { extrinsicFeeDot, extrinsicFeeEther }
}

export async function calculateRelayerFee(
    assetHub: ParachainBase,
    ethChainId: number,
    overrideRelayerFee: undefined | bigint,
    deliveryFeeInEther: bigint,
): Promise<{ relayerFee: bigint; extrinsicFeeDot: bigint; extrinsicFeeEther: bigint }> {
    let relayerFee
    let extrinsicFeeDot = 0n
    let extrinsicFeeEther = 0n

    if (overrideRelayerFee !== undefined) {
        relayerFee = overrideRelayerFee
    } else {
        const extrinsicFees = await inboundMessageExtrinsicFee(assetHub, ethChainId)
        extrinsicFeeDot = extrinsicFees.extrinsicFeeDot
        extrinsicFeeEther = extrinsicFees.extrinsicFeeEther
        relayerFee = extrinsicFeeEther + deliveryFeeInEther
        relayerFee = padFeeByPercentage(relayerFee, 30n)
    }
    return { relayerFee, extrinsicFeeDot, extrinsicFeeEther }
}

export async function buildSwapCallData<T extends EthereumProviderTypes>(
    context: Context<T>,
    registry: AssetRegistry,
    l2ChainId: number,
    l2TokenAddress: string,
    amountOut: bigint,
    amountInMaximum: bigint,
): Promise<string> {
    let tokenIn =
        registry.ethereumChains?.[`ethereum_l2_${l2ChainId}`]?.assets[l2TokenAddress]
            ?.swapTokenAddress
    if (!tokenIn) {
        throw new Error("Token is not registered on Ethereum")
    }
    let swapFee =
        registry.ethereumChains?.[`ethereum_l2_${l2ChainId}`]?.assets[l2TokenAddress]?.swapFee
    const l1FeeTokenAddress = context.environment.l2Bridge?.l1FeeTokenAddress
    const l1HandlerAddress = context.environment.l2Bridge?.l1HandlerAddress
    if (!l1FeeTokenAddress || !l1HandlerAddress) {
        throw new Error("L2 bridge configuration is missing.")
    }
    let swapCalldata: string
    if (registry.environment === "polkadot_mainnet") {
        swapCalldata = context.ethereumProvider.l1SwapRouterExactOutputSingle({
            tokenIn: tokenIn,
            tokenOut: l1FeeTokenAddress,
            fee: BigInt(swapFee ?? 500), // Stable default to 0.05% pool fee
            recipient: l1HandlerAddress,
            deadline: BigInt(Math.floor(Date.now() / 1000) + 600), // 10 minutes from now
            amountOut: amountOut,
            amountInMaximum: amountInMaximum,
            sqrtPriceLimitX96: 0n, // No price limit should be fine as we protect the swap using amountInMaximum
        })
    } // On Sepolia, only the legacy swap router is available, and it supports exactOutputSingle parameters without a deadline.
    else if (
        registry.environment === "paseo_sepolia" ||
        registry.environment === "westend_sepolia"
    ) {
        swapCalldata = context.ethereumProvider.l1LegacySwapRouterExactOutputSingle({
            tokenIn: tokenIn,
            tokenOut: l1FeeTokenAddress,
            fee: BigInt(swapFee ?? 500), // Stable default to 0.05% pool fee
            recipient: l1HandlerAddress,
            amountOut: amountOut,
            amountInMaximum: amountInMaximum,
            sqrtPriceLimitX96: 0n, // No price limit should be fine as we protect the swap using amountInMaximum
        })
    } else {
        throw new Error(`Unsupported environment ${registry.environment} for L1 swap router.`)
    }
    return swapCalldata
}

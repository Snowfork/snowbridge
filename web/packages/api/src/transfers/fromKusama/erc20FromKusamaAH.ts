import { ApiPromise } from "@polkadot/api"
import { AddressOrPair, SignerOptions, SubmittableExtrinsic } from "@polkadot/api/types"
import { ISubmittableResult } from "@polkadot/types/types"
import { isHex, u8aToHex } from "@polkadot/util"
import { blake2AsHex, decodeAddress, xxhashAsHex } from "@polkadot/util-crypto"
import { BN } from "@polkadot/util"
import { Codec } from "@polkadot/types/types"
import {
    AssetRegistry,
    ChainId,
    EthereumChain,
    EthereumProviderTypes,
    Parachain,
    TransferRoute,
} from "@snowbridge/base-types"
import {
    DeliveryFee,
    MessageReceipt,
    Transfer,
    ValidatedTransfer,
    ValidationKind,
    ValidationLog,
    ValidationReason,
} from "../../fromKusamaSnowbridgeV2"
import { Context } from "../.."
import { TransferInterface } from "./transferInterface"
import { ensureValidationSuccess, padFeeByPercentage, resolveBeneficiary, u32ToLeBytes } from "../../utils"
import {
    DOT_LOCATION,
    erc20Location,
    kusamaAssetHubLocation,
    polkadotAssetHubLocation,
    NATIVE_TOKEN_LOCATION,
    isRelaychainLocation,
    ksmLocationOnPolkadotAssetHub,
    dotLocationOnKusamaAssetHub,
    buildAssetHubERC20TransferToKusama,
} from "../../xcmBuilder"
import { buildPolkadotAHCustomXcm } from "../../xcmbuilders/fromKusama/erc20FromKusamaAH"
import {
    estimateEthereumExecutionFee,
    getSnowbridgeDeliveryFee,
} from "../../toEthereumSnowbridgeV2"
import { ETHER_TOKEN_ADDRESS } from "../../assets_v2"
import {
    buildExportXcm,
} from "../../xcmbuilders/toEthereum/erc20FromAH"

function buildMessageId(
    sourceParaId: number,
    sourceAccountHex: string,
    accountNonce: number,
    tokenAddress: string,
    beneficiaryAccount: string,
    amount: bigint,
): string {
    const { stringToU8a, hexToU8a } = require("@polkadot/util")
    const entropy = new Uint8Array([
        ...stringToU8a(sourceParaId.toString()),
        ...hexToU8a(sourceAccountHex),
        ...u32ToLeBytes(accountNonce),
        ...hexToU8a(tokenAddress),
        ...stringToU8a(beneficiaryAccount),
        ...stringToU8a(amount.toString()),
    ])
    return blake2AsHex(entropy)
}

export class ERC20FromKusamaAH<T extends EthereumProviderTypes> implements TransferInterface<T> {
    constructor(
        public readonly context: Context<T>,
        public readonly registry: AssetRegistry,
        public readonly route: TransferRoute,
        public readonly source: Parachain,
        public readonly destination: EthereumChain,
    ) {}

    get from(): ChainId {
        return this.route.from
    }

    get to(): ChainId {
        return this.route.to
    }

    async fee(
        tokenAddress: string,
        options?: {
            padPercentage?: bigint
            slippagePadPercentage?: bigint
        },
    ): Promise<DeliveryFee> {
        if (!this.registry.kusama) {
            throw Error("Kusama config is not set in the registry.")
        }

        const kusamaAssetHub = await this.context.kusamaAssetHub()
        const polkadotAssetHub = await this.context.assetHub()

        const kusamaAHImpl = await this.context.paraImplementation(kusamaAssetHub)
        const polkadotAHImpl = await this.context.paraImplementation(polkadotAssetHub)

        const feePadPercentage = options?.padPercentage ?? 33n

        // 1. Kusama bridge fee (export from Kusama AH to Polkadot AH)
        const kusamaBridgeFeeStorageKey = xxhashAsHex(":XcmBridgeHubRouterBaseFee:", 128, true)
        const kusamaBridgeFeeItem = await kusamaAssetHub.rpc.state.getStorage(kusamaBridgeFeeStorageKey)
        let kusamaBridgeFee = new BN((kusamaBridgeFeeItem as Codec).toHex().replace("0x", ""), "hex", "le")
        let kusamaBridgeFeeNative = kusamaBridgeFee.eqn(0) ? 10_602_492_378n : BigInt(kusamaBridgeFee.toString())
        kusamaBridgeFeeNative = padFeeByPercentage(kusamaBridgeFeeNative, feePadPercentage)

        // 2. Kusama BridgeHub delivery fee
        // Estimate using a dummy XCM
        const dummyXcm = buildAssetHubERC20TransferToKusama(
            kusamaAssetHub.registry,
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000000000000000000000000000",
        )
        let kusamaBridgeHubDeliveryFee = await kusamaAHImpl.calculateDeliveryFeeInDOT(
            this.registry.kusama.bridgeHubParaId,
            dummyXcm,
        )
        kusamaBridgeHubDeliveryFee = padFeeByPercentage(kusamaBridgeHubDeliveryFee, feePadPercentage)

        // 3. Polkadot AH execution fee (for the custom XCM that swaps and initiates Snowbridge transfer)
        const sourceAssetMetadata = this.source.assets[tokenAddress.toLowerCase()]
        if (!sourceAssetMetadata) {
            throw Error(`Token ${tokenAddress} not registered on source parachain.`)
        }
        const ahAssetMetadata =
            this.registry.parachains[`polkadot_${this.registry.assetHubParaId}`].assets[
                tokenAddress.toLowerCase()
            ]
        if (!ahAssetMetadata) {
            throw Error(`Token ${tokenAddress} not registered on Polkadot asset hub.`)
        }

        // Build a mock delivery fee for the Polkadot AH custom XCM
        const snowbridgeDeliveryFeeDOT = await getSnowbridgeDeliveryFee(polkadotAssetHub)
        const bridgeHubDeliveryFeeDOT = await polkadotAHImpl.calculateDeliveryFeeInDOT(
            this.registry.bridgeHubParaId,
            buildExportXcm(
                polkadotAssetHub.registry,
                this.registry.ethChainId,
                ahAssetMetadata,
                "0x0000000000000000000000000000000000000000000000000000000000000000",
                "0x0000000000000000000000000000000000000000",
                "0x0000000000000000000000000000000000000000000000000000000000000000",
                1n,
                1n,
            ),
        )

        let polkadotAHExecutionFee = snowbridgeDeliveryFeeDOT + bridgeHubDeliveryFeeDOT
        polkadotAHExecutionFee = padFeeByPercentage(polkadotAHExecutionFee, feePadPercentage)

        // 4. Ethereum execution fee
        let ethereumExecutionFee = await estimateEthereumExecutionFee(
            this.context,
            this.registry,
            this.source,
            tokenAddress,
        )

        // Convert to DOT for the total
        const totalFeeInDOT = polkadotAHExecutionFee + snowbridgeDeliveryFeeDOT

        // Convert DOT fees to KSM using the Kusama AH swap
        let totalFeeInKSM = kusamaBridgeFeeNative + kusamaBridgeHubDeliveryFee
        // Add the DOT amount needed (converted to KSM)
        const dotAsKSM = await kusamaAHImpl.getAssetHubConversionPalletSwap(
            dotLocationOnKusamaAssetHub,
            NATIVE_TOKEN_LOCATION,
            totalFeeInDOT,
        )
        totalFeeInKSM += padFeeByPercentage(dotAsKSM, feePadPercentage)

        return {
            kind: "kusama->ethereum",
            kusamaBridgeFee: kusamaBridgeFeeNative,
            kusamaBridgeHubDeliveryFee,
            polkadotAHExecutionFee,
            snowbridgeDeliveryFee: snowbridgeDeliveryFeeDOT,
            ethereumExecutionFee,
            totalFeeInDOT,
            totalFeeInKSM,
        }
    }

    async tx(
        sourceAccount: string,
        beneficiaryAccount: string,
        tokenAddress: string,
        amount: bigint,
        fee: DeliveryFee,
    ): Promise<Transfer> {
        if (!this.registry.kusama) {
            throw Error("Kusama config is not set in the registry.")
        }

        const kusamaAssetHub = await this.context.kusamaAssetHub()
        const polkadotAssetHub = await this.context.assetHub()

        let sourceAccountHex = sourceAccount
        if (!isHex(sourceAccountHex)) {
            sourceAccountHex = u8aToHex(decodeAddress(sourceAccount))
        }

        const { hexAddress: beneficiaryAddressHex } = resolveBeneficiary(beneficiaryAccount)

        const tokenErcMetadata =
            this.registry.ethereumChains[`ethereum_${this.registry.ethChainId}`].assets[
                tokenAddress.toLowerCase()
            ]
        if (!tokenErcMetadata) {
            throw Error(
                `No token ${tokenAddress} registered on ethereum chain ${this.registry.ethChainId}.`,
            )
        }
        const ahAssetMetadata =
            this.registry.parachains[`polkadot_${this.registry.assetHubParaId}`].assets[
                tokenAddress.toLowerCase()
            ]
        if (!ahAssetMetadata) {
            throw Error(`Token ${tokenAddress} not registered on Polkadot asset hub.`)
        }
        const sourceAssetMetadata = this.source.assets[tokenAddress.toLowerCase()]
        if (!sourceAssetMetadata) {
            throw Error(`Token ${tokenAddress} not registered on source parachain.`)
        }

        const kusamaAHImpl = await this.context.paraImplementation(kusamaAssetHub)
        const accountNonce = await kusamaAHImpl.accountNonce(sourceAccountHex)
        let messageId = buildMessageId(
            kusamaAHImpl.parachainId,
            sourceAccountHex,
            accountNonce,
            tokenAddress,
            beneficiaryAccount,
            amount,
        )

        // Get the token location on Kusama AH
        let tokenLocation = sourceAssetMetadata.location
        if (!tokenLocation) {
            tokenLocation = erc20Location(this.registry.ethChainId, tokenAddress)
        }

        // Build the custom XCM for Polkadot AH (receives from Kusama, forwards to Ethereum)
        // Must use kusamaAssetHub.registry since the extrinsic is submitted on Kusama AH
        const customXcm = buildPolkadotAHCustomXcm(
            kusamaAssetHub.registry,
            this.registry.ethChainId,
            sourceAccountHex,
            beneficiaryAddressHex,
            messageId,
            ahAssetMetadata,
            amount,
            fee.polkadotAHExecutionFee,
            fee.ethereumExecutionFee,
        )

        // Build the extrinsic: transferAssetsUsingTypeAndThen from Kusama AH to Polkadot AH
        // Assets: DOT (for Polkadot AH fees) + ERC20 token
        let tokenLocationOnKusama = tokenLocation
        let assets: any
        if (isRelaychainLocation(tokenLocationOnKusama)) {
            // Token is KSM - not applicable for kusama->ethereum
            throw Error("KSM transfers from Kusama to Ethereum are not supported.")
        }

        // DOT fee for Polkadot AH execution + the ERC20 token
        assets = {
            v5: [
                {
                    id: dotLocationOnKusamaAssetHub,
                    fun: { Fungible: fee.totalFeeInDOT },
                },
                {
                    id: tokenLocationOnKusama,
                    fun: { Fungible: amount },
                },
            ],
        }

        const destination = { v5: polkadotAssetHubLocation(this.registry.assetHubParaId) }

        const feeAsset = {
            v5: dotLocationOnKusamaAssetHub,
        }

        const tx: SubmittableExtrinsic<"promise", ISubmittableResult> =
            kusamaAssetHub.tx.polkadotXcm.transferAssetsUsingTypeAndThen(
                destination,
                assets,
                "DestinationReserve",
                feeAsset,
                "DestinationReserve",
                customXcm,
                "Unlimited",
            )

        return {
            kind: "kusama->ethereum",
            input: {
                registry: this.registry,
                sourceAccount,
                beneficiaryAccount,
                tokenAddress,
                amount,
                fee,
            },
            computed: {
                sourceParaId: kusamaAHImpl.parachainId,
                sourceAccountHex,
                tokenErcMetadata,
                ahAssetMetadata,
                sourceAssetMetadata,
                sourceParachain: this.source,
                messageId,
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
                padPercentage?: bigint
                slippagePadPercentage?: bigint
            }
        },
    ): Promise<ValidatedTransfer> {
        const fee = await this.fee(tokenAddress, options?.fee)
        const transfer = await this.tx(
            sourceAccount,
            beneficiaryAccount,
            tokenAddress,
            amount,
            fee,
        )
        return ensureValidationSuccess(await this.validate(transfer))
    }

    async validate(transfer: Transfer): Promise<ValidatedTransfer> {
        if (!this.registry.kusama) {
            throw Error("Kusama config is not set in the registry.")
        }

        const kusamaAssetHub = await this.context.kusamaAssetHub()
        const kusamaAHImpl = await this.context.paraImplementation(kusamaAssetHub)

        const { sourceAccountHex } = transfer.computed
        const { amount, fee, tokenAddress } = transfer.input
        const registry = this.registry

        let nativeBalance = await kusamaAHImpl.getNativeBalance(sourceAccountHex, true)

        let tokenLocation = transfer.computed.sourceAssetMetadata.location
        if (!tokenLocation) {
            tokenLocation = erc20Location(registry.ethChainId, tokenAddress)
        }

        let tokenBalance: bigint
        if (isRelaychainLocation(tokenLocation)) {
            tokenBalance = nativeBalance
        } else {
            tokenBalance = await kusamaAHImpl.getTokenBalance(
                sourceAccountHex,
                registry.ethChainId,
                tokenAddress,
                transfer.computed.sourceAssetMetadata,
            )
        }

        const logs: ValidationLog[] = []

        if (amount > tokenBalance) {
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.InsufficientTokenBalance,
                message: "Insufficient token balance to submit transaction.",
            })
        }

        const paymentInfo = await transfer.tx.paymentInfo(sourceAccountHex)
        const sourceExecutionFee = paymentInfo["partialFee"].toBigInt()

        if (sourceExecutionFee + fee.totalFeeInKSM > nativeBalance) {
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.InsufficientFee,
                message: "Insufficient KSM balance to submit transaction on Kusama asset hub.",
            })
        }

        // Dry run on Kusama AH
        let dryRunError: any
        const dryRunResult = await dryRunSourceAssetHub(
            kusamaAssetHub,
            this.registry.kusama.assetHubParaId,
            this.registry.kusama.bridgeHubParaId,
            transfer.tx,
            sourceAccountHex,
        )
        if (!dryRunResult.success) {
            dryRunError = dryRunResult.error
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.DryRunFailed,
                message: "Dry run call on Kusama asset hub failed.",
            })
        }

        // Note: Polkadot AH dry-run is not performed for the custom_xcm_on_dest because
        // it executes in a context where assets are already in the holding register from
        // the bridge transfer. Dry-running it in isolation would always fail (Barrier error).

        const success = logs.find((l) => l.kind === ValidationKind.Error) === undefined

        return {
            logs,
            success,
            data: {
                nativeBalance,
                sourceExecutionFee,
                tokenBalance,
                dryRunError,
            },
            ...transfer,
        }
    }

    async signAndSend(
        transfer: Transfer,
        account: AddressOrPair,
        options: Partial<SignerOptions>,
    ): Promise<MessageReceipt> {
        const kusamaAssetHub = await this.context.kusamaAssetHub()
        const result = await new Promise<MessageReceipt>((resolve, reject) => {
            try {
                transfer.tx.signAndSend(account, options, (c) => {
                    if (c.isError) {
                        console.error(c)
                        reject(c.internalError || c.dispatchError || c)
                    }
                    if (c.isFinalized) {
                        const result = {
                            txHash: u8aToHex(c.txHash),
                            txIndex: c.txIndex || 0,
                            blockNumber: Number((c as any).blockNumber),
                            blockHash: "",
                            events: c.events,
                        }
                        for (const e of c.events) {
                            if (kusamaAssetHub.events.system.ExtrinsicFailed.is(e.event)) {
                                resolve({
                                    ...result,
                                    success: false,
                                    dispatchError: (e.event.data.toHuman(true) as any)
                                        ?.dispatchError,
                                })
                            }
                            if (kusamaAssetHub.events.polkadotXcm.Sent.is(e.event)) {
                                resolve({
                                    ...result,
                                    success: true,
                                    messageId: (e.event.data.toPrimitive() as any)[3],
                                })
                            }
                        }
                        resolve({
                            ...result,
                            success: false,
                        })
                    }
                })
            } catch (e) {
                console.error(e)
                reject(e)
            }
        })
        result.blockHash = u8aToHex(
            await kusamaAssetHub.rpc.chain.getBlockHash(result.blockNumber),
        )
        result.messageId = transfer.computed.messageId ?? result.messageId
        return result
    }
}

async function dryRunSourceAssetHub(
    source: ApiPromise,
    assetHubParaId: number,
    bridgeHubParaId: number,
    tx: SubmittableExtrinsic<"promise", ISubmittableResult>,
    sourceAccount: string,
) {
    const origin = { system: { signed: sourceAccount } }
    // Use version 5 for the dry-run API to handle V5 XCM in forwarded results
    let result: any
    try {
        result = await source.call.dryRunApi.dryRunCall<any>(origin, tx.inner.toHex(), 5)
    } catch {
        result = await source.call.dryRunApi.dryRunCall<any>(origin, tx.inner.toHex())
    }

    const success = result.isOk && result.asOk.executionResult.isOk
    let bridgeHubForwarded
    if (!success) {
        console.error(
            "Error during dry run on Kusama AH:",
            sourceAccount,
            tx.toHuman(),
            result.toHuman(true),
        )
    } else {
        bridgeHubForwarded =
            result.asOk.forwardedXcms.find((x: any) => {
                return (
                    x[0].isV4 &&
                    x[0].asV4.parents.toNumber() === 1 &&
                    x[0].asV4.interior.isX1 &&
                    x[0].asV4.interior.asX1[0].isParachain &&
                    x[0].asV4.interior.asX1[0].asParachain.toNumber() === bridgeHubParaId
                )
            }) ??
            result.asOk.forwardedXcms.find((x: any) => {
                return (
                    x[0].isV5 &&
                    x[0].asV5.parents.toNumber() === 1 &&
                    x[0].asV5.interior.isX1 &&
                    x[0].asV5.interior.asX1[0].isParachain &&
                    x[0].asV5.interior.asX1[0].asParachain.toNumber() === bridgeHubParaId
                )
            })
    }
    return {
        success: success && bridgeHubForwarded,
        error:
            result.isOk && result.asOk.executionResult.isErr
                ? result.asOk.executionResult.asErr.toJSON()
                : undefined,
        bridgeHubForwarded,
    }
}

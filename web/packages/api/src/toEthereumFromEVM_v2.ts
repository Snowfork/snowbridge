import { isHex, numberToHex, u8aToHex } from "@polkadot/util"
import { decodeAddress } from "@polkadot/util-crypto"
import {
    buildResultXcmAssetHubERC20TransferFromParachain,
    buildAssetHubERC20TransferFromParachain,
    DOT_LOCATION,
} from "./xcmBuilder"
import {
    Asset,
    AssetRegistry,
    ContractCall,
    ERC20Metadata,
    EthereumChain,
    Parachain,
} from "@snowbridge/base-types"
import { getOperatingStatus, OperationStatus } from "./status"
import { EventRecord } from "@polkadot/types/interfaces"
import { ContractTransaction, TransactionReceipt } from "ethers"
import { paraImplementation } from "./parachains"
import {
    getDeliveryFee as getDeliveryFeeV1,
    buildMessageId,
    createERC20SourceParachainTx,
    DeliveryFee,
    dryRunAssetHub,
    dryRunOnSourceParachain,
    FeeInfo,
    resolveInputs,
    ValidationKind,
    ValidationLog,
    ValidationReason,
} from "./toEthereum_v2"
import { EthersContext } from "./index"
import { TransferInterface as ToEthereumEvmTransferInterface } from "./transfers/toEthereumEvm/transferInterface"

export type TransferEvm = {
    input: {
        registry: AssetRegistry
        sourceAccount: string
        beneficiaryAccount: any
        tokenAddress: string
        amount: bigint
        fee: DeliveryFee
    }
    computed: {
        sourceParaId: number
        sourceAccountHex: string
        tokenErcMetadata: ERC20Metadata
        ahAssetMetadata: Asset
        sourceAssetMetadata: Asset
        sourceParachain: Parachain
        messageId: string
        ethChain?: EthereumChain
        customXcmHex?: string
        xcTokenAddress?: string
    }
    tx: ContractTransaction
}

export class V1ToEthereumEvmAdapter implements ToEthereumEvmTransferInterface {
    async getDeliveryFee(
        source: { sourceParaId: number; context: EthersContext },
        registry: AssetRegistry,
        tokenAddress: string,
        options?: {
            padPercentage?: bigint
            slippagePadPercentage?: bigint
            defaultFee?: bigint
            feeTokenLocation?: any
            claimerLocation?: any
            contractCall?: ContractCall
        },
    ): Promise<DeliveryFee> {
        if (options?.feeTokenLocation !== undefined) {
            throw new Error("v1 toEthereumEVM adapter does not support options.feeTokenLocation.")
        }
        if (options?.claimerLocation !== undefined) {
            throw new Error("v1 toEthereumEVM adapter does not support options.claimerLocation.")
        }
        if (options?.contractCall !== undefined) {
            throw new Error("v1 toEthereumEVM adapter does not support options.contractCall.")
        }
        return getDeliveryFeeV1(source.context, source.sourceParaId, registry, tokenAddress, {
            padPercentage: options?.padPercentage,
            slippagePadPercentage: options?.slippagePadPercentage,
            defaultFee: options?.defaultFee,
        })
    }

    async createTransfer(
        source: { sourceParaId: number; context: EthersContext },
        registry: AssetRegistry,
        sourceAccount: string,
        beneficiaryAccount: string,
        tokenAddress: string,
        amount: bigint,
        fee: DeliveryFee,
        options?: {
            claimerLocation?: any
            contractCall?: ContractCall
        },
    ): Promise<TransferEvm> {
        if (options?.claimerLocation !== undefined) {
            throw new Error("v1 toEthereumEVM adapter does not support options.claimerLocation.")
        }
        if (options?.contractCall !== undefined) {
            throw new Error("v1 toEthereumEVM adapter does not support options.contractCall.")
        }
        return createTransferEvm(
            source,
            registry,
            sourceAccount,
            beneficiaryAccount,
            tokenAddress,
            amount,
            fee,
        )
    }

    async validateTransfer(
        context: EthersContext,
        transfer: TransferEvm,
    ): Promise<ValidationResultEvm> {
        return validateTransferEvm(context, transfer)
    }
}

export function createTransferImplementationV1(
    _sourceParaId: number,
    _registry: AssetRegistry,
    _tokenAddress: string,
): ToEthereumEvmTransferInterface {
    return new V1ToEthereumEvmAdapter()
}

export async function createTransferEvm(
    source: { sourceParaId: number; context: EthersContext },
    registry: AssetRegistry,
    sourceAccount: string,
    beneficiaryAccount: string,
    tokenAddress: string,
    amount: bigint,
    fee: DeliveryFee,
): Promise<TransferEvm> {
    const { ethChainId, assetHubParaId } = registry

    let sourceAccountHex = sourceAccount
    if (!isHex(sourceAccountHex)) {
        sourceAccountHex = u8aToHex(decodeAddress(sourceAccount))
    }
    if (sourceAccountHex.length !== 42) {
        throw Error(`Source address ${sourceAccountHex} is not a 20 byte address.`)
    }

    const parachain = await source.context.parachain(source.sourceParaId)
    const sourceParachainImpl = await paraImplementation(parachain)
    const { tokenErcMetadata, sourceParachain, ahAssetMetadata, sourceAssetMetadata } =
        resolveInputs(registry, tokenAddress, sourceParachainImpl.parachainId)
    if (!sourceParachain.info.evmChainId) {
        throw Error(`Parachain ${sourceParachainImpl.parachainId} is not an EVM chain.`)
    }
    if (!sourceParachain.xcDOT) {
        throw Error(`Parachain ${sourceParachainImpl.parachainId} does not support XC20 DOT.`)
    }
    const ethChain = registry.ethereumChains[`ethereum_${sourceParachain.info.evmChainId}`]
    if (!ethChain) {
        throw Error(
            `Cannot find eth chain ${sourceParachain.info.evmChainId} for parachain ${sourceParachainImpl.parachainId}.`,
        )
    }
    if (!ethChain.precompile) {
        throw Error(`No precompile for eth chain ${sourceParachain.info.evmChainId}.`)
    }
    if (!ethChain.xcDOT) {
        throw Error(`No XC20 DOT for eth chain ${sourceParachain.info.evmChainId}.`)
    }
    if (!ethChain.xcTokenMap || !ethChain.xcTokenMap[tokenAddress]) {
        throw Error(`No XC20 token for token address ${tokenAddress}.`)
    }

    const xcTokenAddress = ethChain.xcTokenMap[tokenAddress]

    const accountNonce = await sourceParachainImpl.accountNonce(sourceAccountHex)
    const messageId = buildMessageId(
        sourceParachainImpl.parachainId,
        sourceAccountHex,
        accountNonce,
        tokenAddress,
        beneficiaryAccount,
        amount,
    )
    const customXcm = buildAssetHubERC20TransferFromParachain(
        parachain.registry,
        ethChainId,
        sourceAccount,
        beneficiaryAccount,
        tokenAddress,
        messageId,
        sourceParachainImpl.parachainId,
        fee.returnToSenderExecutionFeeDOT,
        DOT_LOCATION, // TODO: Support Native fee for EVM chains
    )

    const tx = await source.context.ethereumProvider.evmParachainTransferAssetsUsingTypeAndThenAddress(
        source.context.ethChain(sourceParachain.info.evmChainId),
        ethChain.precompile,
        sourceAccountHex,
        [1, ["0x00" + numberToHex(assetHubParaId, 32).slice(2)]],
        [
            [ethChain.xcDOT, fee.totalFeeInDot],
            [xcTokenAddress, amount],
        ],
        2,
        0,
        2,
        customXcm.toHex(),
    )
    return {
        input: {
            registry,
            sourceAccount,
            beneficiaryAccount,
            tokenAddress,
            amount,
            fee,
        },
        computed: {
            sourceParaId: sourceParachainImpl.parachainId,
            sourceAccountHex,
            tokenErcMetadata,
            sourceParachain,
            ahAssetMetadata,
            sourceAssetMetadata,
            messageId,
            ethChain,
            xcTokenAddress,
        },
        tx,
    }
}

export type ValidationResultEvm = {
    logs: ValidationLog[]
    success: boolean
    data: {
        bridgeStatus: OperationStatus
        nativeBalance: bigint
        dotBalance?: bigint
        tokenBalance: bigint
        feeInfo?: FeeInfo
        sourceDryRunError: any
        assetHubDryRunError: any
    }
    transfer: TransferEvm
}

export async function validateTransferEvm(
    context: EthersContext,
    transfer: TransferEvm,
): Promise<ValidationResultEvm> {
    const { registry, fee, tokenAddress, amount, beneficiaryAccount } = transfer.input
    const {
        sourceAccountHex,
        sourceParaId,
        sourceParachain: source,
        messageId,
        sourceAssetMetadata,
        ethChain,
    } = transfer.computed

    const sourceParachain = await context.parachain(sourceParaId)
    const gateway = context.gateway()
    const bridgeHub = await context.bridgeHub()
    const assetHub = await context.assetHub()
    const sourceEthChain = context.ethChain(ethChain?.id!)
    const { tx } = transfer

    const sourceParachainImpl = await paraImplementation(sourceParachain)
    const logs: ValidationLog[] = []
    let dotBalance: bigint | undefined = undefined
    if (source.features.hasDotBalance) {
        dotBalance = await sourceParachainImpl.getDotBalance(sourceAccountHex)
    }
    let isNativeBalanceTransfer =
        sourceAssetMetadata.decimals === source.info.tokenDecimals &&
        sourceAssetMetadata.symbol == source.info.tokenSymbols
    const [nativeBalance, tokenBalance] = await Promise.all([
        sourceParachainImpl.getNativeBalance(sourceAccountHex, true),
        sourceParachainImpl.getTokenBalance(sourceAccountHex, registry.ethChainId, tokenAddress),
    ])

    let nativeBalanceCheckFailed = false
    if (
        isNativeBalanceTransfer &&
        fee.totalFeeInNative &&
        amount + fee.totalFeeInNative > tokenBalance
    ) {
        nativeBalanceCheckFailed = true
        logs.push({
            kind: ValidationKind.Error,
            reason: ValidationReason.InsufficientTokenBalance,
            message: "Insufficient token balance to submit transaction.",
        })
    } else if (amount > tokenBalance) {
        logs.push({
            kind: ValidationKind.Error,
            reason: ValidationReason.InsufficientTokenBalance,
            message: "Insufficient token balance to submit transaction.",
        })
    }

    // Create a mock tx that calls the substrate extrinsic on pallet-xcm with the same parameters so that we can dry run.
    const mockTx = createERC20SourceParachainTx(
        sourceParachainImpl,
        registry.ethChainId,
        registry.assetHubParaId,
        sourceAccountHex,
        tokenAddress,
        beneficiaryAccount,
        amount,
        fee.totalFeeInDot,
        messageId,
        sourceParaId,
        fee.returnToSenderExecutionFeeDOT,
        fee.totalFeeInNative !== undefined,
    )

    let sourceDryRunError
    let assetHubDryRunError
    if (source.features.hasDryRunApi) {
        // do the dry run, get the forwarded xcm and dry run that
        const dryRunSource = await dryRunOnSourceParachain(
            sourceParachain,
            registry.assetHubParaId,
            registry.bridgeHubParaId,
            mockTx,
            sourceAccountHex,
        )
        if (!dryRunSource.success) {
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.DryRunFailed,
                message: "Dry run call on source failed.",
            })
            sourceDryRunError = dryRunSource.error
        }

        if (dryRunSource.success && sourceParaId !== registry.assetHubParaId) {
            if (!dryRunSource.assetHubForwarded) {
                logs.push({
                    kind: ValidationKind.Error,
                    reason: ValidationReason.DryRunFailed,
                    message: "Dry run call did not provide a forwarded xcm.",
                })
            } else {
                const dryRunResultAssetHub = await dryRunAssetHub(
                    assetHub,
                    sourceParaId,
                    registry.bridgeHubParaId,
                    dryRunSource.assetHubForwarded[1][0],
                )
                if (!dryRunResultAssetHub.success) {
                    logs.push({
                        kind: ValidationKind.Error,
                        reason: ValidationReason.DryRunFailed,
                        message: "Dry run failed on Asset Hub.",
                    })
                    assetHubDryRunError = dryRunResultAssetHub.errorMessage
                }
            }
        }
    } else {
        logs.push({
            kind: ValidationKind.Warning,
            reason: ValidationReason.DryRunApiNotAvailable,
            message: "Source parachain can not dry run call. Cannot verify success.",
        })
        if (sourceParaId !== registry.assetHubParaId) {
            const dryRunResultAssetHub = await dryRunAssetHub(
                assetHub,
                sourceParaId,
                registry.bridgeHubParaId,
                buildResultXcmAssetHubERC20TransferFromParachain(
                    sourceParachain.registry,
                    registry.ethChainId,
                    sourceAccountHex,
                    beneficiaryAccount,
                    tokenAddress,
                    "0x0000000000000000000000000000000000000000000000000000000000000000",
                    amount,
                    fee.totalFeeInDot,
                    fee.assetHubExecutionFeeDOT,
                    sourceParaId,
                    fee.returnToSenderExecutionFeeDOT,
                    DOT_LOCATION, // TODO: Support native fee for EVM
                    DOT_LOCATION,
                    false,
                ),
            )
            if (!dryRunResultAssetHub.success) {
                logs.push({
                    kind: ValidationKind.Error,
                    reason: ValidationReason.DryRunFailed,
                    message: "Dry run failed on Asset Hub.",
                })
                assetHubDryRunError = dryRunResultAssetHub.errorMessage
            }
        }
    }

    if (!dotBalance) {
        logs.push({
            kind: ValidationKind.Error,
            reason: ValidationReason.InsufficientDotFee,
            message: "Could not determine the DOT balance",
        })
    } else if (fee.totalFeeInDot > dotBalance) {
        logs.push({
            kind: ValidationKind.Error,
            reason: ValidationReason.InsufficientDotFee,
            message: "Insufficient DOT balance to submit transaction on the source parachain.",
        })
    }

    let feeInfo: FeeInfo | undefined
    if (logs.length === 0) {
        const [estimatedGas, feeData] = await Promise.all([
            context.ethereumProvider.estimateGas(sourceEthChain, tx),
            context.ethereumProvider.getFeeData(sourceEthChain),
        ])
        const sourceExecutionFee = (feeData.gasPrice ?? 0n) * estimatedGas
        if (sourceExecutionFee === 0n) {
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.FeeEstimationError,
                message: "Could not get fetch fee details.",
            })
        }

        if (sourceExecutionFee > nativeBalance && !nativeBalanceCheckFailed) {
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.InsufficientNativeFee,
                message:
                    "Insufficient native balance to submit transaction on the source parachain.",
            })
        }
        feeInfo = {
            estimatedGas,
            feeData,
            executionFee: sourceExecutionFee,
            totalTxCost: sourceExecutionFee,
        }
    }
    // Recheck balance after execution fee
    if (
        !nativeBalanceCheckFailed &&
        isNativeBalanceTransfer &&
        fee.totalFeeInNative &&
        amount + fee.totalFeeInNative + (feeInfo?.totalTxCost ?? 0n) > tokenBalance
    ) {
        logs.push({
            kind: ValidationKind.Error,
            reason: ValidationReason.InsufficientTokenBalance,
            message: "Insufficient token balance to submit transaction.",
        })
    }
    const bridgeStatus = await getOperatingStatus({ gateway, bridgeHub })
    if (bridgeStatus.toEthereum.outbound !== "Normal") {
        logs.push({
            kind: ValidationKind.Error,
            reason: ValidationReason.BridgeStatusNotOperational,
            message: "Bridge operations have been paused by onchain governance.",
        })
    }

    const success = logs.find((l) => l.kind === ValidationKind.Error) === undefined
    return {
        logs,
        success,
        data: {
            bridgeStatus,
            nativeBalance,
            dotBalance,
            feeInfo,
            tokenBalance,
            sourceDryRunError,
            assetHubDryRunError,
        },
        transfer,
    }
}

export type MessageReceiptEvm = {
    blockNumber: number
    blockHash: string
    substrateBlockHash: string
    txIndex: number
    txHash: string
    success: boolean
    events: EventRecord[]
    dispatchError?: any
    messageId?: string
}

export async function getMessageReceipt(
    source: { sourceParaId: number; context: EthersContext },
    receipt: TransactionReceipt,
): Promise<MessageReceiptEvm> {
    const sourceParachain = await source.context.parachain(source.sourceParaId)
    const blockHash = await sourceParachain.rpc.chain.getBlockHash(receipt.blockNumber)
    const events = await (await sourceParachain.at(blockHash)).query.system.events<EventRecord[]>()
    let success = false
    let dispatchError: any
    let messageId: string | undefined
    const eventTx = events.find(
        (e) =>
            sourceParachain.events.ethereum.Executed.is(e.event) &&
            e.event.data[2].toPrimitive()?.toString().toLowerCase() === receipt.hash.toLowerCase(),
    )
    if (!(eventTx && eventTx.phase.isApplyExtrinsic)) {
        throw Error(`Could not find tx hash ${receipt.hash} in block ${receipt.blockNumber}.`)
    }
    const matchedEvents: EventRecord[] = events.filter(
        (e) =>
            e.phase.isApplyExtrinsic &&
            e.phase.asApplyExtrinsic.toNumber() === eventTx.phase.asApplyExtrinsic.toNumber(),
    )

    for (const e of matchedEvents) {
        const data = e.event.data
        if (sourceParachain.events.system.ExtrinsicFailed.is(e.event)) {
            dispatchError = data.toHuman(true) as any
            break
        } else if (sourceParachain.events.polkadotXcm.Sent.is(e.event)) {
            success = true
            const pData = data.toPrimitive()
            const xcm = (pData as any)[2]
            messageId = xcm.length > 0 ? xcm[xcm.length - 1].setTopic : (pData as any)[3]
            break
        }
    }
    if (!messageId) {
        throw Error(`Not a bridge transfer`)
    }
    return {
        messageId: messageId,
        blockNumber: receipt.blockNumber,
        substrateBlockHash: blockHash.toHex(),
        blockHash: receipt.blockHash,
        txHash: receipt.hash,
        txIndex: receipt.index,
        success: success && receipt.status === 1,
        dispatchError,
        events: matchedEvents.map((x) => x.toPrimitive() as any as EventRecord),
    }
}

import { ApiPromise } from "@polkadot/api"
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
    ERC20Metadata,
    EthereumChain,
    Parachain,
} from "@snowbridge/base-types"
import { getOperatingStatus, OperationStatus } from "./status"
import { IGatewayV1 as IGateway } from "@snowbridge/contract-types"
import { EventRecord } from "@polkadot/types/interfaces"
import { AbstractProvider, Contract, ContractTransaction, TransactionReceipt } from "ethers"
import { paraImplementation } from "./parachains"
import {
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
import { Context } from "./index"

const PALLET_XCM_PRECOMPILE = [
    {
        inputs: [
            {
                components: [
                    { internalType: "uint8", name: "parents", type: "uint8" },
                    { internalType: "bytes[]", name: "interior", type: "bytes[]" },
                ],
                internalType: "struct XCM.Location",
                name: "dest",
                type: "tuple",
            },
            {
                components: [
                    { internalType: "address", name: "asset", type: "address" },
                    { internalType: "uint256", name: "amount", type: "uint256" },
                ],
                internalType: "struct XCM.AssetAddressInfo[]",
                name: "assets",
                type: "tuple[]",
            },
            {
                internalType: "enum XCM.TransferType",
                name: "assetsTransferType",
                type: "uint8",
            },
            { internalType: "uint8", name: "remoteFeesIdIndex", type: "uint8" },
            {
                internalType: "enum XCM.TransferType",
                name: "feesTransferType",
                type: "uint8",
            },
            { internalType: "bytes", name: "customXcmOnDest", type: "bytes" },
        ],
        name: "transferAssetsUsingTypeAndThenAddress",
        outputs: [],
        stateMutability: "nonpayable",
        type: "function",
    },
]

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

export async function createTransferEvm(
    source: { sourceParaId: number; context: Context } | { parachain: ApiPromise },
    registry: AssetRegistry,
    sourceAccount: string,
    beneficiaryAccount: string,
    tokenAddress: string,
    amount: bigint,
    fee: DeliveryFee
): Promise<TransferEvm> {
    const { ethChainId, assetHubParaId } = registry

    let sourceAccountHex = sourceAccount
    if (!isHex(sourceAccountHex)) {
        sourceAccountHex = u8aToHex(decodeAddress(sourceAccount))
    }
    if (sourceAccountHex.length !== 42) {
        throw Error(`Source address ${sourceAccountHex} is not a 20 byte address.`)
    }

    const { parachain } =
        "sourceParaId" in source
            ? { parachain: await source.context.parachain(source.sourceParaId) }
            : source
    const sourceParachainImpl = await paraImplementation(parachain)
    const { tokenErcMetadata, sourceParachain, ahAssetMetadata, sourceAssetMetadata } =
        resolveInputs(registry, tokenAddress, sourceParachainImpl.parachainId)
    if (!sourceParachain.info.evmChainId) {
        throw Error(`Parachain ${sourceParachainImpl.parachainId} is not an EVM chain.`)
    }
    if (!sourceParachain.xcDOT) {
        throw Error(`Parachain ${sourceParachainImpl.parachainId} does not support XC20 DOT.`)
    }
    const ethChain = registry.ethereumChains[sourceParachain.info.evmChainId.toString()]
    if (!ethChain) {
        throw Error(
            `Cannot find eth chain ${sourceParachain.info.evmChainId} for parachain ${sourceParachainImpl.parachainId}.`
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
    const contract = new Contract(ethChain.precompile, PALLET_XCM_PRECOMPILE)

    const messageId = await buildMessageId(
        parachain,
        sourceParachainImpl.parachainId,
        sourceAccountHex,
        tokenAddress,
        beneficiaryAccount,
        amount
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
        DOT_LOCATION // TODO: Support Native fee for EVM chains
    )

    const tx = await contract[
        "transferAssetsUsingTypeAndThenAddress((uint8,bytes[]),(address,uint256)[],uint8,uint8,uint8,bytes)"
    ].populateTransaction(
        // This represents (1,X1(Parachain(1000)))
        [1, ["0x00" + numberToHex(assetHubParaId, 32).slice(2)]],
        // Assets including fee and the ERC20 asset, with fee be the first
        [
            [ethChain.xcDOT, fee.totalFeeInDot],
            [xcTokenAddress, amount],
        ],
        // The TransferType corresponding to asset being sent, 2 represents `DestinationReserve`
        2,
        // index for the fee
        0,
        // The TransferType corresponding to fee asset
        2,
        customXcm.toHex()
    )

    tx.from = sourceAccountHex
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
    context:
        | Context
        | {
              sourceParachain: ApiPromise
              sourceEthChain: AbstractProvider
              assetHub: ApiPromise
              gateway: IGateway
              bridgeHub: ApiPromise
          },
    transfer: TransferEvm
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

    const { sourceParachain, gateway, bridgeHub, assetHub, sourceEthChain } =
        context instanceof Context
            ? {
                  sourceParachain: await context.parachain(sourceParaId),
                  gateway: context.gateway(),
                  bridgeHub: await context.bridgeHub(),
                  assetHub: await context.assetHub(),
                  sourceEthChain: context.ethChain(ethChain?.chainId!),
              }
            : context
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
        sourceParachainImpl.getNativeBalance(sourceAccountHex),
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
        sourceParachain,
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
        fee.totalFeeInNative !== undefined
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
            sourceAccountHex
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
                    message: "Dry run call did not provide a forwared xcm.",
                })
            } else {
                const dryRunResultAssetHub = await dryRunAssetHub(
                    assetHub,
                    sourceParaId,
                    registry.bridgeHubParaId,
                    dryRunSource.assetHubForwarded[1][0]
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
                    DOT_LOCATION
                )
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
            sourceEthChain.estimateGas(tx),
            sourceEthChain.getFeeData(),
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
    source: { sourceParaId: number; context: Context } | { sourceParachain: ApiPromise },
    receipt: TransactionReceipt
): Promise<MessageReceiptEvm> {
    const { sourceParachain } =
        "sourceParaId" in source
            ? { sourceParachain: await source.context.parachain(source.sourceParaId) }
            : source
    const blockHash = await sourceParachain.rpc.chain.getBlockHash(receipt.blockNumber)
    const events = await (await sourceParachain.at(blockHash)).query.system.events<EventRecord[]>()
    let success = false
    let dispatchError: any
    let messageId: string | undefined
    const eventTx = events.find(
        (e) =>
            sourceParachain.events.ethereum.Executed.is(e.event) &&
            e.event.data[2].toPrimitive()?.toString().toLowerCase() === receipt.hash.toLowerCase()
    )
    if (!(eventTx && eventTx.phase.isApplyExtrinsic)) {
        throw Error(`Could not find tx hash ${receipt.hash} in block ${receipt.blockNumber}.`)
    }
    const matchedEvents: EventRecord[] = events.filter(
        (e) =>
            e.phase.isApplyExtrinsic &&
            e.phase.asApplyExtrinsic.toNumber() === eventTx.phase.asApplyExtrinsic.toNumber()
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

import { ApiPromise } from "@polkadot/api";
import { AddressOrPair, SignerOptions, SubmittableExtrinsic } from "@polkadot/api/types";
import { Codec, ISubmittableResult } from "@polkadot/types/types";
import { BN, hexToU8a, isHex, numberToHex, stringToU8a, u8aToHex } from "@polkadot/util";
import { blake2AsHex, decodeAddress, xxhashAsHex } from "@polkadot/util-crypto";
import { bridgeLocation, buildResultXcmAssetHubERC20TransferFromParachain, buildAssetHubERC20TransferFromParachain, DOT_LOCATION, erc20Location, parahchainLocation, buildParachainERC20ReceivedXcmOnDestination } from "./xcmBuilder";
import { Asset, AssetRegistry, calculateDeliveryFee, calculateDestinationFee, ERC20Metadata, EthereumChain, getDotBalance, getNativeBalance, getParachainId, getTokenBalance, padFeeByPercentage, Parachain } from "./assets_v2";
import { getOperatingStatus, OperationStatus } from "./status";
import { IGateway } from "@snowbridge/contract-types";
import { CallDryRunEffects, EventRecord, XcmDryRunApiError, XcmDryRunEffects } from "@polkadot/types/interfaces";
import { Result } from "@polkadot/types";
import { AbstractProvider, Contract, ContractTransaction, FeeData,  TransactionReceipt } from "ethers";

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

export type Transfer = {
    input: {
        registry: AssetRegistry
        sourceAccount: string
        beneficiaryAccount: any
        tokenAddress: string
        amount: bigint
        fee: DeliveryFee
    },
    computed: {
        sourceParaId: number
        sourceAccountHex: string
        tokenErcMetadata: ERC20Metadata
        ahAssetMetadata: Asset
        sourceAssetMetadata: Asset
        sourceParachain: Parachain
        messageId?: string
    },
    tx: SubmittableExtrinsic<"promise", ISubmittableResult>
}

export type TransferEvm = {
    input: {
        registry: AssetRegistry
        sourceAccount: string
        beneficiaryAccount: any
        tokenAddress: string
        amount: bigint
        fee: DeliveryFee
    },
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
    },
    tx: ContractTransaction
}

export type DeliveryFee = {
    snowbridgeDeliveryFeeDOT: bigint
    bridgeHubDeliveryFeeDOT: bigint
    assetHubExecutionFeeDOT: bigint
    returnToSenderExecutionFeeDOT: bigint
    returnToSenderDeliveryFeeDOT: bigint
    totalFeeInDot: bigint
}

export type FeeInfo = {
    estimatedGas: bigint
    feeData: FeeData
    executionFee: bigint
    totalTxCost: bigint
}

export async function createTransfer(
    parachain: ApiPromise,
    registry: AssetRegistry,
    sourceAccount: string,
    beneficiaryAccount: string,
    tokenAddress: string,
    amount: bigint,
    fee: DeliveryFee,
): Promise<Transfer> {
    const { ethChainId, assetHubParaId } = registry

    let sourceAccountHex = sourceAccount
    if (!isHex(sourceAccountHex)) {
        sourceAccountHex = u8aToHex(decodeAddress(sourceAccount))
    }

    const sourceParaId = await getParachainId(parachain)
    const { tokenErcMetadata, sourceParachain, ahAssetMetadata, sourceAssetMetadata } = resolveInputs(registry, tokenAddress, sourceParaId)

    let messageId: string | undefined
    let tx: SubmittableExtrinsic<"promise", ISubmittableResult>;
    if (sourceParaId === assetHubParaId) {
        tx = createERC20AssetHubTx(parachain, ethChainId, tokenAddress, beneficiaryAccount, amount)
    } else {
        messageId = await buildMessageId(parachain, sourceParaId, sourceAccountHex, tokenAddress, beneficiaryAccount, amount)
        tx = createERC20SourceParachainTx(parachain, ethChainId, assetHubParaId, sourceAccountHex, tokenAddress, beneficiaryAccount, amount, fee.totalFeeInDot, messageId, sourceParaId, fee.returnToSenderExecutionFeeDOT)
    }

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
            sourceParaId,
            sourceAccountHex,
            tokenErcMetadata,
            sourceParachain,
            ahAssetMetadata,
            sourceAssetMetadata,
            messageId,
        },
        tx
    }
}

export async function createTransferEvm(
    parachain: ApiPromise,
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

    const sourceParaId = await getParachainId(parachain)
    const { tokenErcMetadata, sourceParachain, ahAssetMetadata, sourceAssetMetadata } = resolveInputs(registry, tokenAddress, sourceParaId)
    if (!sourceParachain.info.evmChainId) {
        throw Error(`Parachain ${sourceParaId} is not an EVM chain.`)
    }
    if (!sourceParachain.xcDOT) {
        throw Error(`Parachain ${sourceParaId} does not support XC20 DOT.`)
    }
    const ethChain = registry.ethereumChains[sourceParachain.info.evmChainId.toString()]
    if (!ethChain) {
        throw Error(`Cannot find eth chain ${sourceParachain.info.evmChainId} for parachain ${sourceParaId}.`)
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

    const messageId = await buildMessageId(parachain, sourceParaId, sourceAccountHex, tokenAddress, beneficiaryAccount, amount)
    const customXcm = buildAssetHubERC20TransferFromParachain(parachain.registry, ethChainId, sourceAccount, beneficiaryAccount, tokenAddress, messageId, sourceParaId, fee.returnToSenderExecutionFeeDOT)

    const tx = await contract["transferAssetsUsingTypeAndThenAddress((uint8,bytes[]),(address,uint256)[],uint8,uint8,uint8,bytes)"]
        .populateTransaction(
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

    tx.from = sourceAccountHex;
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
            sourceParaId,
            sourceAccountHex,
            tokenErcMetadata,
            sourceParachain,
            ahAssetMetadata,
            sourceAssetMetadata,
            messageId,
            ethChain,
            xcTokenAddress
        },
        tx
    }
}

export async function getDeliveryFee(
    connections: { assetHub: ApiPromise, source: ApiPromise },
    parachain: number,
    registry: AssetRegistry,
    padPercentage?: bigint,
    defaultFee?: bigint
): Promise<DeliveryFee> {
    const { assetHub, source } = connections
    // Fees stored in 0x5fbc5c7ba58845ad1f1a9a7c5bc12fad
    const feePadPercentage = padPercentage ?? 33n
    const feeStorageKey = xxhashAsHex(":BridgeHubEthereumBaseFee:", 128, true)
    const feeStorageItem = await assetHub.rpc.state.getStorage(feeStorageKey)
    let leFee = new BN((feeStorageItem as Codec).toHex().replace("0x", ""), "hex", "le")

    let snowbridgeDeliveryFeeDOT = 0n
    if (leFee.eqn(0)) {
        console.warn("Asset Hub onchain BridgeHubEthereumBaseFee not set. Using default fee.")
        snowbridgeDeliveryFeeDOT = defaultFee ?? 2_750_872_500_000n
    }
    else {
        snowbridgeDeliveryFeeDOT = BigInt(leFee.toString())
    }

    const xcm = buildResultXcmAssetHubERC20TransferFromParachain(
        assetHub.registry,
        registry.ethChainId,
        "0x0000000000000000000000000000000000000000000000000000000000000000",
        "0x0000000000000000000000000000000000000000",
        "0x0000000000000000000000000000000000000000",
        "0x0000000000000000000000000000000000000000000000000000000000000000",
        340282366920938463463374607431768211455n,
        340282366920938463463374607431768211455n,
        340282366920938463463374607431768211455n,
        parachain,
        340282366920938463463374607431768211455n,
    )

    let assetHubExecutionFeeDOT = 0n
    let returnToSenderExecutionFeeDOT = 0n;
    let returnToSenderDeliveryFeeDOT = 0n;
    const bridgeHubDeliveryFeeDOT = await calculateDeliveryFee(assetHub, registry.bridgeHubParaId, xcm)
    if (parachain !== registry.assetHubParaId) {
        const returnToSenderXcm = buildParachainERC20ReceivedXcmOnDestination(
            assetHub.registry,
            registry.ethChainId,
            "0x0000000000000000000000000000000000000000",
            340282366920938463463374607431768211455n,
            340282366920938463463374607431768211455n,
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000000000000000000000000000"
        )
        returnToSenderDeliveryFeeDOT = await calculateDeliveryFee(assetHub, parachain, returnToSenderXcm)
        if (registry.parachains[parachain].features.hasXcmPaymentApi) {
            returnToSenderExecutionFeeDOT = padFeeByPercentage(await calculateDestinationFee(source, returnToSenderXcm), feePadPercentage)
        } else {
            console.warn(`Parachain ${parachain} does not support payment apis. Using an estimated fee.`)
            returnToSenderExecutionFeeDOT = padFeeByPercentage(registry.parachains[parachain].estimatedExecutionFeeDOT, feePadPercentage)
        }
        assetHubExecutionFeeDOT = padFeeByPercentage(await calculateDestinationFee(assetHub, xcm), feePadPercentage)
    }

    return {
        snowbridgeDeliveryFeeDOT,
        assetHubExecutionFeeDOT,
        bridgeHubDeliveryFeeDOT,
        returnToSenderDeliveryFeeDOT,
        returnToSenderExecutionFeeDOT,
        totalFeeInDot: snowbridgeDeliveryFeeDOT + assetHubExecutionFeeDOT + returnToSenderExecutionFeeDOT + returnToSenderDeliveryFeeDOT + bridgeHubDeliveryFeeDOT
    }
}

export enum ValidationKind {
    Warning, Error
}

export enum ValidationReason {
    BridgeStatusNotOperational,
    InsufficientTokenBalance,
    FeeEstimationError,
    InsufficientDotFee,
    InsufficientNativeFee,
    DryRunApiNotAvailable,
    DryRunFailed,
}

export type ValidationLog = {
    kind: ValidationKind
    reason: ValidationReason
    message: string
}

export type ValidationResult = {
    logs: ValidationLog[]
    success: boolean
    data: {
        bridgeStatus: OperationStatus
        nativeBalance: bigint
        dotBalance: bigint
        sourceExecutionFee: bigint
        tokenBalance: bigint
        sourceDryRunError: any
        assetHubDryRunError: any
    };
    transfer: Transfer
}

export type ValidationResultEvm = {
    logs: ValidationLog[]
    success: boolean
    data: {
        bridgeStatus: OperationStatus
        nativeBalance: bigint
        dotBalance: bigint
        tokenBalance: bigint
        feeInfo?: FeeInfo
        sourceDryRunError: any
        assetHubDryRunError: any
    };
    transfer: TransferEvm
}

export async function validateTransfer(
    connections: {
        sourceParachain: ApiPromise
        assetHub: ApiPromise
        gateway: IGateway
        bridgeHub: ApiPromise
    },
    transfer: Transfer): Promise<ValidationResult> {

    const { sourceParachain, gateway, bridgeHub, assetHub } = connections
    const { registry, fee, tokenAddress, amount, beneficiaryAccount } = transfer.input
    const { sourceAccountHex, sourceParaId, sourceParachain: source } = transfer.computed
    const { tx } = transfer

    const logs: ValidationLog[] = []

    const [nativeBalance, dotBalance, tokenBalance] = await Promise.all([
        getNativeBalance(sourceParachain, sourceAccountHex),
        getDotBalance(sourceParachain, source.info.specName, sourceAccountHex),
        getTokenBalance(sourceParachain, source.info.specName, sourceAccountHex, registry.ethChainId, tokenAddress)
    ])

    if (amount > tokenBalance) {
        logs.push({ kind: ValidationKind.Error, reason: ValidationReason.InsufficientTokenBalance, message: 'Insufficient token balance to submit transaction.' })
    }

    let sourceDryRunError;
    let assetHubDryRunError;
    if (source.features.hasDryRunApi) {
        // do the dry run, get the forwarded xcm and dry run that
        const dryRunSource = await dryRunOnSourceParachain(sourceParachain, registry.assetHubParaId, registry.bridgeHubParaId, transfer.tx, sourceAccountHex)
        if (!dryRunSource.success) {
            logs.push({ kind: ValidationKind.Error, reason: ValidationReason.DryRunFailed, message: 'Dry run call on source failed.' })
            sourceDryRunError = dryRunSource.error
        }

        if (dryRunSource.success && sourceParaId !== registry.assetHubParaId) {
            if (!dryRunSource.assetHubForwarded) {
                logs.push({ kind: ValidationKind.Error, reason: ValidationReason.DryRunFailed, message: 'Dry run call did not provide a forwared xcm.' })
            } else {
                const dryRunResultAssetHub = await dryRunAssetHub(assetHub, sourceParaId, registry.bridgeHubParaId, dryRunSource.assetHubForwarded[1][0])
                if (!dryRunResultAssetHub.success) {
                    logs.push({ kind: ValidationKind.Error, reason: ValidationReason.DryRunFailed, message: 'Dry run failed on Asset Hub.' })
                    assetHubDryRunError = dryRunResultAssetHub.errorMessage
                }
            }
        }
    } else {
        logs.push({ kind: ValidationKind.Warning, reason: ValidationReason.DryRunApiNotAvailable, message: 'Source parachain can not dry run call. Cannot verify success.' })
        if (sourceParaId !== registry.assetHubParaId) {
            const dryRunResultAssetHub = await dryRunAssetHub(assetHub, sourceParaId, registry.bridgeHubParaId, buildResultXcmAssetHubERC20TransferFromParachain(
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
                fee.returnToSenderExecutionFeeDOT
            ))
            if (!dryRunResultAssetHub.success) {
                logs.push({ kind: ValidationKind.Error, reason: ValidationReason.DryRunFailed, message: 'Dry run failed on Asset Hub.' })
                assetHubDryRunError = dryRunResultAssetHub.errorMessage
            }
        }
    }

    const paymentInfo = await tx.paymentInfo(sourceAccountHex)
    const sourceExecutionFee = paymentInfo['partialFee'].toBigInt()

    if (sourceParaId === registry.assetHubParaId) {
        if ((sourceExecutionFee + fee.totalFeeInDot) > (dotBalance)) {
            logs.push({ kind: ValidationKind.Error, reason: ValidationReason.InsufficientDotFee, message: 'Insufficient DOT balance to submit transaction on the source parachain.' })
        }
    }
    else {
        if (fee.totalFeeInDot > dotBalance) {
            logs.push({ kind: ValidationKind.Error, reason: ValidationReason.InsufficientDotFee, message: 'Insufficient DOT balance to submit transaction on the source parachain.' })
        }
        if (sourceExecutionFee > nativeBalance) {
            logs.push({ kind: ValidationKind.Error, reason: ValidationReason.InsufficientNativeFee, message: 'Insufficient native balance to submit transaction on the source parachain.' })
        }
    }
    const bridgeStatus = await getOperatingStatus({ gateway, bridgeHub })
    if (bridgeStatus.toEthereum.outbound !== "Normal") {
        logs.push({ kind: ValidationKind.Error, reason: ValidationReason.BridgeStatusNotOperational, message: 'Bridge operations have been paused by onchain governance.' })
    }

    const success = logs.find(l => l.kind === ValidationKind.Error) === undefined

    return {
        logs,
        success,
        data: {
            bridgeStatus,
            nativeBalance,
            dotBalance,
            sourceExecutionFee,
            tokenBalance,
            sourceDryRunError,
            assetHubDryRunError
        },
        transfer,
    }
}

export async function validateTransferEvm(
    connections: {
        sourceParachain: ApiPromise
        sourceEthChain: AbstractProvider
        assetHub: ApiPromise
        gateway: IGateway
        bridgeHub: ApiPromise
    },
    transfer: TransferEvm): Promise<ValidationResultEvm> {
    const { sourceParachain, gateway, bridgeHub, assetHub, sourceEthChain } = connections
    const { registry, fee, tokenAddress, amount, beneficiaryAccount } = transfer.input
    const { sourceAccountHex, sourceParaId, sourceParachain: source, messageId } = transfer.computed
    const { tx } = transfer

    const logs: ValidationLog[] = []

    const [nativeBalance, dotBalance, tokenBalance] = await Promise.all([
        getNativeBalance(sourceParachain, sourceAccountHex),
        getDotBalance(sourceParachain, source.info.specName, sourceAccountHex),
        getTokenBalance(sourceParachain, source.info.specName, sourceAccountHex, registry.ethChainId, tokenAddress)
    ])

    if (amount > tokenBalance) {
        logs.push({ kind: ValidationKind.Error, reason: ValidationReason.InsufficientTokenBalance, message: 'Insufficient token balance to submit transaction.' })
    }

    // Create a mock tx that calls the substrate extrinsic on pallet-xcm with the same parameters so that we can dry run.
    const mockTx = createERC20SourceParachainTx(sourceParachain, registry.ethChainId, registry.assetHubParaId, sourceAccountHex, tokenAddress, beneficiaryAccount, amount, fee.totalFeeInDot, messageId, sourceParaId, fee.returnToSenderExecutionFeeDOT)

    let sourceDryRunError;
    let assetHubDryRunError;
    if (source.features.hasDryRunApi) {
        // do the dry run, get the forwarded xcm and dry run that
        const dryRunSource = await dryRunOnSourceParachain(sourceParachain, registry.assetHubParaId, registry.bridgeHubParaId, mockTx, sourceAccountHex)
        if (!dryRunSource.success) {
            logs.push({ kind: ValidationKind.Error, reason: ValidationReason.DryRunFailed, message: 'Dry run call on source failed.' })
            sourceDryRunError = dryRunSource.error
        }

        if (dryRunSource.success && sourceParaId !== registry.assetHubParaId) {
            if (!dryRunSource.assetHubForwarded) {
                logs.push({ kind: ValidationKind.Error, reason: ValidationReason.DryRunFailed, message: 'Dry run call did not provide a forwared xcm.' })
            } else {
                const dryRunResultAssetHub = await dryRunAssetHub(assetHub, sourceParaId, registry.bridgeHubParaId, dryRunSource.assetHubForwarded[1][0])
                if (!dryRunResultAssetHub.success) {
                    logs.push({ kind: ValidationKind.Error, reason: ValidationReason.DryRunFailed, message: 'Dry run failed on Asset Hub.' })
                    assetHubDryRunError = dryRunResultAssetHub.errorMessage
                }
            }
        }
    } else {
        logs.push({ kind: ValidationKind.Warning, reason: ValidationReason.DryRunApiNotAvailable, message: 'Source parachain can not dry run call. Cannot verify success.' })
        if (sourceParaId !== registry.assetHubParaId) {
            const dryRunResultAssetHub = await dryRunAssetHub(assetHub, sourceParaId, registry.bridgeHubParaId, buildResultXcmAssetHubERC20TransferFromParachain(
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
                fee.returnToSenderExecutionFeeDOT
            ))
            if (!dryRunResultAssetHub.success) {
                logs.push({ kind: ValidationKind.Error, reason: ValidationReason.DryRunFailed, message: 'Dry run failed on Asset Hub.' })
                assetHubDryRunError = dryRunResultAssetHub.errorMessage
            }
        }
    }

    if (fee.totalFeeInDot > dotBalance) {
        logs.push({ kind: ValidationKind.Error, reason: ValidationReason.InsufficientDotFee, message: 'Insufficient DOT balance to submit transaction on the source parachain.' })
    }

    let feeInfo: FeeInfo | undefined;
    if (logs.length === 0) {
        const [estimatedGas, feeData] = await Promise.all([
            sourceEthChain.estimateGas(tx),
            sourceEthChain.getFeeData(),
        ])
        const sourceExecutionFee = (feeData.gasPrice ?? 0n) * estimatedGas
        if (sourceExecutionFee === 0n) {
            logs.push({ kind: ValidationKind.Error, reason: ValidationReason.FeeEstimationError, message: 'Could not get fetch fee details.' })
        }

        if (sourceExecutionFee > nativeBalance) {
            logs.push({ kind: ValidationKind.Error, reason: ValidationReason.InsufficientNativeFee, message: 'Insufficient native balance to submit transaction on the source parachain.' })
        }
        feeInfo = {
            estimatedGas,
            feeData,
            executionFee: sourceExecutionFee,
            totalTxCost: sourceExecutionFee,
        }
    }

    const bridgeStatus = await getOperatingStatus({ gateway, bridgeHub })
    if (bridgeStatus.toEthereum.outbound !== "Normal") {
        logs.push({ kind: ValidationKind.Error, reason: ValidationReason.BridgeStatusNotOperational, message: 'Bridge operations have been paused by onchain governance.' })
    }

    const success = logs.find(l => l.kind === ValidationKind.Error) === undefined
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
            assetHubDryRunError
        },
        transfer,
    }
}

export type MessageReceipt = {
    blockNumber: number
    blockHash: string
    txIndex: number
    txHash: string
    success: boolean
    events: EventRecord[]
    dispatchError?: any
    messageId?: string
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

export async function signAndSend(parachain: ApiPromise, transfer: Transfer, account: AddressOrPair, options: Partial<SignerOptions>): Promise<MessageReceipt> {
    const result = await new Promise<MessageReceipt>((resolve, reject) => {
        try {
            transfer.tx.signAndSend(account, options, (c) => {
                if (c.isError) {
                    console.error(c)
                    reject(c.internalError || c.dispatchError || c)
                }
                if (c.isInBlock) {
                    const result = {
                        txHash: u8aToHex(c.txHash),
                        txIndex: c.txIndex || 0,
                        blockNumber: Number((c as any).blockNumber),
                        blockHash: "",
                        events: c.events,
                    }
                    for (const e of c.events) {
                        if (parachain.events.system.ExtrinsicFailed.is(e.event)) {
                            resolve({
                                ...result,
                                success: false,
                                dispatchError: (e.event.data.toHuman(true) as any)
                                    ?.dispatchError,
                            })
                        }

                        if (parachain.events.polkadotXcm.Sent.is(e.event)) {
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

    result.blockHash = u8aToHex(await parachain.rpc.chain.getBlockHash(result.blockNumber))
    result.messageId = transfer.computed.messageId ?? result.messageId

    return result
}

export async function getMessageReceipt(sourceParachain: ApiPromise, receipt: TransactionReceipt): Promise<MessageReceiptEvm> {
    const blockHash = await sourceParachain.rpc.chain.getBlockHash(receipt.blockNumber)
    const events = await (await sourceParachain.at(blockHash)).query.system.events<EventRecord[]>()
    let success = false
    let dispatchError: any
    let messageId: string | undefined
    const eventTx = events.find(e =>
        sourceParachain.events.ethereum.Executed.is(e.event)
        && e.event.data[2].toPrimitive()?.toString().toLowerCase() === receipt.hash.toLowerCase()
    )
    if (!(eventTx && eventTx.phase.isApplyExtrinsic)) {
        throw Error(`Could not find tx hash ${receipt.hash} in block ${receipt.blockNumber}.`)
    }
    const matchedEvents: EventRecord[] =
        events.filter(
            e => e.phase.isApplyExtrinsic
                && e.phase.asApplyExtrinsic.toNumber() === eventTx.phase.asApplyExtrinsic.toNumber()
        )

    console.log()
    for (const e of matchedEvents) {
        const data = e.event.data
        if (sourceParachain.events.system.ExtrinsicFailed.is(e.event)) {
            dispatchError = (data.toHuman(true) as any)
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
        events: matchedEvents.map(x => x.toPrimitive() as any as EventRecord)
    }
}

function resolveInputs(registry: AssetRegistry, tokenAddress: string, sourceParaId: number) {
    const tokenErcMetadata = registry.ethereumChains[registry.ethChainId.toString()].assets[tokenAddress.toLowerCase()];
    if (!tokenErcMetadata) {
        throw Error(`No token ${tokenAddress} registered on ethereum chain ${registry.ethChainId}.`)
    }
    const sourceParachain = registry.parachains[sourceParaId.toString()]
    if (!sourceParachain) {
        throw Error(`Could not find ${sourceParaId} in the asset registry.`)
    }
    const ahAssetMetadata = registry.parachains[registry.assetHubParaId].assets[tokenAddress.toLowerCase()]
    if (!ahAssetMetadata) {
        throw Error(`Token ${tokenAddress} not registered on asset hub.`)
    }

    const sourceAssetMetadata = sourceParachain.assets[tokenAddress.toLowerCase()]
    if (!sourceAssetMetadata) {
        throw Error(`Token ${tokenAddress} not registered on source parachain ${sourceParaId}.`)
    }

    return { tokenErcMetadata, sourceParachain, ahAssetMetadata, sourceAssetMetadata }
}

function createERC20AssetHubTx(
    parachain: ApiPromise,
    ethChainId: number,
    tokenAddress: string,
    beneficiaryAccount: string,
    amount: bigint
): SubmittableExtrinsic<"promise", ISubmittableResult> {
    const assetLocation = erc20Location(ethChainId, tokenAddress)
    const assets = {
        v4: [
            {
                id: assetLocation,
                fun: { Fungible: amount },
            }
        ]
    }
    const destination = { v4: bridgeLocation(ethChainId) }
    const beneficiaryLocation = {
        v4: {
            parents: 0,
            interior: { x1: [{ accountKey20: { key: beneficiaryAccount } }] },
        }
    }
    return parachain.tx.polkadotXcm.transferAssets(destination, beneficiaryLocation, assets, 0, "Unlimited")
}

function createERC20SourceParachainTx(
    parachain: ApiPromise,
    ethChainId: number,
    assetHubParaId: number,
    sourceAccount: string,
    tokenAddress: string,
    beneficiaryAccount: string,
    amount: bigint,
    totalFeeInDot: bigint,
    messageId: string,
    sourceParaId: number,
    returnToSenderFeeInDOT: bigint,
): SubmittableExtrinsic<"promise", ISubmittableResult> {
    const assets = {
        v4: [
            {
                id: DOT_LOCATION,
                fun: { Fungible: totalFeeInDot },
            },
            {
                id: erc20Location(ethChainId, tokenAddress),
                fun: { Fungible: amount },
            },
        ]
    }
    const destination = { v4: parahchainLocation(assetHubParaId) }

    const feeAsset = {
        v4: DOT_LOCATION
    }
    const customXcm = buildAssetHubERC20TransferFromParachain(parachain.registry, ethChainId, sourceAccount, beneficiaryAccount, tokenAddress, messageId, sourceParaId, returnToSenderFeeInDOT)
    return parachain.tx.polkadotXcm.transferAssetsUsingTypeAndThen(destination, assets, "DestinationReserve", feeAsset, "DestinationReserve", customXcm, "Unlimited")
}

async function dryRunOnSourceParachain(
    source: ApiPromise,
    assetHubParaId: number,
    bridgeHubParaId: number,
    tx: SubmittableExtrinsic<"promise", ISubmittableResult>,
    sourceAccount: string
) {
    const origin = { system: { signed: sourceAccount } }
    const result = (await source.call.dryRunApi.dryRunCall<Result<CallDryRunEffects, XcmDryRunApiError>>(
        origin,
        tx,
    ))

    let assetHubForwarded;
    let bridgeHubForwarded;
    const success = result.isOk && result.asOk.executionResult.isOk
    if (!success) {
        console.error("Error during dry run on source parachain:", sourceAccount, tx.toHuman(), result.toHuman())
    } else {
        bridgeHubForwarded = result.asOk.forwardedXcms.find(x => {
            return x[0].isV4
                && x[0].asV4.parents.toNumber() === 1
                && x[0].asV4.interior.isX1
                && x[0].asV4.interior.asX1[0].isParachain
                && x[0].asV4.interior.asX1[0].asParachain.toNumber() === bridgeHubParaId
        })
        assetHubForwarded = result.asOk.forwardedXcms.find(x => {
            return x[0].isV4
                && x[0].asV4.parents.toNumber() === 1
                && x[0].asV4.interior.isX1
                && x[0].asV4.interior.asX1[0].isParachain
                && x[0].asV4.interior.asX1[0].asParachain.toNumber() === assetHubParaId
        })
    }
    return {
        success: success && (bridgeHubForwarded || assetHubForwarded),
        error: result.isOk && result.asOk.executionResult.isErr ? result.asOk.executionResult.asErr.toJSON() : undefined,
        assetHubForwarded,
        bridgeHubForwarded,
    }
}

async function dryRunAssetHub(assetHub: ApiPromise, parachainId: number, bridgeHubParaId: number, xcm: any) {
    const sourceParachain = { v4: { parents: 1, interior: { x1: [{ parachain: parachainId }] } } }
    const result = (await assetHub.call.dryRunApi.dryRunXcm<Result<XcmDryRunEffects, XcmDryRunApiError>>(
        sourceParachain,
        xcm
    ))

    const resultPrimitive = result.toPrimitive() as any
    const resultHuman = result.toHuman() as any

    const success = result.isOk && result.asOk.executionResult.isComplete
    let sourceParachainForwarded;
    let bridgeHubForwarded;
    if (!success) {
        console.error("Error during dry run on asset hub:", xcm.toHuman(), result.toHuman())
    } else {
        bridgeHubForwarded = result.asOk.forwardedXcms.find(x => {
            return x[0].isV4
                && x[0].asV4.parents.toNumber() === 1
                && x[0].asV4.interior.isX1
                && x[0].asV4.interior.asX1[0].isParachain
                && x[0].asV4.interior.asX1[0].asParachain.toNumber() === bridgeHubParaId
        })
        sourceParachainForwarded = result.asOk.forwardedXcms.find(x => {
            return x[0].isV4
                && x[0].asV4.parents.toNumber() === 1
                && x[0].asV4.interior.isX1
                && x[0].asV4.interior.asX1[0].isParachain
                && x[0].asV4.interior.asX1[0].asParachain.toNumber() === parachainId
        })
    }
    return {
        success: success && bridgeHubForwarded,
        sourceParachainForwarded,
        bridgeHubForwarded,
        errorMessage: resultHuman.Ok.executionResult.Incomplete?.error,
    }
}

async function buildMessageId(parachain: ApiPromise, sourceParaId: number, sourceAccountHex: string, tokenAddress: string, beneficiaryAccount: string, amount: bigint) {
    const [accountNextId] = await Promise.all([
        parachain.rpc.system.accountNextIndex(sourceAccountHex),
    ]);
    const entropy = new Uint8Array([
        ...stringToU8a(sourceParaId.toString()),
        ...hexToU8a(sourceAccountHex),
        ...accountNextId.toU8a(),
        ...hexToU8a(tokenAddress),
        ...stringToU8a(beneficiaryAccount),
        ...stringToU8a(amount.toString()),
    ]);
    return blake2AsHex(entropy);
}
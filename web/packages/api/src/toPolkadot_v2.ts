import { MultiAddressStruct } from "@snowbridge/contract-types/src/IGateway";
import { AbstractProvider, Contract, ContractTransaction, FeeData, LogDescription, TransactionReceipt } from "ethers";
import { beneficiaryMultiAddress } from "./utils";
import { IERC20__factory, IGateway, IGateway__factory } from "@snowbridge/contract-types";
import { Asset, AssetRegistry, ERC20Metadata, Parachain } from "./assets_v2";
import { getOperatingStatus, OperationStatus } from "./status";
import { ApiPromise } from "@polkadot/api";
import { buildAssetHubERC20ReceivedXcm, buildParachainERC20ReceivedXcmOnAssetHub } from "./xcmBuilder";

export type Transfer = {
    input: {
        registry: AssetRegistry
        sourceAccount: string
        beneficiaryAccount: string
        tokenAddress: string
        destinationParaId: number
        amount: bigint
        deliveryFeeInWei: bigint
    },
    computed: {
        gatewayAddress: string
        beneficiaryAddressHex: string
        beneficiaryMultiAddress: MultiAddressStruct
        totalValue: bigint
        tokenErcMetadata: ERC20Metadata
        ahAssetMetadata: Asset
        destAssetMetadata: Asset
        destParachain: Parachain
        destinationFeeInDOT: bigint
        minimalBalance: bigint
    },
    tx: ContractTransaction
}

export enum ValidationKind {
    Warning, Error
}

export enum ValidationReason {
    MinimumAmountValidation,
    GatewaySpenderLimitReached,
    InsufficientTokenBalance,
    FeeEstimationError,
    InsufficientEther,
    BridgeStatusNotOperational,
    DryRunNotSupportedOnDestination,
    NoDestinationParachainConnection,
    DryRunFailed,
}

export type ValidationLog = {
    kind: ValidationKind
    reason: ValidationReason
    message: string
}

export type FeeInfo = {
    estimatedGas: bigint
    feeData: FeeData
    executionFee: bigint
    totalTxCost: bigint
}

export type DeliveryFee = {
    deliveryFeeInWei: bigint
}

export type Validation = {
    logs: ValidationLog[]
    data: {
        etherBalance: bigint
        tokenBalance: {
            balance: bigint
            gatewayAllowance: bigint
        };
        feeInfo?: FeeInfo
        bridgeStatus: OperationStatus
        assetHubDryRunError?: string
        destinationParachainDryRunError?: string
    };
    transfer: Transfer;
}

interface Connections {
    ethereum: AbstractProvider
    gateway: IGateway
    bridgeHub: ApiPromise
    assetHub: ApiPromise
    destParachain?: ApiPromise
}

export type MessageReceipt = {
    channelId: string
    nonce: bigint
    messageId: string
    blockNumber: number
    blockHash: string
}

export async function getDeliveryFee(gateway: IGateway, registry: AssetRegistry, tokenAddress: string, destinationParaId: number): Promise<DeliveryFee> {
    const { destParachain } = resolveInputs(registry, tokenAddress, destinationParaId)
    return {
        deliveryFeeInWei: await gateway.quoteSendTokenFee(tokenAddress, destinationParaId, destParachain.destinationFeeInDOT)
    }
}

export async function createTransfer(
    registry: AssetRegistry,
    sourceAccount: string,
    beneficiaryAccount: string,
    tokenAddress: string,
    destinationParaId: number,
    amount: bigint,
    fee: DeliveryFee,
): Promise<Transfer> {
    const { tokenErcMetadata, destParachain, ahAssetMetadata, destAssetMetadata } = resolveInputs(registry, tokenAddress, destinationParaId)
    const minimalBalance = ahAssetMetadata.minimumBalance > destAssetMetadata.minimumBalance
        ? ahAssetMetadata.minimumBalance : destAssetMetadata.minimumBalance

    let { address: beneficiary, hexAddress: beneficiaryAddressHex } = beneficiaryMultiAddress(beneficiaryAccount)
    const value = fee.deliveryFeeInWei
    const ifce = IGateway__factory.createInterface()
    const con = new Contract(registry.gatewayAddress, ifce);
    const tx = await con.getFunction("sendToken").populateTransaction(
        tokenAddress,
        destinationParaId,
        beneficiary,
        destParachain.destinationFeeInDOT,
        amount,
        {
            value,
            from: sourceAccount
        }
    )

    return {
        input: {
            registry,
            sourceAccount,
            beneficiaryAccount,
            tokenAddress,
            destinationParaId,
            amount,
            deliveryFeeInWei: fee.deliveryFeeInWei,
        }, computed: {
            gatewayAddress: registry.gatewayAddress,
            beneficiaryAddressHex,
            beneficiaryMultiAddress: beneficiary,
            totalValue: value,
            tokenErcMetadata,
            ahAssetMetadata,
            destAssetMetadata,
            minimalBalance,
            destParachain,
            destinationFeeInDOT: destParachain.destinationFeeInDOT
        },
        tx,
    }
}

export async function validateTransfer(connections: Connections, transfer: Transfer): Promise<Validation> {
    const { tx } = transfer
    const { ethereum, gateway, bridgeHub, assetHub, destParachain: destParachainApi } = connections
    const { amount, sourceAccount, tokenAddress, registry, destinationParaId } = transfer.input
    const { totalValue, minimalBalance, destParachain, destAssetMetadata, ahAssetMetadata } = transfer.computed

    const logs: ValidationLog[] = []
    if (amount < minimalBalance) {
        logs.push({ kind: ValidationKind.Error, reason: ValidationReason.MinimumAmountValidation, message: 'The amount transfered is less than the minimum amount.' })
    }
    const [etherBalance, tokenBalance] = await Promise.all([
        ethereum.getBalance(sourceAccount),
        erc20Balance(ethereum, tokenAddress, sourceAccount, registry.gatewayAddress),
    ])
    if (tokenBalance.gatewayAllowance < amount) {
        logs.push({ kind: ValidationKind.Error, reason: ValidationReason.GatewaySpenderLimitReached, message: 'The Snowbridge gateway contract needs to approved as a spender for this token and amount.' })
    }
    if (tokenBalance.balance < amount) {
        logs.push({ kind: ValidationKind.Error, reason: ValidationReason.InsufficientTokenBalance, message: 'The amount transferred is greater than the users token balance.' })
    }
    let feeInfo: FeeInfo | undefined;
    if (logs.length === 0) {
        const [estimatedGas, feeData] = await Promise.all([
            ethereum.estimateGas(tx),
            ethereum.getFeeData(),
        ])
        const executionFee = (feeData.gasPrice ?? 0n) * estimatedGas
        if (executionFee === 0n) {
            logs.push({ kind: ValidationKind.Error, reason: ValidationReason.FeeEstimationError, message: 'Could not get fetch fee details.' })
        }
        const totalTxCost = totalValue + executionFee
        if (etherBalance < totalTxCost) {
            logs.push({ kind: ValidationKind.Error, reason: ValidationReason.InsufficientEther, message: 'Insufficient ether to submit transaction.' })
        }
        feeInfo = {
            estimatedGas,
            feeData,
            executionFee,
            totalTxCost,
        }
    }
    const bridgeStatus = await getOperatingStatus({ gateway, bridgeHub })
    if (bridgeStatus.toPolkadot.outbound !== "Normal" || bridgeStatus.toPolkadot.beacon !== "Normal") {
        logs.push({ kind: ValidationKind.Error, reason: ValidationReason.BridgeStatusNotOperational, message: 'Bridge operations have been paused by onchain governance.' })
    }

    // Check if asset can be received on asset hub (dry run)
    const ahParachain = registry.parachains[registry.assetHubParaId]
    if (!ahParachain.features.hasDryRunApi) {
        logs.push({ kind: ValidationKind.Error, reason: ValidationReason.DryRunNotSupportedOnDestination, message: 'Asset Hub does not support dry running of XCM. Transaction success cannot be confirmed.' })
    }
    // build asset hub packet and dryRun
    const {
        success: dryRunAhSuccess,
        errorMessage: assetHubDryRunError,
        destinationXcm
    } = await dryRunAssetHub(assetHub, transfer)
    if (!dryRunAhSuccess) {
        logs.push({ kind: ValidationKind.Error, reason: ValidationReason.DryRunFailed, message: 'Dry run on Asset Hub failed.' })
    }

    let destinationParachainDryRunError: string | undefined
    if (destinationParaId !== registry.assetHubParaId) {
        // TODO: Check if sovereign account balance for token is at 0 and that consumers is maxxed out.
        if (!destParachainApi) {
            logs.push({ kind: ValidationKind.Warning, reason: ValidationReason.NoDestinationParachainConnection, message: 'The destination paracahain connection was not supplied. Transaction success cannot be confirmed.' })
        } else {
            if (!destAssetMetadata.isSufficient) {
                // TODO: Check acocunt created
            }
            if (destParachain.features.hasDryRunApi) {
                if (!destinationXcm) {
                    logs.push({ kind: ValidationKind.Error, reason: ValidationReason.DryRunFailed, message: 'Dry run on Asset Hub did not produce an XCM to be forwarded to the destination parachain.' })
                }
                const [location, xcm] = destinationXcm
                if (xcm.length !== 1) {
                    logs.push({ kind: ValidationKind.Error, reason: ValidationReason.DryRunFailed, message: 'Dry run on Asset Hub did not produce an XCM to be forwarded to the destination parachain.' })
                }
                const forwardedToCorrectDestination =
                    location.v4.parents === 1 &&
                    location.v4.interior.x1.length === 1 &&
                    location.v4.interior.x1[0].parachain === destinationParaId
                if (!forwardedToCorrectDestination) {
                    logs.push({ kind: ValidationKind.Error, reason: ValidationReason.DryRunFailed, message: 'Dry run on Asset Hub did produced an XCM to be forwarded to an incorrect parachain.' })
                }
                const {
                    success: dryRunDestinationSuccess,
                    errorMessage: destMessage,
                } = await dryRunDestination(destParachainApi, transfer, xcm[0])
                if (!dryRunDestinationSuccess) {
                    logs.push({ kind: ValidationKind.Error, reason: ValidationReason.DryRunFailed, message: 'Dry run on destination parachain failed.' })
                }
                destinationParachainDryRunError = destMessage
            } else {
                logs.push({ kind: ValidationKind.Warning, reason: ValidationReason.DryRunNotSupportedOnDestination, message: 'The destination paracahain does not support dry running of XCM. Transaction success cannot be confirmed.' })
            }
        }
    } else {
        if (!ahAssetMetadata.isSufficient) {
            // TODO: Check acocunt created
        }
        // TODO: Check if sovereign account balance for token is at 0 and that consumers is maxxed out.
    }

    return {
        logs,
        data: {
            etherBalance,
            tokenBalance,
            feeInfo,
            bridgeStatus,
            assetHubDryRunError,
            destinationParachainDryRunError
        },
        transfer,
    }
}

export async function getMessageReceipt(receipt: TransactionReceipt): Promise<MessageReceipt | null> {
    const events: LogDescription[] = []
    const gatewayInterface = IGateway__factory.createInterface()
    receipt.logs.forEach((log) => {
        let event = gatewayInterface.parseLog({
            topics: [...log.topics],
            data: log.data,
        })
        if (event !== null) {
            events.push(event)
        }
    })

    const messageAccepted = events.find((log) => log.name === "OutboundMessageAccepted")
    if (!messageAccepted) return null
    return {
        channelId: String(messageAccepted.args[0]),
        nonce: BigInt(messageAccepted.args[1]),
        messageId: String(messageAccepted.args[2]),
        blockNumber: receipt.blockNumber,
        blockHash: receipt.blockHash,
    }
}

async function erc20Balance(ethereum: AbstractProvider, tokenAddress: string, owner: string, spender: string) {
    const tokenContract = IERC20__factory.connect(tokenAddress, ethereum)
    const [balance, gatewayAllowance] = await Promise.all([
        tokenContract.balanceOf(owner),
        tokenContract.allowance(owner, spender),
    ])
    return {
        balance,
        gatewayAllowance,
    }
}

function resolveInputs(registry: AssetRegistry, tokenAddress: string, destinationParaId: number) {
    const tokenErcMetadata = registry.ethereumChains[registry.ethChainId.toString()].assets[tokenAddress.toLowerCase()];
    if (!tokenErcMetadata) {
        throw Error(`No token ${tokenAddress} registered on ethereum chain ${registry.ethChainId}.`)
    }
    const destParachain = registry.parachains[destinationParaId.toString()]
    if (!destParachain) {
        throw Error(`Could not find ${destinationParaId} in the asset registry.`)
    }
    const ahAssetMetadata = registry.parachains[registry.assetHubParaId].assets[tokenAddress.toLowerCase()]
    if (!ahAssetMetadata) {
        throw Error(`Token ${tokenAddress} not registered on asset hub.`)
    }

    const destAssetMetadata = destParachain.assets[tokenAddress.toLowerCase()]
    if (!destAssetMetadata) {
        throw Error(`Token ${tokenAddress} not registered on destination parachain ${destinationParaId}.`)
    }

    return { tokenErcMetadata, destParachain, ahAssetMetadata, destAssetMetadata }
}

async function dryRunAssetHub(assetHub: ApiPromise, transfer: Transfer) {
    const { registry, amount, tokenAddress, beneficiaryAccount, destinationParaId } = transfer.input
    const { destinationFeeInDOT } = transfer.computed
    const bridgeHubLocation = { v4: { parents: 1, interior: { x1: [{ parachain: registry.bridgeHubParaId }] } } }
    let xcm: any
    if (destinationParaId !== registry.assetHubParaId) {
        xcm = buildParachainERC20ReceivedXcmOnAssetHub(
            assetHub.registry,
            registry.ethChainId,
            tokenAddress,
            destinationParaId,
            amount,
            1_000_000_000_000n, // OK to hard code this here, all fees calculated on chain
            destinationFeeInDOT,
            beneficiaryAccount,
            "0x0000000000000000000000000000000000000000000000000000000000000000",
        )
    }
    else {
        xcm = buildAssetHubERC20ReceivedXcm(
            assetHub.registry,
            registry.ethChainId,
            tokenAddress,
            amount,
            1_000_000_000_000n, // OK to hard code this here, all fees calculated on chain
            beneficiaryAccount,
            "0x0000000000000000000000000000000000000000000000000000000000000000"
        )
    }
    const result = (await assetHub.call.dryRunApi.dryRunXcm(
        bridgeHubLocation,
        xcm
    ))

    const resultPrimitive = result.toPrimitive() as any
    const resultHuman = result.toHuman() as any

    const destinationXcmsLength = resultPrimitive.ok.forwardedXcms.length
    const destinationXcm = destinationXcmsLength > 0 ? resultPrimitive.ok.forwardedXcms[destinationXcmsLength-1] : undefined
    return {
        success: resultPrimitive.ok?.executionResult?.complete !== undefined,
        errorMessage: resultHuman.Ok.executionResult.Incomplete?.error,
        destinationXcm,
    }
}

async function dryRunDestination(destination: ApiPromise, transfer: Transfer, xcm: any) {
    const { registry } = transfer.input
    const assetHubOrigin = { v4: { parents: 1, interior: { x1: [{ parachain: registry.assetHubParaId }] } } }
    const result = (await destination.call.dryRunApi.dryRunXcm(
        assetHubOrigin,
        xcm
    ))

    const resultPrimitive = result.toPrimitive() as any
    const resultHuman = result.toHuman() as any

    return {
        success: resultPrimitive.ok?.executionResult?.complete !== undefined,
        errorMessage: resultHuman.Ok.executionResult.Incomplete?.error,
    }
}

import { MultiAddressStruct } from "@snowbridge/contract-types/src/IGateway.sol/IGatewayV1"
import {
    AbstractProvider,
    Contract,
    ContractTransaction,
    FeeData,
    LogDescription,
    parseUnits,
    TransactionReceipt,
} from "ethers"
import { beneficiaryMultiAddress, padFeeByPercentage, paraIdToSovereignAccount } from "./utils"
import {
    IERC20__factory,
    IGatewayV1 as IGateway,
    IGatewayV1__factory as IGateway__factory,
} from "@snowbridge/contract-types"
import { ETHER_TOKEN_ADDRESS } from "./assets_v2"
import { Asset, AssetRegistry, ERC20Metadata, Parachain } from "@snowbridge/base-types"
import { getOperatingStatus, OperationStatus } from "./status"
import { ApiPromise } from "@polkadot/api"
import {
    buildAssetHubERC20ReceivedXcm,
    buildAssetHubPNAReceivedXcm,
    buildParachainERC20ReceivedXcmOnAssetHub,
    buildParachainERC20ReceivedXcmOnDestination,
    buildParachainPNAReceivedXcmOnAssetHub,
    buildParachainPNAReceivedXcmOnDestination,
    DOT_LOCATION,
} from "./xcmBuilder"
import { Result } from "@polkadot/types"
import { XcmDryRunApiError, XcmDryRunEffects } from "@polkadot/types/interfaces"
import { paraImplementation } from "./parachains"
import { ParachainBase } from "./parachains/parachainBase"
import { Context } from "./index"

export type Transfer = {
    input: {
        registry: AssetRegistry
        sourceAccount: string
        beneficiaryAccount: string
        tokenAddress: string
        destinationParaId: number
        amount: bigint
        fee: DeliveryFee
    }
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
    }
    tx: ContractTransaction
}

export enum ValidationKind {
    Warning,
    Error,
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
    MaxConsumersReached,
    AccountDoesNotExist,
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
    destinationDeliveryFeeDOT: bigint
    destinationExecutionFeeDOT: bigint
    totalFeeInWei: bigint
}

export type ValidationResult = {
    logs: ValidationLog[]
    success: boolean
    data: {
        etherBalance: bigint
        tokenBalance: {
            balance: bigint
            gatewayAllowance: bigint
        }
        feeInfo?: FeeInfo
        bridgeStatus: OperationStatus
        assetHubDryRunError?: string
        destinationParachainDryRunError?: string
    }
    transfer: Transfer
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
    txHash: string
    txIndex: number
}

export async function getDeliveryFee(
    context: Context | { gateway: IGateway; assetHub: ApiPromise; destination: ApiPromise },
    registry: AssetRegistry,
    tokenAddress: string,
    destinationParaId: number,
    paddFeeByPercentage?: bigint
): Promise<DeliveryFee> {
    const { gateway, assetHub, destination } =
        context instanceof Context
            ? {
                  gateway: context.gateway(),
                  assetHub: await context.assetHub(),
                  destination: await context.parachain(destinationParaId),
              }
            : context

    const { destParachain, destAssetMetadata } = resolveInputs(
        registry,
        tokenAddress,
        destinationParaId
    )

    let destinationDeliveryFeeDOT = 0n
    let destinationExecutionFeeDOT = 0n
    if (destinationParaId !== registry.assetHubParaId) {
        let destinationXcm: any
        if (destAssetMetadata.location) {
            destinationXcm = buildParachainPNAReceivedXcmOnDestination(
                destination.registry,
                destAssetMetadata.location,
                340282366920938463463374607431768211455n,
                340282366920938463463374607431768211455n,
                destParachain.info.accountType === "AccountId32"
                    ? "0x0000000000000000000000000000000000000000000000000000000000000000"
                    : "0x0000000000000000000000000000000000000000",
                "0x0000000000000000000000000000000000000000000000000000000000000000"
            )
        } else {
            destinationXcm = buildParachainERC20ReceivedXcmOnDestination(
                destination.registry,
                registry.ethChainId,
                "0x0000000000000000000000000000000000000000",
                340282366920938463463374607431768211455n,
                340282366920938463463374607431768211455n,
                destParachain.info.accountType === "AccountId32"
                    ? "0x0000000000000000000000000000000000000000000000000000000000000000"
                    : "0x0000000000000000000000000000000000000000",
                "0x0000000000000000000000000000000000000000000000000000000000000000"
            )
        }

        const assetHubImpl = await paraImplementation(assetHub)
        destinationDeliveryFeeDOT = await assetHubImpl.calculateDeliveryFeeInDOT(
            destinationParaId,
            destinationXcm
        )
        const destinationImpl = await paraImplementation(destination)
        destinationExecutionFeeDOT = padFeeByPercentage(
            await destinationImpl.calculateXcmFee(destinationXcm, DOT_LOCATION),
            paddFeeByPercentage ?? 33n
        )
    }
    const totalFeeInDOT = destinationExecutionFeeDOT + destinationDeliveryFeeDOT
    return {
        destinationExecutionFeeDOT,
        destinationDeliveryFeeDOT,
        totalFeeInWei: await gateway.quoteSendTokenFee(
            tokenAddress,
            destinationParaId,
            totalFeeInDOT
        ),
    }
}

export async function createTransfer(
    registry: AssetRegistry,
    sourceAccount: string,
    beneficiaryAccount: string,
    tokenAddress: string,
    destinationParaId: number,
    amount: bigint,
    fee: DeliveryFee
): Promise<Transfer> {
    const { tokenErcMetadata, destParachain, ahAssetMetadata, destAssetMetadata } = resolveInputs(
        registry,
        tokenAddress,
        destinationParaId
    )
    const minimalBalance =
        ahAssetMetadata.minimumBalance > destAssetMetadata.minimumBalance
            ? ahAssetMetadata.minimumBalance
            : destAssetMetadata.minimumBalance

    let { address: beneficiary, hexAddress: beneficiaryAddressHex } =
        beneficiaryMultiAddress(beneficiaryAccount)
    let value = fee.totalFeeInWei
    if (tokenAddress === ETHER_TOKEN_ADDRESS) {
        value += amount
    }
    const ifce = IGateway__factory.createInterface()
    const con = new Contract(registry.gatewayAddress, ifce)

    const totalFeeDot = fee.destinationDeliveryFeeDOT + fee.destinationExecutionFeeDOT
    const tx = await con
        .getFunction("sendToken")
        .populateTransaction(tokenAddress, destinationParaId, beneficiary, totalFeeDot, amount, {
            value,
            from: sourceAccount,
        })

    return {
        input: {
            registry,
            sourceAccount,
            beneficiaryAccount,
            tokenAddress,
            destinationParaId,
            amount,
            fee,
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
            destinationFeeInDOT: totalFeeDot,
        },
        tx,
    }
}

async function validateAccount(
    parachainImpl: ParachainBase,
    beneficiaryAddress: string,
    ethChainId: number,
    tokenAddress: string,
    assetMetadata?: Asset,
    maxConsumers?: bigint
) {
    // Check if the account is created
    const [beneficiaryAccount, beneficiaryTokenBalance] = await Promise.all([
        parachainImpl.getNativeAccount(beneficiaryAddress),
        parachainImpl.getTokenBalance(beneficiaryAddress, ethChainId, tokenAddress, assetMetadata),
    ])
    return {
        accountExists: !(
            beneficiaryAccount.consumers === 0n &&
            beneficiaryAccount.providers === 0n &&
            beneficiaryAccount.sufficients === 0n
        ),
        accountMaxConumers:
            beneficiaryAccount.consumers >= (maxConsumers ?? 63n) && beneficiaryTokenBalance === 0n,
    }
}

export async function validateTransfer(
    context: Context | Connections,
    transfer: Transfer
): Promise<ValidationResult> {
    const { tx } = transfer
    const { amount, sourceAccount, tokenAddress, registry, destinationParaId } = transfer.input
    const {
        ethereum,
        gateway,
        bridgeHub,
        assetHub,
        destParachain: destParachainApi,
    } = context instanceof Context
        ? {
              ethereum: context.ethereum(),
              gateway: context.gateway(),
              bridgeHub: await context.bridgeHub(),
              assetHub: await context.assetHub(),
              destParachain: await context.parachain(destinationParaId),
          }
        : context

    const {
        totalValue,
        minimalBalance,
        destParachain,
        destAssetMetadata,
        ahAssetMetadata,
        beneficiaryAddressHex,
    } = transfer.computed

    const logs: ValidationLog[] = []
    if (amount < minimalBalance) {
        logs.push({
            kind: ValidationKind.Error,
            reason: ValidationReason.MinimumAmountValidation,
            message: "The amount transferred is less than the minimum amount.",
        })
    }
    const etherBalance = await ethereum.getBalance(sourceAccount)

    let tokenBalance: { balance: bigint; gatewayAllowance: bigint }
    if (tokenAddress !== ETHER_TOKEN_ADDRESS) {
        tokenBalance = await erc20Balance(
            ethereum,
            tokenAddress,
            sourceAccount,
            registry.gatewayAddress
        )
    } else {
        tokenBalance = {
            balance: etherBalance,
            // u128 max
            gatewayAllowance: 340282366920938463463374607431768211455n,
        }
    }

    // PNA is controlled by Gateway, so no allowance is needed.
    if (tokenBalance.gatewayAllowance < amount && !destAssetMetadata.location) {
        logs.push({
            kind: ValidationKind.Error,
            reason: ValidationReason.GatewaySpenderLimitReached,
            message:
                "The Snowbridge gateway contract needs to approved as a spender for this token and amount.",
        })
    }
    if (tokenBalance.balance < amount) {
        logs.push({
            kind: ValidationKind.Error,
            reason: ValidationReason.InsufficientTokenBalance,
            message: "The amount transferred is greater than the users token balance.",
        })
    }
    let feeInfo: FeeInfo | undefined
    if (logs.length === 0) {
        const [estimatedGas, feeData] = await Promise.all([
            ethereum.estimateGas(tx),
            ethereum.getFeeData(),
        ])
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
    const bridgeStatus = await getOperatingStatus({ gateway, bridgeHub })
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

    // Check if asset can be received on asset hub (dry run)
    const ahParachain = registry.parachains[registry.assetHubParaId]
    let dryRunAhSuccess, forwardedDestination, assetHubDryRunError
    if (!ahParachain.features.hasDryRunApi) {
        logs.push({
            kind: ValidationKind.Warning,
            reason: ValidationReason.DryRunNotSupportedOnDestination,
            message:
                "Asset Hub does not support dry running of XCM. Transaction success cannot be confirmed.",
        })
    } else {
        // build asset hub packet and dryRun
        let result = await dryRunAssetHub(assetHub, transfer)
        dryRunAhSuccess = result.success
        assetHubDryRunError = result.errorMessage
        forwardedDestination = result.forwardedDestination
        if (!dryRunAhSuccess) {
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.DryRunFailed,
                message: "Dry run on Asset Hub failed.",
            })
        }
    }

    const assetHubImpl = await paraImplementation(assetHub)
    let destinationParachainDryRunError: string | undefined
    if (destinationParaId !== registry.assetHubParaId) {
        // Check if sovereign account balance for token is at 0 and that consumers is maxxed out.
        if (!ahAssetMetadata.isSufficient && !dryRunAhSuccess) {
            const sovereignAccountId = paraIdToSovereignAccount("sibl", destinationParaId)
            const { accountMaxConumers, accountExists } = await validateAccount(
                assetHubImpl,
                sovereignAccountId,
                registry.ethChainId,
                tokenAddress,
                ahAssetMetadata
            )

            if (!accountExists) {
                logs.push({
                    kind: ValidationKind.Error,
                    reason: ValidationReason.MaxConsumersReached,
                    message: "Sovereign account does not exist on Asset Hub.",
                })
            }
            if (accountMaxConumers) {
                logs.push({
                    kind: ValidationKind.Error,
                    reason: ValidationReason.MaxConsumersReached,
                    message:
                        "Sovereign account for destination parachain has reached the max consumer limit on Asset Hub.",
                })
            }
        }
        if (!destParachainApi) {
            logs.push({
                kind: ValidationKind.Warning,
                reason: ValidationReason.NoDestinationParachainConnection,
                message:
                    "The destination paracahain connection was not supplied. Transaction success cannot be confirmed.",
            })
        } else {
            if (destParachain.features.hasDryRunApi) {
                if (!forwardedDestination) {
                    logs.push({
                        kind: ValidationKind.Error,
                        reason: ValidationReason.DryRunFailed,
                        message:
                            "Dry run on Asset Hub did not produce an XCM to be forwarded to the destination parachain.",
                    })
                } else {
                    const xcm = forwardedDestination[1]
                    if (xcm.length !== 1) {
                        logs.push({
                            kind: ValidationKind.Error,
                            reason: ValidationReason.DryRunFailed,
                            message:
                                "Dry run on Asset Hub did not produce an XCM to be forwarded to the destination parachain.",
                        })
                    }
                    const { success: dryRunDestinationSuccess, errorMessage: destMessage } =
                        await dryRunDestination(destParachainApi, transfer, xcm[0])
                    if (!dryRunDestinationSuccess) {
                        logs.push({
                            kind: ValidationKind.Error,
                            reason: ValidationReason.DryRunFailed,
                            message: "Dry run on destination parachain failed.",
                        })
                    }
                    destinationParachainDryRunError = destMessage
                }
            } else {
                logs.push({
                    kind: ValidationKind.Warning,
                    reason: ValidationReason.DryRunNotSupportedOnDestination,
                    message:
                        "The destination paracahain does not support dry running of XCM. Transaction success cannot be confirmed.",
                })
            }
            if (
                !destAssetMetadata.isSufficient &&
                ((destParachain.features.hasDryRunApi && destinationParachainDryRunError) ||
                    !destParachain.features.hasDryRunApi)
            ) {
                const destParachainImpl = await paraImplementation(destParachainApi)
                // Check if the account is created
                const { accountMaxConumers, accountExists } = await validateAccount(
                    destParachainImpl,
                    beneficiaryAddressHex,
                    registry.ethChainId,
                    tokenAddress,
                    destAssetMetadata
                )
                if (accountMaxConumers) {
                    logs.push({
                        kind: ValidationKind.Error,
                        reason: ValidationReason.MaxConsumersReached,
                        message:
                            "Beneficiary account has reached the max consumer limit on the destination chain.",
                    })
                }
                if (!accountExists) {
                    logs.push({
                        kind: ValidationKind.Error,
                        reason: ValidationReason.AccountDoesNotExist,
                        message: "Beneficiary account does not exist on the destination chain.",
                    })
                }
            }
        }
    } else if (!ahAssetMetadata.isSufficient && !dryRunAhSuccess) {
        const { accountMaxConumers, accountExists } = await validateAccount(
            assetHubImpl,
            beneficiaryAddressHex,
            registry.ethChainId,
            tokenAddress,
            ahAssetMetadata
        )

        if (accountMaxConumers) {
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
            etherBalance,
            tokenBalance,
            feeInfo,
            bridgeStatus,
            assetHubDryRunError,
            destinationParachainDryRunError,
        },
        transfer,
    }
}

export async function getMessageReceipt(
    receipt: TransactionReceipt
): Promise<MessageReceipt | null> {
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
        txHash: receipt.hash,
        txIndex: receipt.index,
    }
}

async function erc20Balance(
    ethereum: AbstractProvider,
    tokenAddress: string,
    owner: string,
    spender: string
) {
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
    const tokenErcMetadata =
        registry.ethereumChains[registry.ethChainId.toString()].assets[tokenAddress.toLowerCase()]
    if (!tokenErcMetadata) {
        throw Error(`No token ${tokenAddress} registered on ethereum chain ${registry.ethChainId}.`)
    }
    const destParachain = registry.parachains[destinationParaId.toString()]
    if (!destParachain) {
        throw Error(`Could not find ${destinationParaId} in the asset registry.`)
    }
    const ahAssetMetadata =
        registry.parachains[registry.assetHubParaId].assets[tokenAddress.toLowerCase()]
    if (!ahAssetMetadata) {
        throw Error(`Token ${tokenAddress} not registered on asset hub.`)
    }

    const destAssetMetadata = destParachain.assets[tokenAddress.toLowerCase()]
    if (!destAssetMetadata) {
        throw Error(
            `Token ${tokenAddress} not registered on destination parachain ${destinationParaId}.`
        )
    }

    return { tokenErcMetadata, destParachain, ahAssetMetadata, destAssetMetadata }
}

async function dryRunAssetHub(assetHub: ApiPromise, transfer: Transfer) {
    const { registry, amount, tokenAddress, beneficiaryAccount, destinationParaId } = transfer.input
    const { destinationFeeInDOT, destAssetMetadata } = transfer.computed
    const bridgeHubLocation = {
        v4: { parents: 1, interior: { x1: [{ parachain: registry.bridgeHubParaId }] } },
    }
    let xcm: any
    //  taken from chopsticks and based on our exchange rate calculation.
    const baseFee = parseUnits("0.1", transfer.input.registry.relaychain.tokenDecimals)
    const assetHubFee =
        baseFee +
        transfer.input.fee.destinationDeliveryFeeDOT +
        transfer.input.fee.destinationExecutionFeeDOT
    if (destinationParaId !== registry.assetHubParaId) {
        if (destAssetMetadata.location) {
            xcm = buildParachainPNAReceivedXcmOnAssetHub(
                assetHub.registry,
                registry.ethChainId,
                destAssetMetadata.locationOnAH,
                destinationParaId,
                amount,
                assetHubFee,
                destinationFeeInDOT,
                beneficiaryAccount,
                "0x0000000000000000000000000000000000000000000000000000000000000000"
            )
        } else {
            xcm = buildParachainERC20ReceivedXcmOnAssetHub(
                assetHub.registry,
                registry.ethChainId,
                tokenAddress,
                destinationParaId,
                amount,
                assetHubFee,
                destinationFeeInDOT,
                beneficiaryAccount,
                "0x0000000000000000000000000000000000000000000000000000000000000000"
            )
        }
    } else {
        if (destAssetMetadata.location) {
            xcm = buildAssetHubPNAReceivedXcm(
                assetHub.registry,
                registry.ethChainId,
                destAssetMetadata.location,
                amount,
                assetHubFee,
                beneficiaryAccount,
                "0x0000000000000000000000000000000000000000000000000000000000000000"
            )
        } else {
            xcm = buildAssetHubERC20ReceivedXcm(
                assetHub.registry,
                registry.ethChainId,
                tokenAddress,
                amount,
                assetHubFee,
                beneficiaryAccount,
                "0x0000000000000000000000000000000000000000000000000000000000000000"
            )
        }
    }
    const result = await assetHub.call.dryRunApi.dryRunXcm<
        Result<XcmDryRunEffects, XcmDryRunApiError>
    >(bridgeHubLocation, xcm)

    const resultHuman = result.toHuman() as any

    const success = result.isOk && result.asOk.executionResult.isComplete
    let forwardedDestination
    if (!success) {
        console.error("Error during dry run on asset hub:", xcm.toHuman(), result.toHuman())
    } else {
        forwardedDestination = result.asOk.forwardedXcms.find((x) => {
            return (
                x[0].isV4 &&
                x[0].asV4.parents.toNumber() === 1 &&
                x[0].asV4.interior.isX1 &&
                x[0].asV4.interior.asX1[0].isParachain &&
                x[0].asV4.interior.asX1[0].asParachain.toNumber() === destinationParaId
            )
        })
    }
    return {
        success,
        errorMessage: resultHuman.Ok.executionResult.Incomplete?.error,
        forwardedDestination,
    }
}

async function dryRunDestination(destination: ApiPromise, transfer: Transfer, xcm: any) {
    const { registry } = transfer.input
    const assetHubOrigin = {
        v4: { parents: 1, interior: { x1: [{ parachain: registry.assetHubParaId }] } },
    }
    const result = await destination.call.dryRunApi.dryRunXcm<
        Result<XcmDryRunEffects, XcmDryRunApiError>
    >(assetHubOrigin, xcm)

    const resultHuman = result.toHuman() as any

    const success = result.isOk && result.asOk.executionResult.isComplete

    if (!success) {
        console.error("Error during dry run on source parachain:", xcm.toHuman(), result.toHuman())
    }
    return {
        success,
        errorMessage: resultHuman.Ok.executionResult.Incomplete?.error,
    }
}

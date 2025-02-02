import { MultiAddressStruct } from "@snowbridge/contract-types/src/IGateway";
import { AbstractProvider, Contract, ContractTransaction, FeeData } from "ethers";
import { beneficiaryMultiAddress } from "./utils";
import { IERC20__factory, IGateway, IGateway__factory } from "@snowbridge/contract-types";
import { Asset, AssetRegistry, ERC20Metadata, Parachain } from "./assets_v2";
import { getOperatingStatus, OperationStatus } from "./status";
import { ApiPromise } from "@polkadot/api";

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

export async function getDeliveryFee(gateway: IGateway, registry: AssetRegistry, tokenAddress: string, destinationParaId: number): Promise<bigint> {
    const { destParachain } = resolveInputs(registry, tokenAddress, destinationParaId)
    return await gateway.quoteSendTokenFee(tokenAddress, destinationParaId, destParachain.destinationFeeInDOT)
}

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
        minimalBalance: bigint
        destParachain: Parachain
        destinationFeeInDOT: bigint
    },
    tx: ContractTransaction
}

export async function createTransfer(
    registry: AssetRegistry,
    sourceAccount: string,
    beneficiaryAccount: string,
    tokenAddress: string,
    destinationParaId: number,
    amount: bigint,
    deliveryFeeInWei: bigint,
): Promise<Transfer> {
    const { tokenErcMetadata, destParachain, ahAssetMetadata, destAssetMetadata } = resolveInputs(registry, tokenAddress, destinationParaId)
    const minimalBalance = ahAssetMetadata.minimumBalance > destAssetMetadata.minimumBalance
        ? ahAssetMetadata.minimumBalance : destAssetMetadata.minimumBalance

    let { address: beneficiary, hexAddress: beneficiaryAddressHex } = beneficiaryMultiAddress(beneficiaryAccount)
    const value = deliveryFeeInWei
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
            deliveryFeeInWei,
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
}

export type ValidationLog = {
    kind: ValidationKind
    reason: ValidationReason
    message: string
}

export type Validation = {
    logs: ValidationLog[]
    data: {
        etherBalance: bigint
        tokenBalance: {
            balance: bigint
            gatewayAllowance: bigint
        };
        estimatedGas: bigint
        feeData: FeeData
        executionFee: bigint
        bridgeStatus: OperationStatus
        totalTxCost: bigint,
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

export async function validateTransfer({ ethereum, gateway, bridgeHub }: Connections, transfer: Transfer): Promise<Validation> {
    const { tx } = transfer
    const { amount, sourceAccount, tokenAddress, registry, destinationParaId } = transfer.input
    const { totalValue, minimalBalance, destParachain } = transfer.computed

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
    const bridgeStatus = await getOperatingStatus({ gateway, bridgeHub })
    if (bridgeStatus.toPolkadot.outbound !== "Normal" || bridgeStatus.toPolkadot.beacon !== "Normal") {
        logs.push({ kind: ValidationKind.Error, reason: ValidationReason.BridgeStatusNotOperational, message: 'Bridge operations have been paused by onchain governance.' })
    }

    // Check if asset can be received on asset hub (dry run)
    const ahParachain = registry.parachains[registry.assetHubParaId]
    if (!ahParachain.features.hasDryRunApi) {
        logs.push({ kind: ValidationKind.Error, reason: ValidationReason.DryRunNotSupportedOnDestination, message: 'Asset Hub does not support dry running of XCM. Transaction success cannot be confirmed.' })
    }



    if (!destParachain) {
        logs.push({ kind: ValidationKind.Warning, reason: ValidationReason.NoDestinationParachainConnection, message: 'The destination paracahain connection was not supplied. Transaction success cannot be confirmed.' })
    } else if (destinationParaId !== registry.assetHubParaId && destParachain.features.hasDryRunApi) {
        throw Error()
    } else {
        logs.push({ kind: ValidationKind.Warning, reason: ValidationReason.DryRunNotSupportedOnDestination, message: 'The destination paracahain does not support dry running of XCM. Transaction success cannot be confirmed.' })
    }

    return {
        logs,
        data: {
            etherBalance,
            tokenBalance,
            estimatedGas,
            feeData,
            executionFee,
            bridgeStatus,
            totalTxCost
        },
        transfer,
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

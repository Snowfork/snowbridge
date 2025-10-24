import { TransferInterface } from "./transfers/toPolkadot/transferInterface"
import { ERC20ToAH } from "./transfers/toPolkadot/erc20ToAH"
import { Asset, AssetRegistry, ERC20Metadata, Parachain } from "@snowbridge/base-types"
import { PNAToAH } from "./transfers/toPolkadot/pnaToAH"
import { ERC20ToParachain } from "./transfers/toPolkadot/erc20ToParachain"
import { PNAToParachain } from "./transfers/toPolkadot/pnaToParachain"
import { MultiAddressStruct } from "@snowbridge/contract-types/dist/IGateway.sol/IGatewayV1"
import { AbstractProvider, AbiCoder, Contract, ContractTransaction, LogDescription, TransactionReceipt } from "ethers"
import { hexToU8a, stringToU8a } from "@polkadot/util"
import { blake2AsHex } from "@polkadot/util-crypto"
import { IGatewayV2__factory } from "@snowbridge/contract-types"
import { OperationStatus } from "./status"
import { FeeInfo, ValidationLog, ValidationKind, ValidationReason } from "./toPolkadot_v2"
import { ApiPromise } from "@polkadot/api"
import { accountToLocation, ethereumNetwork } from "./xcmBuilder"
import { Codec } from "@polkadot/types/types"
import { ETHER_TOKEN_ADDRESS } from "./assets_v2"
import { beneficiaryMultiAddress } from "./utils"
import { paraImplementation } from "./parachains"
import { getOperatingStatus } from "./status"
export { ValidationKind } from "./toPolkadot_v2"

export type DeliveryFee = {
    feeAsset: any
    assetHubDeliveryFeeEther: bigint
    assetHubExecutionFeeEther: bigint
    destinationDeliveryFeeEther: bigint
    destinationExecutionFeeEther: bigint
    relayerFee: bigint
    totalFeeInWei: bigint
}

export type Transfer = {
    input: {
        registry: AssetRegistry
        sourceAccount: string
        beneficiaryAccount: string
        tokenAddress: string
        destinationParaId: number
        amount: bigint
        fee: DeliveryFee
        customXcm?: any[] // Optional custom XCM instructions
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
        minimalBalance: bigint
        claimer: any
        topic: string
    }
    tx: ContractTransaction
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
        bridgeHubDryRunError?: string
        destinationParachainDryRunError?: string
    }
    transfer: Transfer
}

export type MessageReceipt = {
    nonce: bigint
    payload: any
    blockNumber: number
    blockHash: string
    txHash: string
    txIndex: number
}

export interface TransactOnPolkadotParams {
    registry: AssetRegistry
    sourceAccount: string
    beneficiaryAccount: string
    tokenAddress: string
    amount: bigint
    fee: DeliveryFee
    customXcm: any // XCM instructions to be appended to the standard XCM
}

export function createTransferImplementation(
    destinationParaId: number,
    registry: AssetRegistry,
    tokenAddress: string
): TransferInterface {
    const { ahAssetMetadata } = resolveInputs(registry, tokenAddress, destinationParaId)

    let transferImpl: TransferInterface
    if (destinationParaId == registry.assetHubParaId) {
        if (ahAssetMetadata.location) {
            transferImpl = new PNAToAH()
        } else {
            transferImpl = new ERC20ToAH()
        }
    } else {
        if (ahAssetMetadata.location) {
            transferImpl = new PNAToParachain()
        } else {
            transferImpl = new ERC20ToParachain()
        }
    }
    return transferImpl
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

export function buildMessageId(
    destParaId: number,
    sourceAccountHex: string,
    tokenAddress: string,
    beneficiaryAccount: string,
    amount: bigint,
    accountNonce: number
) {
    const entropy = new Uint8Array([
        ...stringToU8a(destParaId.toString()),
        ...hexToU8a(sourceAccountHex),
        ...hexToU8a(tokenAddress),
        ...stringToU8a(beneficiaryAccount),
        ...stringToU8a(amount.toString()),
        ...stringToU8a(accountNonce.toString()),
    ])
    return blake2AsHex(entropy)
}

// ERC20 asset: abi.encode(0, tokenAddress, amount)
// 0 = AssetKind.NativeTokenERC20 from Solidity Types.sol
export function encodeNativeAsset(tokenAddress: string, amount: bigint) {
    return AbiCoder.defaultAbiCoder().encode(
        ["uint8", "address", "uint128"],
        [0, tokenAddress, amount]
    )
}

// Encode assets array as bytes[] for the gateway contract
export function encodeAssetsArray(encodedAssets: string[]) {
    return AbiCoder.defaultAbiCoder().encode(["bytes[]"], [encodedAssets])
}

export async function getMessageReceipt(
    receipt: TransactionReceipt
): Promise<MessageReceipt | null> {
    const events: LogDescription[] = []
    const gatewayInterface = IGatewayV2__factory.createInterface()
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
        nonce: BigInt(messageAccepted.args[0]),
        payload: messageAccepted.args[1],
        blockNumber: receipt.blockNumber,
        blockHash: receipt.blockHash,
        txHash: receipt.hash,
        txIndex: receipt.index,
    }
}

export function claimerFromBeneficiary(assetHub: ApiPromise, beneficiaryAddressHex: string) {
    let accountLocation = {
        parents: 0,
        interior: { x1: [accountToLocation(beneficiaryAddressHex)] },
    }
    return assetHub.registry.createType("StagingXcmV5Location", accountLocation)
}

export function claimerLocationToBytes(claimerLocation: Codec) {
    return hexToU8a(claimerLocation.toHex())
}

import { TransferInterface } from "./transfers/toPolkadot/transferInterface"
import { ERC20ToAH } from "./transfers/toPolkadot/erc20ToAH"
import { Asset, AssetRegistry, ERC20Metadata, Parachain } from "@snowbridge/base-types"
import { PNAToAH } from "./transfers/toPolkadot/pnaToAH"
import { ERC20ToParachain } from "./transfers/toPolkadot/erc20ToParachain"
import { PNAToParachain } from "./transfers/toPolkadot/pnaToParachain"
import { MultiAddressStruct } from "@snowbridge/contract-types/dist/IGateway.sol/IGatewayV1"
import { AbiCoder, AbstractProvider, ContractTransaction } from "ethers"
import { hexToU8a, stringToU8a } from "@polkadot/util"
import { blake2AsHex } from "@polkadot/util-crypto"
import { IERC20__factory } from "@snowbridge/contract-types"
import { ParachainBase } from "./parachains/parachainBase"
import { OperationStatus } from "./status"
import { FeeInfo, ValidationLog } from "./toPolkadot_v2"
import { ApiPromise } from "@polkadot/api"
import { buildAssetHubERC20ReceivedXcm } from "./xcmbuilders/toPolkadot/erc20ToAH"
import { accountId32Location } from "./xcmBuilder"
import { Result } from "@polkadot/types"
import { XcmDryRunApiError, XcmDryRunEffects } from "@polkadot/types/interfaces"
export { ValidationKind } from "./toPolkadot_v2"

export type DeliveryFee = {
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
        destinationParachainDryRunError?: string
    }
    transfer: Transfer
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
    amount: bigint
) {
    const entropy = new Uint8Array([
        ...stringToU8a(destParaId.toString()),
        ...hexToU8a(sourceAccountHex),
        ...hexToU8a(tokenAddress),
        ...stringToU8a(beneficiaryAccount),
        ...stringToU8a(amount.toString()),
    ])
    return blake2AsHex(entropy)
}

export function hexToBytes(hexString: string): Uint8Array {
    // Ensure the string has an even number of characters
    if (hexString.length % 2 !== 0) {
        throw new Error("Hex string must have an even number of characters.")
    }

    const bytes = new Uint8Array(hexString.length / 2)

    for (let i = 0; i < hexString.length; i += 2) {
        const byteString = hexString.substring(i, i + 2)
        bytes[i / 2] = parseInt(byteString, 16)
    }

    return bytes
}

export async function erc20Balance(
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

// ERC20 asset: abi.encode(0, tokenAddress, amount)
// 0 = AssetKind.NativeTokenERC20 from Solidity Types.sol
export function encodeNativeAsset(tokenAddress: string, amount: bigint) {
    return AbiCoder.defaultAbiCoder().encode(
        ["uint8", "address", "uint128"],
        [0, tokenAddress, amount]
    )
}

// Foreign asset: abi.encode(1, foreignID, amount)
// 1 = AssetKind.ForeignTokenERC20 from Solidity Types.sol
export function encodeForeignAsset(foreignID: string, amount: bigint) {
    return AbiCoder.defaultAbiCoder().encode(
        ["uint8", "bytes32", "uint128"],
        [1, foreignID, amount]
    )
}

// Encode assets array as bytes[] for the gateway contract
export function encodeAssetsArray(encodedAssets: string[]) {
    return AbiCoder.defaultAbiCoder().encode(["bytes[]"], [encodedAssets])
}

export async function validateAccount(
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

export async function dryRunAssetHub(
    assetHub: ApiPromise,
    bridgeHubParaId: number,
    destinationParaId: number,
    xcm: any
) {
    const bridgeHubLocation = {
        v4: { parents: 1, interior: { x1: [{ parachain: bridgeHubParaId }] } },
    }

    const result = await assetHub.call.dryRunApi.dryRunXcm<
        Result<XcmDryRunEffects, XcmDryRunApiError>
    >(bridgeHubLocation, xcm)

    const resultHuman = result.toHuman() as any

    console.dir(resultHuman, { depth: 100 })

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
        if (!forwardedDestination) {
            forwardedDestination = result.asOk.forwardedXcms.find((x) => {
                return (
                    x[0].isV5 &&
                    x[0].asV5.parents.toNumber() === 1 &&
                    x[0].asV5.interior.isX1 &&
                    x[0].asV5.interior.asX1[0].isParachain &&
                    x[0].asV5.interior.asX1[0].asParachain.toNumber() === destinationParaId
                )
            })
        }
    }
    return {
        success,
        errorMessage: resultHuman.Ok.executionResult.Incomplete?.error,
        forwardedDestination,
    }
}

export async function dryRunDestination(destination: ApiPromise, transfer: Transfer, xcm: any) {
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

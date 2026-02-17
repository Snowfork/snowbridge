import { TransferInterface } from "./transfers/toPolkadot/transferInterface"
import { TransferInterface as L2TransferInterface } from "./transfers/l2ToPolkadot/transferInterface"
import { ERC20ToAH } from "./transfers/toPolkadot/erc20ToAH"
import { ERC20ToAH as ERC20FromL2ToAH } from "./transfers/l2ToPolkadot/erc20ToAH"
import { RegisterToken } from "./registration/toPolkadot/registerToken"
import { TokenRegistration } from "./registration/toPolkadot/registrationInterface"
import { Asset, AssetRegistry, ERC20Metadata, Parachain } from "@snowbridge/base-types"
import { PNAToAH } from "./transfers/toPolkadot/pnaToAH"
import { ERC20ToParachain } from "./transfers/toPolkadot/erc20ToParachain"
import { PNAToParachain } from "./transfers/toPolkadot/pnaToParachain"
import { MultiAddressStruct } from "@snowbridge/contract-types/dist/IGateway.sol/IGatewayV1"
import { AbiCoder, ContractTransaction, LogDescription, TransactionReceipt, Wallet } from "ethers"
import { hexToU8a, stringToU8a } from "@polkadot/util"
import { blake2AsHex } from "@polkadot/util-crypto"
import { IGatewayV2__factory } from "@snowbridge/contract-types"
import { OperationStatus } from "./status"
import { FeeInfo, ValidationLog } from "./toPolkadot_v2"
import { ApiPromise } from "@polkadot/api"
import { accountToLocation, DOT_LOCATION, erc20Location } from "./xcmBuilder"
import { Codec } from "@polkadot/types/types"
import { ETHER_TOKEN_ADDRESS } from "./assets_v2"
import { padFeeByPercentage } from "./utils"
import { Context } from "./index"
export { ValidationKind } from "./toPolkadot_v2"
import { ParachainBase } from "./parachains/parachainBase"

export type DeliveryFee = {
    feeAsset: any
    assetHubDeliveryFeeEther: bigint
    assetHubExecutionFeeEther: bigint
    destinationDeliveryFeeEther: bigint
    destinationExecutionFeeEther?: bigint
    destinationExecutionFeeDOT?: bigint
    relayerFee: bigint
    extrinsicFeeDot: bigint // Fee for submitting to BridgeHub in DOT (part of relayerFee)
    extrinsicFeeEther: bigint // Fee for submitting to BridgeHub in Ether (part of relayerFee)
    totalFeeInWei: bigint
    bridgeFeeInL2Token?: bigint // Fee for the actual token transfer in the input L2 token.
    swapFeeInL1Token?: bigint // Fee for Gateway.v2_sendMessage in the output L1 token.
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
        l2TokenAddress?: string
        sourceChainId?: number
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
        l2AdapterAddress?: string
        totalInputAmount: bigint
    }
    tx: ContractTransaction
}

export type ValidationResult = {
    logs: ValidationLog[]
    success: boolean
    data: {
        etherBalance: bigint
        totalInputAmount?: bigint
        tokenBalance: {
            balance: bigint
            gatewayAllowance: bigint
        }
        feeInfo?: FeeInfo
        bridgeStatus: OperationStatus
        assetHubDryRunError?: string
        destinationParachainDryRunError?: string
        l2BridgeDryRunError?: string
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

// Re-export registration types for convenience
export type {
    TokenRegistration,
    RegistrationValidationResult,
    RegistrationFee,
    RegistrationInterface,
} from "./registration/toPolkadot/registrationInterface"

export function createTransferImplementation(
    destinationParaId: number,
    registry: AssetRegistry,
    tokenAddress: string,
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

export function createL2TransferImplementation(
    l2ChainId: number,
    destinationParaId: number,
    registry: AssetRegistry,
    l2TokenAddress: string,
): L2TransferInterface {
    const assets = registry.ethereumChains[`ethereum_l2_${l2ChainId}`].assets
    const tokenMetadata = assets[l2TokenAddress]
    if (!tokenMetadata) {
        throw Error(`No token ${l2TokenAddress} registered on ethereum chain ${l2ChainId}.`)
    }
    const tokenAddress = tokenMetadata.swapTokenAddress
    if (!tokenAddress) {
        throw Error(`No swap token address for ${l2TokenAddress} on ethereum chain ${l2ChainId}.`)
    }

    // Todo: Resolve inputs based on the token address and support non-system destination parachain
    let transferImpl: L2TransferInterface = new ERC20FromL2ToAH()
    return transferImpl
}

function resolveInputs(registry: AssetRegistry, tokenAddress: string, destinationParaId: number) {
    const tokenErcMetadata =
        registry.ethereumChains[`ethereum_${registry.ethChainId}`].assets[
            tokenAddress.toLowerCase()
        ]
    if (!tokenErcMetadata) {
        throw Error(`No token ${tokenAddress} registered on ethereum chain ${registry.ethChainId}.`)
    }
    const destParachain = registry.parachains[`polkadot_${destinationParaId}`]
    if (!destParachain) {
        throw Error(`Could not find ${destinationParaId} in the asset registry.`)
    }
    const ahAssetMetadata =
        registry.parachains[`polkadot_${registry.assetHubParaId}`].assets[
            tokenAddress.toLowerCase()
        ]
    if (!ahAssetMetadata) {
        throw Error(`Token ${tokenAddress} not registered on asset hub.`)
    }

    const destAssetMetadata = destParachain.assets[tokenAddress.toLowerCase()]
    if (!destAssetMetadata) {
        throw Error(
            `Token ${tokenAddress} not registered on destination parachain ${destinationParaId}.`,
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

// ERC20 asset: abi.encode(0, tokenAddress, amount)
// 0 = AssetKind.NativeTokenERC20 from Solidity Types.sol
export function encodeNativeAsset(tokenAddress: string, amount: bigint) {
    return AbiCoder.defaultAbiCoder().encode(
        ["uint8", "address", "uint128"],
        [0, tokenAddress, amount],
    )
}

// Encode assets array as bytes[] for the gateway contract
export function encodeAssetsArray(encodedAssets: string[]) {
    return AbiCoder.defaultAbiCoder().encode(["bytes[]"], [encodedAssets])
}

export async function getMessageReceipt(
    receipt: TransactionReceipt,
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

export function createRegistrationImplementation() {
    return new RegisterToken()
}

export async function sendRegistration(
    registration: TokenRegistration,
    wallet: Wallet,
): Promise<TransactionReceipt> {
    const response = await wallet.sendTransaction(registration.tx)
    const receipt = await response.wait(1)

    if (!receipt) {
        throw Error(`Transaction ${response.hash} not included.`)
    }

    return receipt
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

export async function buildSwapCallData(
    context: Context,
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
    let swapCalldata: string
    if (registry.environment === "polkadot_mainnet") {
        const l1SwapRouter = context.l1SwapRouter()
        swapCalldata = l1SwapRouter.interface.encodeFunctionData("exactOutputSingle", [
            {
                tokenIn: tokenIn,
                tokenOut: context.l1FeeTokenAddress(),
                fee: swapFee ?? 500, // Stable default to 0.05% pool fee
                recipient: context.l1HandlerAddress(),
                deadline: Math.floor(Date.now() / 1000) + 600, // 10 minutes from now
                amountOut: amountOut,
                amountInMaximum: amountInMaximum,
                sqrtPriceLimitX96: 0n, // No price limit should be fine as we protect the swap using amountInMaximum
            },
        ])
    } // On Sepolia, only the legacy swap router is available, and it supports exactOutputSingle parameters without a deadline.
    else if (
        registry.environment === "paseo_sepolia" ||
        registry.environment === "westend_sepolia"
    ) {
        const l1SwapRouter = context.l1LegacySwapRouter()
        swapCalldata = l1SwapRouter.interface.encodeFunctionData("exactOutputSingle", [
            {
                tokenIn: tokenIn,
                tokenOut: context.l1FeeTokenAddress(),
                fee: swapFee ?? 500, // Stable default to 0.05% pool fee
                recipient: context.l1HandlerAddress(),
                amountOut: amountOut,
                amountInMaximum: amountInMaximum,
                sqrtPriceLimitX96: 0n, // No price limit should be fine as we protect the swap using amountInMaximum
            },
        ])
    } else {
        throw new Error(`Unsupported environment ${registry.environment} for L1 swap router.`)
    }
    return swapCalldata
}

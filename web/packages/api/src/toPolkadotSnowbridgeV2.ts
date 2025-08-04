import { TransferInterface } from "./transfers/toPolkadot/transferInterface"
import { ERC20ToAH } from "./transfers/toPolkadot/erc20ToAH"
import { AssetRegistry } from "@snowbridge/base-types"
export { ValidationKind } from "./toPolkadot_v2"

export type DeliveryFee = {
    assetHubDeliveryFeeEther: bigint
    assetHubExecutionFeeEther: bigint
    destinationDeliveryFeeEther: bigint
    destinationExecutionFeeEther: bigint
    totalFeeInWei: bigint
}

export function createTransferImplementation(
    destinationParaId: number,
    registry: AssetRegistry,
    tokenAddress: string
): TransferInterface {
    const { ahAssetMetadata } = resolveInputs(registry, tokenAddress, destinationParaId)

    let transferImpl: TransferInterface
    // if (destinationParaId == registry.assetHubParaId) {
    //    if (ahAssetMetadata.location) {
    //transferImpl = new PNAFromAH()
    //    } else {
    transferImpl = new ERC20ToAH()
    //    }
    // } else {
    //     if (ahAssetMetadata.location) {
    // transferImpl = new PNAFromParachain()
    //     } else {
    // transferImpl = new ERC20FromParachain()
    //    }
    //}
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

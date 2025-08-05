import { ApiPromise } from "@polkadot/api"
import { AssetRegistry } from "@snowbridge/base-types"
import { TransferInterface } from "./transferInterface"
import { IGatewayV2 as IGateway } from "@snowbridge/contract-types"
import { Context } from "../../index"
import { DeliveryFee } from "../../toPolkadotSnowbridgeV2"
import { accountId32Location, DOT_LOCATION, erc20Location } from "../../xcmBuilder"
import { paraImplementation } from "../../parachains"
import { ETHER_TOKEN_ADDRESS, swapAsset1ForAsset2 } from "../../assets_v2"
import { padFeeByPercentage } from "../../utils"
import { resolveInputs } from "../../toPolkadot_v2"
import {
    buildAssetHubXcm,
    buildParachainERC20ReceivedXcmOnDestination
} from "../../xcmbuilders/toPolkadot/erc20ToParachain"

export class ERC20ToParachain implements TransferInterface {
    async getDeliveryFee(
        context:
            | Context
            | {
                  gateway: IGateway
                  assetHub: ApiPromise
                  bridgeHub: ApiPromise
                  destination: ApiPromise
              },
        registry: AssetRegistry,
        tokenAddress: string,
        destinationParaId: number,
        paddFeeByPercentage?: bigint
    ): Promise<DeliveryFee> {
        const { assetHub, bridgeHub, destination } =
            context instanceof Context
                ? {
                      assetHub: await context.assetHub(),
                      bridgeHub: await context.bridgeHub(),
                      destination: await context.parachain(destinationParaId),
                  }
                : context

        const { destParachain } = resolveInputs(registry, tokenAddress, destinationParaId)

        // AssetHub fees
        let assetHubXcm = buildAssetHubXcm(
            assetHub.registry,
            registry.ethChainId,
            tokenAddress,
            1000000000000n,
            1000000000000n,
            accountId32Location(
                "0x0000000000000000000000000000000000000000000000000000000000000000"
            ),
            "0x0000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000000000000000000000000000"
        )
        console.dir(assetHubXcm.toHuman(), {depth:100})
        const bridgeHubImpl = await paraImplementation(bridgeHub)
        const assetHubImpl = await paraImplementation(assetHub)
        let ether = erc20Location(registry.ethChainId, ETHER_TOKEN_ADDRESS)

        // Delivery fee BridgeHub to AssetHub
        const deliveryFeeInDOT = await bridgeHubImpl.calculateDeliveryFeeInDOT(
            registry.assetHubParaId,
            assetHubXcm
        )
        // AssetHub execution fee
        let assetHubExecutionFeeDOT = await assetHubImpl.calculateXcmFee(assetHubXcm, DOT_LOCATION)
        // Swap to ether
        const deliveryFeeInEther = await swapAsset1ForAsset2(
            assetHub,
            DOT_LOCATION,
            ether,
            deliveryFeeInDOT
        )
        let assetHubExecutionFeeEther = padFeeByPercentage(
            await swapAsset1ForAsset2(assetHub, DOT_LOCATION, ether, assetHubExecutionFeeDOT),
            paddFeeByPercentage ?? 33n
        )

        // Destination fees
        let destinationXcm = buildParachainERC20ReceivedXcmOnDestination(
            destination.registry,
            registry.ethChainId,
            "0x0000000000000000000000000000000000000000",
            3402823669209384634633746074317682114n,
            3402823669209384634633746074317682114n,
            destParachain.info.accountType === "AccountId32"
                ? "0x0000000000000000000000000000000000000000000000000000000000000000"
                : "0x0000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000000000000000000000000000"
        )
        console.dir(destinationXcm.toHuman(), {depth:100})
        const destinationImpl = await paraImplementation(destination)
        // Delivery fee AssetHub to Destination
        let destinationDeliveryFeeDOT = await assetHubImpl.calculateDeliveryFeeInDOT(
            destinationParaId,
            destinationXcm
        )
        // Destination execution fee
        let destinationExecutionFeeDOT = await destinationImpl.calculateXcmFee(
            destinationXcm,
            DOT_LOCATION
        )

        // Swap to ether
        const destinationDeliveryFeeEther = await swapAsset1ForAsset2(
            assetHub,
            DOT_LOCATION,
            ether,
            destinationDeliveryFeeDOT
        )
        let destinationExecutionFeeEther = padFeeByPercentage(
            await swapAsset1ForAsset2(assetHub, DOT_LOCATION, ether, destinationExecutionFeeDOT),
            paddFeeByPercentage ?? 33n
        )

        const totalFeeInWei = deliveryFeeInEther + assetHubExecutionFeeEther
        return {
            assetHubDeliveryFeeEther: deliveryFeeInEther,
            assetHubExecutionFeeEther: assetHubExecutionFeeEther,
            destinationDeliveryFeeEther: destinationDeliveryFeeEther,
            destinationExecutionFeeEther: destinationExecutionFeeEther,
            totalFeeInWei: totalFeeInWei,
        }
    }
}

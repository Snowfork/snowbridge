import { ApiPromise } from "@polkadot/api"
import { AssetRegistry } from "@snowbridge/base-types"
import { TransferInterface } from "./transferInterface"
import { IGatewayV2 as IGateway } from "@snowbridge/contract-types"
import { Context } from "../../index"
import { DeliveryFee } from "../../toPolkadotSnowbridgeV2"
import { buildAssetHubXcm } from "../../xcmbuilders/toPolkadot/erc20ToAH"
import { accountId32Location, DOT_LOCATION, erc20Location } from "../../xcmBuilder"
import { paraImplementation } from "../../parachains"
import { ETHER_TOKEN_ADDRESS, swapAsset1ForAsset2 } from "../../assets_v2"
import { padFeeByPercentage } from "../../utils"

export class ERC20ToAH implements TransferInterface {
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
        _destinationParaId: number,
        paddFeeByPercentage?: bigint
    ): Promise<DeliveryFee> {
        const { assetHub, bridgeHub } =
            context instanceof Context
                ? {
                      assetHub: await context.assetHub(),
                      bridgeHub: await context.bridgeHub(),
                  }
                : context

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
        let ether = erc20Location(registry.ethChainId, ETHER_TOKEN_ADDRESS)

        // Delivery fee BridgeHub to AssetHub
        const bridgeHubImpl = await paraImplementation(bridgeHub)
        const deliveryFeeInDOT = await bridgeHubImpl.calculateDeliveryFeeInDOT(
            registry.assetHubParaId,
            assetHubXcm
        )

        const assetHubImpl = await paraImplementation(assetHub)
        const deliveryFeeInEther = await swapAsset1ForAsset2(
            assetHub,
            DOT_LOCATION,
            ether,
            deliveryFeeInDOT
        )
        // AssetHub Execution fee
        let assetHubExecutionFeeDOT = await assetHubImpl.calculateXcmFee(assetHubXcm, DOT_LOCATION)

        let assetHubExecutionFeeEther = padFeeByPercentage(
            await swapAsset1ForAsset2(assetHub, DOT_LOCATION, ether, assetHubExecutionFeeDOT),
            paddFeeByPercentage ?? 33n
        )

        const totalFeeInWei = deliveryFeeInEther + assetHubExecutionFeeEther
        return {
            assetHubDeliveryFeeEther: deliveryFeeInEther,
            assetHubExecutionFeeEther: assetHubExecutionFeeEther,
            destinationDeliveryFeeEther: 0n,
            destinationExecutionFeeEther: 0n,
            totalFeeInWei: totalFeeInWei,
        }
    }
}

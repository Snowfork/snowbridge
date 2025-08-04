import { ApiPromise } from "@polkadot/api"
import { AssetRegistry } from "@snowbridge/base-types"
import { TransferInterface } from "./transferInterface"
import {
    IGatewayV2 as IGateway,
} from "@snowbridge/contract-types"
import {Context} from "../../index";
import {DeliveryFee} from "../../toPolkadotSnowbridgeV2";
import {resolveInputs} from "../../toPolkadot_v2";

export class ERC20FromAH implements TransferInterface {
    async getDeliveryFee(
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

        // Delivery fee AssetHub to BridgeHub
        let assetHubDeliveryFeeEther = 10000000n;

       // AssetHub Execution fee
        let assetHubExecutionFeeEther = 10000000n;

        const totalFeeInWei = assetHubDeliveryFeeEther + assetHubExecutionFeeEther
        return {
            assetHubDeliveryFeeEther: 0n,
            assetHubExecutionFeeEther: 0n,
            destinationDeliveryFeeEther: 0n,
            destinationExecutionFeeEther: 0n,
            totalFeeInWei: 0n,
        }
    }
}

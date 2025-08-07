import { AssetRegistry } from "@snowbridge/base-types"
import { DeliveryFee } from "../../toPolkadotSnowbridgeV2"
import { Context } from "../../index"
import { IGatewayV2 as IGateway } from "@snowbridge/contract-types"
import { ApiPromise } from "@polkadot/api"
import { Transfer } from "../../toPolkadotSnowbridgeV2"

export interface TransferInterface {
    getDeliveryFee(
        context: Context | { gateway: IGateway; assetHub: ApiPromise; destination: ApiPromise },
        registry: AssetRegistry,
        tokenAddress: string,
        destinationParaId: number,
        paddFeeByPercentage?: bigint
    ): Promise<DeliveryFee>

    createTransfer(
        destination: ApiPromise,
        registry: AssetRegistry,
        destinationParaId: number,
        sourceAccount: string,
        beneficiaryAccount: string,
        tokenAddress: string,
        amount: bigint,
        fee: DeliveryFee
    ): Promise<Transfer>
}

import { AssetRegistry } from "@snowbridge/base-types"
import { DeliveryFee } from "../../toPolkadotSnowbridgeV2"
import { Context } from "../../index"
import { IGatewayV2 as IGateway } from "@snowbridge/contract-types"
import { ApiPromise } from "@polkadot/api"
import { Transfer } from "../../toPolkadotSnowbridgeV2"
import { ValidationResult } from "../../toPolkadotSnowbridgeV2"
import { AbstractProvider } from "ethers"

export interface Connections {
    ethereum: AbstractProvider
    gateway: IGateway
    bridgeHub: ApiPromise
    assetHub: ApiPromise
    destination?: ApiPromise
}

export interface TransferInterface {
    getDeliveryFee(
        context: Context | { gateway: IGateway; assetHub: ApiPromise; destination: ApiPromise },
        registry: AssetRegistry,
        tokenAddress: string,
        destinationParaId: number,
        relayerFee: bigint,
        options?: {
            paddFeeByPercentage?: bigint
            feeAsset?: any
        }
    ): Promise<DeliveryFee>

    createTransfer(
        context: Context | { assetHub: ApiPromise; destination: ApiPromise | undefined },
        registry: AssetRegistry,
        destinationParaId: number,
        sourceAccount: string,
        beneficiaryAccount: string,
        tokenAddress: string,
        amount: bigint,
        fee: DeliveryFee
    ): Promise<Transfer>

    validateTransfer(context: Context | Connections, transfer: Transfer): Promise<ValidationResult>
}

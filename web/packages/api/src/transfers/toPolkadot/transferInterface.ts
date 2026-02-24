import { AssetRegistry } from "@snowbridge/base-types"
import { DeliveryFee } from "../../toPolkadotSnowbridgeV2"
import { EthersContext } from "../../index"
import { Transfer } from "../../toPolkadotSnowbridgeV2"
import { ValidationResult } from "../../toPolkadotSnowbridgeV2"

export interface TransferInterface {
    getDeliveryFee(
        context: EthersContext,
        registry: AssetRegistry,
        tokenAddress: string,
        destinationParaId: number,
        options?: {
            paddFeeByPercentage?: bigint
            feeAsset?: any
            customXcm?: any[] // Optional custom XCM instructions to append
            overrideRelayerFee?: bigint
        },
    ): Promise<DeliveryFee>

    createTransfer(
        context: EthersContext,
        registry: AssetRegistry,
        destinationParaId: number,
        sourceAccount: string,
        beneficiaryAccount: string,
        tokenAddress: string,
        amount: bigint,
        fee: DeliveryFee,
        customXcm?: any[], // Optional custom XCM instructions to append
    ): Promise<Transfer>

    validateTransfer(context: EthersContext, transfer: Transfer): Promise<ValidationResult>
}

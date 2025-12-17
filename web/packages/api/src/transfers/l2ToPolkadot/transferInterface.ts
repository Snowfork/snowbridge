import { AssetRegistry } from "@snowbridge/base-types"
import { Context } from "../../index"
import { DeliveryFee, Transfer, ValidationResult } from "../../toPolkadotSnowbridgeV2"

export interface TransferInterface {
    getDeliveryFee(
        context: Context,
        registry: AssetRegistry,
        l2TokenAddress: string,
        tokenAddress: string,
        amount: bigint,
        destinationParaId: number,
        options?: {
            paddFeeByPercentage?: bigint
            feeAsset?: any
            customXcm?: any[] // Optional custom XCM instructions to append
            overrideRelayerFee?: bigint
            l2PadFeeByPercentage?: bigint
        },
    ): Promise<DeliveryFee>

    createTransfer(
        context: Context,
        registry: AssetRegistry,
        l2TokenAddress: string,
        tokenAddress: string,
        amount: bigint,
        destinationParaId: number,
        sourceAccount: string,
        beneficiaryAccount: string,
        fee: DeliveryFee,
        customXcm?: any[], // Optional custom XCM instructions to append
    ): Promise<Transfer>

    validateTransfer(context: Context, transfer: Transfer): Promise<ValidationResult>
}

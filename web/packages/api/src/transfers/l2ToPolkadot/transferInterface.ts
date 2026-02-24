import { AssetRegistry } from "@snowbridge/base-types"
import { EthersContext } from "../../index"
import { DeliveryFee, Transfer, ValidationResult } from "../../toPolkadotSnowbridgeV2"

export interface TransferInterface {
    getDeliveryFee(
        context: EthersContext,
        registry: AssetRegistry,
        l2ChainId: number,
        tokenAddress: string,
        amount: bigint,
        destinationParaId: number,
        options?: {
            paddFeeByPercentage?: bigint
            feeAsset?: any
            customXcm?: any[] // Optional custom XCM instructions to append
            overrideRelayerFee?: bigint
            l2PadFeeByPercentage?: bigint
            fillDeadlineBuffer?: bigint // Optional buffer added to the relay fill deadline for L2 calls.
        },
    ): Promise<DeliveryFee>

    createTransfer(
        context: EthersContext,
        registry: AssetRegistry,
        l2ChainId: number,
        tokenAddress: string,
        amount: bigint,
        destinationParaId: number,
        sourceAccount: string,
        beneficiaryAccount: string,
        fee: DeliveryFee,
        options?: {
            customXcm?: any[] // Optional custom XCM instructions to append
            fillDeadlineBuffer?: bigint // Optional buffer added to the relay fill deadline for L2 calls.
        },
    ): Promise<Transfer>

    validateTransfer(context: EthersContext, transfer: Transfer): Promise<ValidationResult>
}

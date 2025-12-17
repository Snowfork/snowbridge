import { AssetRegistry, ContractCall } from "@snowbridge/base-types"
import { DeliveryFee, Transfer, ValidationResult } from "../../toEthereum_v2"
import { Context } from "../../index"

export interface TransferInterface {
    getDeliveryFee(
        context: Context,
        registry: AssetRegistry,
        tokenAddress: string,
        l2TokenAddress: string,
        options?: {
            padPercentage?: bigint
            slippagePadPercentage?: bigint
            defaultFee?: bigint
            feeTokenLocation?: any
            claimerLocation?: any
            contractCall?: ContractCall
            l2PadFeeByPercentage?: bigint
        },
    ): Promise<DeliveryFee>

    createTransfer(
        context: Context,
        registry: AssetRegistry,
        sourceAccount: string,
        beneficiaryAccount: string,
        tokenAddress: string,
        l2TokenAddress: string,
        amount: bigint,
        fee: DeliveryFee,
        options?: {
            claimerLocation?: any
            contractCall?: ContractCall
        },
    ): Promise<Transfer>

    validateTransfer(context: Context, transfer: Transfer): Promise<ValidationResult>
}

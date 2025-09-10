import { AssetRegistry } from "@snowbridge/base-types"
import { DeliveryFee, Transfer, ValidationResult } from "../../toEthereum_v2"
import { Context } from "../../index"

export interface TransferInterface {
    getDeliveryFee(
        source: { sourceParaId: number; context: Context },
        registry: AssetRegistry,
        tokenAddress: string,
        options?: {
            padPercentage?: bigint
            slippagePadPercentage?: bigint
            defaultFee?: bigint
            feeTokenLocation?: any
            claimerLocation?: any
            accelerated?: boolean
        }
    ): Promise<DeliveryFee>

    createTransfer(
        source: { sourceParaId: number; context: Context },
        registry: AssetRegistry,
        sourceAccount: string,
        beneficiaryAccount: string,
        tokenAddress: string,
        amount: bigint,
        fee: DeliveryFee,
        claimerLocation?: any
    ): Promise<Transfer>

    validateTransfer(context: Context, transfer: Transfer): Promise<ValidationResult>
}

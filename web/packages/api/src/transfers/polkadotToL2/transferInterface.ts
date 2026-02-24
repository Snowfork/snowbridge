import { AssetRegistry, ContractCall } from "@snowbridge/base-types"
import { DeliveryFee, Transfer, ValidationResult } from "../../toEthereum_v2"
import { EthersContext } from "../../index"

export interface TransferInterface {
    getDeliveryFee(
        context: EthersContext,
        registry: AssetRegistry,
        l2ChainId: number,
        tokenAddress: string,
        amount: bigint,
        options?: {
            padPercentage?: bigint
            slippagePadPercentage?: bigint
            defaultFee?: bigint
            feeTokenLocation?: any
            claimerLocation?: any
            contractCall?: ContractCall
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
        sourceAccount: string,
        beneficiaryAccount: string,
        fee: DeliveryFee,
        options?: {
            claimerLocation?: any
            contractCall?: ContractCall
            fillDeadlineBuffer?: bigint
        },
    ): Promise<Transfer>

    validateTransfer(context: EthersContext, transfer: Transfer): Promise<ValidationResult>
}

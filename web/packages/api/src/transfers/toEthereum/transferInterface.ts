import { AddressOrPair, SignerOptions } from "@polkadot/api/types"
import { ContractCall } from "@snowbridge/base-types"
import { DeliveryFee, MessageReceipt, Transfer, ValidationResult } from "../../toEthereum_v2"

export interface TransferInterface {
    getDeliveryFee(
        tokenAddress: string,
        options?: {
            padPercentage?: bigint
            slippagePadPercentage?: bigint
            defaultFee?: bigint
            feeTokenLocation?: any
            claimerLocation?: any
            contractCall?: ContractCall
        },
    ): Promise<DeliveryFee>

    createTransfer(
        sourceAccount: string,
        beneficiaryAccount: string,
        tokenAddress: string,
        amount: bigint,
        fee: DeliveryFee,
        options?: {
            claimerLocation?: any
            contractCall?: ContractCall
        },
    ): Promise<Transfer>

    validateTransfer(transfer: Transfer): Promise<ValidationResult>

    signAndSend(
        transfer: Transfer,
        account: AddressOrPair,
        options: Partial<SignerOptions>,
    ): Promise<MessageReceipt>
}

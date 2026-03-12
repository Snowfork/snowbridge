import { Context } from "../.."
import { AddressOrPair, SignerOptions } from "@polkadot/api/types"
import { ContractCall, EthereumProviderTypes } from "@snowbridge/base-types"
import { DeliveryFee, MessageReceipt, Transfer, ValidationResult } from "../../toEthereum_v2"

export interface TransferInterface<T extends EthereumProviderTypes> {
    readonly context: Context<T>

    fee(
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

    rawTx(
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

    validate(transfer: Transfer): Promise<ValidationResult>

    signAndSend(
        transfer: Transfer,
        account: AddressOrPair,
        options: Partial<SignerOptions>,
    ): Promise<MessageReceipt>
}

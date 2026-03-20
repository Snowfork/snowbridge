import { Context } from "../.."
import { AddressOrPair, SignerOptions } from "@polkadot/api/types"
import { ContractCall, EthereumProviderTypes } from "@snowbridge/base-types"
import type {
    DeliveryFee,
    MessageReceipt,
    Transfer,
    ValidatedTransfer,
} from "../../types/toEthereum"

export interface TransferInterface<T extends EthereumProviderTypes> {
    readonly context: Context<T>

    fee(
        tokenAddress: string,
        options?: {
            padFeeByPercentage?: bigint
            slippagePadPercentage?: bigint
            defaultFee?: bigint
            feeTokenLocation?: any
            claimerLocation?: any
            contractCall?: ContractCall
        },
    ): Promise<DeliveryFee>

    tx(
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

    validate(transfer: Transfer): Promise<ValidatedTransfer>

    build(
        sourceAccount: string,
        beneficiaryAccount: string,
        tokenAddress: string,
        amount: bigint,
        options?: {
            fee?: {
                padFeeByPercentage?: bigint
                slippagePadPercentage?: bigint
                defaultFee?: bigint
                feeTokenLocation?: any
                claimerLocation?: any
                contractCall?: ContractCall
            }
            tx?: {
                claimerLocation?: any
                contractCall?: ContractCall
            }
        },
    ): Promise<ValidatedTransfer>

    signAndSend(
        transfer: Transfer,
        account: AddressOrPair,
        options: Partial<SignerOptions>,
    ): Promise<MessageReceipt>
}

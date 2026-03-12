import { Context } from "../.."
import { AddressOrPair, SignerOptions } from "@polkadot/api/types"
import { DeliveryFee, MessageReceipt, Transfer, ValidationResult } from "../../forInterParachain"
import { EthereumProviderTypes } from "@snowbridge/base-types"

export interface TransferInterface<T extends EthereumProviderTypes> {
    readonly context: Context<T>

    fee(
        tokenAddress: string,
        options?: {
            padPercentage?: bigint
        },
    ): Promise<DeliveryFee>

    rawTx(
        sourceAccount: string,
        beneficiaryAccount: string,
        tokenAddress: string,
        amount: bigint,
        fee: DeliveryFee,
    ): Promise<Transfer>

    validate(transfer: Transfer): Promise<ValidationResult>

    signAndSend(
        transfer: Transfer,
        account: AddressOrPair,
        options: Partial<SignerOptions>,
    ): Promise<MessageReceipt>
}

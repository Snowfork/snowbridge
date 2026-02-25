import { AddressOrPair, SignerOptions } from "@polkadot/api/types"
import { DeliveryFee, MessageReceipt, Transfer, ValidationResult } from "../../forKusama"

export interface TransferInterface {
    getDeliveryFee(tokenAddress: string): Promise<DeliveryFee>

    createTransfer(
        sourceAccount: string,
        beneficiaryAccount: string,
        tokenAddress: string,
        amount: bigint,
        fee: DeliveryFee,
    ): Promise<Transfer>

    validateTransfer(transfer: Transfer): Promise<ValidationResult>

    signAndSend(
        transfer: Transfer,
        account: AddressOrPair,
        options: Partial<SignerOptions>,
    ): Promise<MessageReceipt>
}

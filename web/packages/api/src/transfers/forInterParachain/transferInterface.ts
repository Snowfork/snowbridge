import { AddressOrPair, SignerOptions } from "@polkadot/api/types"
import { DeliveryFee, MessageReceipt, Transfer, ValidationResult } from "../../forInterParachain"

export interface TransferInterface {
    getDeliveryFee(
        tokenAddress: string,
        options?: {
            padPercentage?: bigint
        },
    ): Promise<DeliveryFee>

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

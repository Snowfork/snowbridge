import { TransactionReceipt } from "ethers"
import {
    DeliveryFee,
    MessageReceipt,
    Transfer,
    ValidationResult,
} from "../../toPolkadotSnowbridgeV2"

export interface TransferInterface {
    getDeliveryFee(
        tokenAddress: string,
        amount: bigint,
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
        tokenAddress: string,
        amount: bigint,
        sourceAccount: string,
        beneficiaryAccount: string,
        fee: DeliveryFee,
        options?: {
            customXcm?: any[] // Optional custom XCM instructions to append
            fillDeadlineBuffer?: bigint // Optional buffer added to the relay fill deadline for L2 calls.
        },
    ): Promise<Transfer>

    validateTransfer(transfer: Transfer): Promise<ValidationResult>

    getMessageReceipt(receipt: TransactionReceipt): Promise<MessageReceipt | null>
}

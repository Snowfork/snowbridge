import { AddressOrPair, SignerOptions } from "@polkadot/api/types"
import { AssetRegistry } from "@snowbridge/base-types"
import {
    DeliveryFee,
    MessageReceipt,
    Transfer,
    ValidationResult,
} from "../../forInterParachain"
import { EthersContext } from "../../index"

export interface TransferInterface {
    getDeliveryFee(
        connections: { context: EthersContext; sourceParaId: number; destinationParaId: number },
        registry: AssetRegistry,
        tokenAddress: string,
        options?: {
            padPercentage?: bigint
        },
    ): Promise<DeliveryFee>

    createTransfer(
        connections: { context: EthersContext; sourceParaId: number },
        registry: AssetRegistry,
        sourceAccount: string,
        beneficiaryAccount: string,
        destinationParaId: number,
        tokenAddress: string,
        amount: bigint,
        fee: DeliveryFee,
    ): Promise<Transfer>

    validateTransfer(
        connections: { context: EthersContext; sourceParaId: number; destinationParaId: number },
        transfer: Transfer,
    ): Promise<ValidationResult>

    signAndSend(
        connections: { context: EthersContext; sourceParaId: number },
        transfer: Transfer,
        account: AddressOrPair,
        options: Partial<SignerOptions>,
    ): Promise<MessageReceipt>
}

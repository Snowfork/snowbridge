import { AddressOrPair, SignerOptions } from "@polkadot/api/types"
import { AssetRegistry, ContractCall } from "@snowbridge/base-types"
import { DeliveryFee, MessageReceipt, Transfer, ValidationResult } from "../../toEthereum_v2"
import { EthersContext } from "../../index"

export interface TransferInterface {
    getDeliveryFee(
        source: { sourceParaId: number; context: EthersContext },
        registry: AssetRegistry,
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
        source: { sourceParaId: number; context: EthersContext },
        registry: AssetRegistry,
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

    validateTransfer(context: EthersContext, transfer: Transfer): Promise<ValidationResult>

    signAndSend(
        context: EthersContext,
        transfer: Transfer,
        account: AddressOrPair,
        options: Partial<SignerOptions>,
    ): Promise<MessageReceipt>
}

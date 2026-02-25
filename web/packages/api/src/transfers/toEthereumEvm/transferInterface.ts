import { AssetRegistry, ContractCall } from "@snowbridge/base-types"
import { DeliveryFee } from "../../toEthereum_v2"
import { TransferEvm, ValidationResultEvm } from "../../toEthereumFromEVM_v2"
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
    ): Promise<TransferEvm>

    validateTransfer(context: EthersContext, transfer: TransferEvm): Promise<ValidationResultEvm>
}

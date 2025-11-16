import { AssetRegistry, ContractCall } from "@snowbridge/base-types"
import { DeliveryFeeV2, TransferV2, ValidationResultV2 } from "../../toEthereumSnowbridgeV2"
import { Context } from "../../index"
import { ConcreteToken } from "../../assets_v2"

export interface TransferInterface {
    getDeliveryFee(
        context: Context,
        sourceParaId: number,
        registry: AssetRegistry,
        tokenAddresses: string[],
        options?: {
            padPercentage?: bigint
            slippagePadPercentage?: bigint
            defaultFee?: bigint
            feeTokenLocation?: any
            claimerLocation?: any
            contractCall?: ContractCall
        },
    ): Promise<DeliveryFeeV2>

    createTransfer(
        context: Context,
        sourceParaId: number,
        registry: AssetRegistry,
        sourceAccount: string,
        beneficiaryAccount: string,
        tokens: ConcreteToken[],
        fee: DeliveryFeeV2,
        options?: {
            claimerLocation?: any
            contractCall?: ContractCall
        },
    ): Promise<TransferV2>

    validateTransfer(context: Context, transfer: TransferV2): Promise<ValidationResultV2>
}

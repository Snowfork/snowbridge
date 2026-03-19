import { AssetRegistry, EthereumProviderTypes } from "@snowbridge/base-types"
import { OperationStatus } from "../../status"
import { FeeInfo, ValidationLog } from "../../toPolkadot_v2"
import type { MessageReceipt } from "../../toPolkadotSnowbridgeV2"

export type TokenRegistration<T extends EthereumProviderTypes> = {
    input: {
        registry: AssetRegistry
        sourceAccount: string
        tokenAddress: string
        fee: RegistrationFee
    }
    computed: {
        gatewayAddress: string
        totalValue: bigint
    }
    tx: T["ContractTransaction"]
}

export type ValidatedRegisterToken<T extends EthereumProviderTypes> = TokenRegistration<T> & {
    logs: ValidationLog[]
    success: boolean
    data: {
        etherBalance: bigint
        feeInfo?: FeeInfo
        bridgeStatus: OperationStatus
        isTokenAlreadyRegistered: boolean
        assetHubDryRunError?: string
    }
}

export type RegistrationFee = {
    assetHubDeliveryFeeEther: bigint
    assetHubExecutionFeeEther: bigint
    assetDepositEther: bigint
    assetDepositDOT: bigint
    relayerFee: bigint
    totalFeeInWei: bigint
}

export interface RegistrationInterface<T extends EthereumProviderTypes> {
    fee(
        relayerFee: bigint,
        options?: {
            padFeeByPercentage?: bigint
        },
    ): Promise<RegistrationFee>

    tx(
        sourceAccount: string,
        tokenAddress: string,
        fee: RegistrationFee,
    ): Promise<TokenRegistration<T>>

    validate(registration: TokenRegistration<T>): Promise<ValidatedRegisterToken<T>>

    build(
        sourceAccount: string,
        tokenAddress: string,
        relayerFee: bigint,
        options?: {
            padFeeByPercentage?: bigint
        },
    ): Promise<ValidatedRegisterToken<T>>

    messageId(receipt: T["TransactionReceipt"]): Promise<MessageReceipt | null>
}

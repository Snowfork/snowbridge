import { AssetRegistry, EthereumProviderTypes } from "@snowbridge/base-types"
import { Context } from "../../index"
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
        context: Context<T>,
        registry: AssetRegistry,
        relayerFee: bigint,
        options?: {
            paddFeeByPercentage?: bigint
        },
    ): Promise<RegistrationFee>

    tx(
        context: Context<T>,
        registry: AssetRegistry,
        sourceAccount: string,
        tokenAddress: string,
        fee: RegistrationFee,
    ): Promise<TokenRegistration<T>>

    validate(
        context: Context<T>,
        registration: TokenRegistration<T>,
    ): Promise<ValidatedRegisterToken<T>>

    build(
        context: Context<T>,
        registry: AssetRegistry,
        sourceAccount: string,
        tokenAddress: string,
        relayerFee: bigint,
        options?: {
            paddFeeByPercentage?: bigint
        },
    ): Promise<ValidatedRegisterToken<T>>

    messageId(context: Context<T>, receipt: T["TransactionReceipt"]): Promise<MessageReceipt | null>
}

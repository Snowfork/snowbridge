import { AssetRegistry } from "@snowbridge/base-types"
import { Context, EthereumProviderTypes } from "../../index"
import { OperationStatus } from "../../status"
import { FeeInfo, ValidationLog } from "../../toPolkadot_v2"
import type { MessageReceipt } from "../../toPolkadotSnowbridgeV2"

export type TokenRegistration<T extends EthereumProviderTypes = EthereumProviderTypes> = {
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

export type RegistrationValidationResult<T extends EthereumProviderTypes = EthereumProviderTypes> =
    {
        logs: ValidationLog[]
        success: boolean
        data: {
            etherBalance: bigint
            feeInfo?: FeeInfo
            bridgeStatus: OperationStatus
            isTokenAlreadyRegistered: boolean
            assetHubDryRunError?: string
        }
        registration: TokenRegistration<T>
    }

export type RegistrationFee = {
    assetHubDeliveryFeeEther: bigint
    assetHubExecutionFeeEther: bigint
    assetDepositEther: bigint
    assetDepositDOT: bigint
    relayerFee: bigint
    totalFeeInWei: bigint
}

export interface RegistrationInterface<T extends EthereumProviderTypes = EthereumProviderTypes> {
    getRegistrationFee(
        context: Context<T>,
        registry: AssetRegistry,
        relayerFee: bigint,
        options?: {
            paddFeeByPercentage?: bigint
        },
    ): Promise<RegistrationFee>

    createRegistration(
        context: Context<T>,
        registry: AssetRegistry,
        sourceAccount: string,
        tokenAddress: string,
        fee: RegistrationFee,
    ): Promise<TokenRegistration<T>>

    validateRegistration(
        context: Context<T>,
        registration: TokenRegistration<T>,
    ): Promise<RegistrationValidationResult<T>>

    getMessageReceipt(
        context: Context<T>,
        receipt: T["TransactionReceipt"],
    ): Promise<MessageReceipt | null>
}

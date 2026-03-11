import { AssetRegistry } from "@snowbridge/base-types"
import { Context, EthersProviderTypes } from "../../index"
import { ContractTransaction, TransactionReceipt } from "ethers"
import { OperationStatus } from "../../status"
import { FeeInfo, ValidationLog } from "../../toPolkadot_v2"
import type { MessageReceipt } from "../../toPolkadotSnowbridgeV2"

export type TokenRegistration = {
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
    tx: ContractTransaction
}

export type RegistrationValidationResult = {
    logs: ValidationLog[]
    success: boolean
    data: {
        etherBalance: bigint
        feeInfo?: FeeInfo
        bridgeStatus: OperationStatus
        isTokenAlreadyRegistered: boolean
        assetHubDryRunError?: string
    }
    registration: TokenRegistration
}

export type RegistrationFee = {
    assetHubDeliveryFeeEther: bigint
    assetHubExecutionFeeEther: bigint
    assetDepositEther: bigint
    assetDepositDOT: bigint
    relayerFee: bigint
    totalFeeInWei: bigint
}

export interface RegistrationInterface {
    getRegistrationFee(
        context: Context<EthersProviderTypes>,
        registry: AssetRegistry,
        relayerFee: bigint,
        options?: {
            paddFeeByPercentage?: bigint
        },
    ): Promise<RegistrationFee>

    createRegistration(
        context: Context<EthersProviderTypes>,
        registry: AssetRegistry,
        sourceAccount: string,
        tokenAddress: string,
        fee: RegistrationFee,
    ): Promise<TokenRegistration>

    validateRegistration(
        context: Context<EthersProviderTypes>,
        registration: TokenRegistration,
    ): Promise<RegistrationValidationResult>

    getMessageReceipt(
        context: Context<EthersProviderTypes>,
        receipt: TransactionReceipt,
    ): Promise<MessageReceipt | null>
}

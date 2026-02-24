import { AssetRegistry } from "@snowbridge/base-types"
import { Context } from "../../index"
import { ContractTransaction } from "ethers"
import { OperationStatus } from "../../status"
import { FeeInfo, ValidationLog } from "../../toPolkadot_v2"

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
        context: Context,
        registry: AssetRegistry,
        relayerFee: bigint,
        options?: {
            paddFeeByPercentage?: bigint
        },
    ): Promise<RegistrationFee>

    createRegistration(
        context: Context,
        registry: AssetRegistry,
        sourceAccount: string,
        tokenAddress: string,
        fee: RegistrationFee,
    ): Promise<TokenRegistration>

    validateRegistration(
        context: Context,
        registration: TokenRegistration,
    ): Promise<RegistrationValidationResult>
}

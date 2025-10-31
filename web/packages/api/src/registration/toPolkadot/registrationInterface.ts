import { AssetRegistry, ERC20Metadata } from "@snowbridge/base-types"
import { Context } from "../../index"
import { IGatewayV2 as IGateway } from "@snowbridge/contract-types"
import { ApiPromise } from "@polkadot/api"
import { AbstractProvider, ContractTransaction } from "ethers"
import { OperationStatus } from "../../status"
import { FeeInfo, ValidationLog } from "../../toPolkadot_v2"

export interface Connections {
    ethereum: AbstractProvider
    gateway: IGateway
    bridgeHub: ApiPromise
    assetHub: ApiPromise
}

export type TokenRegistration = {
    input: {
        registry: AssetRegistry
        sourceAccount: string
        tokenAddress: string
        assetHubDeliveryFeeEther: bigint
        assetHubExecutionFeeEther: bigint
        relayerFee: bigint
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
    relayerFee: bigint
    totalFeeInWei: bigint
}

export interface RegistrationInterface {
    getRegistrationFee(
        context:
            | Context
            | {
                  assetHub: ApiPromise
                  bridgeHub: ApiPromise
              },
        registry: AssetRegistry,
        relayerFee: bigint,
        options?: {
            paddFeeByPercentage?: bigint
        }
    ): Promise<RegistrationFee>

    createRegistration(
        context:
            | Context
            | {
                  ethereum: AbstractProvider
              },
        registry: AssetRegistry,
        sourceAccount: string,
        tokenAddress: string,
        fee: RegistrationFee
    ): Promise<TokenRegistration>

    validateRegistration(
        context: Context | Connections,
        registration: TokenRegistration
    ): Promise<RegistrationValidationResult>
}

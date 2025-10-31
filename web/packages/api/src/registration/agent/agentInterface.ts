import { AssetRegistry } from "@snowbridge/base-types"
import { Context } from "../../index"
import { IGatewayV2 as IGateway } from "@snowbridge/contract-types"
import { AbstractProvider, ContractTransaction } from "ethers"
import { OperationStatus } from "../../status"
import { FeeInfo, ValidationLog } from "../../toPolkadot_v2"

export interface AgentConnections {
    ethereum: AbstractProvider
    gateway: IGateway
}

export type AgentCreation = {
    input: {
        registry: AssetRegistry
        sourceAccount: string
        agentId: string
    }
    computed: {
        gatewayAddress: string
    }
    tx: ContractTransaction
}

export type AgentCreationValidationResult = {
    logs: ValidationLog[]
    success: boolean
    data: {
        etherBalance: bigint
        feeInfo?: FeeInfo
        agentAlreadyExists: boolean
        agentAddress?: string
    }
    creation: AgentCreation
}

export interface AgentCreationInterface {
    createAgentCreation(
        context:
            | Context
            | {
                  ethereum: AbstractProvider
              },
        registry: AssetRegistry,
        sourceAccount: string,
        agentId: string
    ): Promise<AgentCreation>

    validateAgentCreation(
        context: Context | AgentConnections,
        creation: AgentCreation
    ): Promise<AgentCreationValidationResult>
}

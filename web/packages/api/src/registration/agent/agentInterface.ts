import { AssetRegistry } from "@snowbridge/base-types"
import { EthersContext } from "../../index"
import { ContractTransaction } from "ethers"
import { FeeInfo, ValidationLog } from "../../toPolkadot_v2"

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
        context: EthersContext,
        registry: AssetRegistry,
        sourceAccount: string,
        agentId: string,
    ): Promise<AgentCreation>

    validateAgentCreation(
        context: EthersContext,
        creation: AgentCreation,
    ): Promise<AgentCreationValidationResult>
}

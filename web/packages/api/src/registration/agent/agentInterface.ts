import { FeeInfo, ValidationLog } from "../../toPolkadot_v2"

export type AgentCreation<ContractTransaction> = {
    input: {
        sourceAccount: string
        agentId: string
    }
    computed: {
        gatewayAddress: string
    }
    tx: ContractTransaction
}

export type AgentCreationValidationResult<ContractTransaction> = {
    logs: ValidationLog[]
    success: boolean
    data: {
        etherBalance: bigint
        feeInfo?: FeeInfo
        agentAlreadyExists: boolean
        agentAddress?: string
    }
    creation: AgentCreation<ContractTransaction>
}

export interface AgentCreationInterface<ContractTransaction> {
    rawTx(sourceAccount: string, agentId: string): Promise<AgentCreation<ContractTransaction>>

    validateTx(
        creation: AgentCreation<ContractTransaction>,
    ): Promise<AgentCreationValidationResult<ContractTransaction>>

    tx(
        sourceAccount: string,
        agentId: string,
    ): Promise<AgentCreationValidationResult<ContractTransaction>>
}

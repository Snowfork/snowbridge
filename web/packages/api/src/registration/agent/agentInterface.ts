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

export interface AgentCreationInterface<Context, ContractTransaction> {
    readonly context: Context

    rawTx(sourceAccount: string, agentId: string): Promise<AgentCreation<ContractTransaction>>

    validateTx(
        creation: AgentCreation<ContractTransaction>,
    ): Promise<AgentCreationValidationResult<ContractTransaction>>

    tx(creation: AgentCreation<ContractTransaction>): Promise<AgentCreation<ContractTransaction>>
}

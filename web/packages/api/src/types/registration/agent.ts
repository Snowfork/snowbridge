import type { FeeInfo, ValidationLog } from "../toPolkadot"

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

export type ValidatedCreateAgent<ContractTransaction> = AgentCreation<ContractTransaction> & {
    logs: ValidationLog[]
    success: boolean
    data: {
        etherBalance: bigint
        feeInfo?: FeeInfo
        agentAlreadyExists: boolean
        agentAddress?: string
    }
}

export interface AgentCreationInterface<ContractTransaction> {
    agentIdForAccount(parachainId: number, account: string): Promise<string>

    tx(sourceAccount: string, agentId: string): Promise<AgentCreation<ContractTransaction>>

    validate(
        creation: AgentCreation<ContractTransaction>,
    ): Promise<ValidatedCreateAgent<ContractTransaction>>

    build(
        sourceAccount: string,
        agentId: string,
    ): Promise<ValidatedCreateAgent<ContractTransaction>>
}

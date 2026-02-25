import { AssetRegistry } from "@snowbridge/base-types"
import {
    AgentCreationInterface,
    AgentCreation,
    AgentCreationValidationResult,
} from "./agentInterface"
import { EthersContext } from "../../index"
import { ValidationKind } from "../../toPolkadotSnowbridgeV2"
import { ValidationLog, ValidationReason } from "../../toPolkadot_v2"

export class CreateAgent implements AgentCreationInterface {
    async createAgentCreation(
        context: EthersContext,
        registry: AssetRegistry,
        sourceAccount: string,
        agentId: string,
    ): Promise<AgentCreation> {
        const con = context.gatewayV2()

        const tx = await context.ethereumProvider.populateTransaction(
            con,
            "v2_createAgent",
            agentId,
            {
                from: sourceAccount,
            },
        )

        return {
            input: {
                registry,
                sourceAccount,
                agentId,
            },
            computed: {
                gatewayAddress: registry.gatewayAddress,
            },
            tx,
        }
    }

    async validateAgentCreation(
        context: EthersContext,
        creation: AgentCreation,
    ): Promise<AgentCreationValidationResult> {
        const { tx } = creation
        const { sourceAccount, agentId } = creation.input
        const ethereum = context.ethereum()
        const gateway = context.gatewayV2()

        const logs: ValidationLog[] = []

        // Check if agent already exists
        let agentAlreadyExists = false
        let existingAgent: string | undefined
        try {
            existingAgent = await gateway.agentOf(agentId)
            agentAlreadyExists = existingAgent !== "0x0000000000000000000000000000000000000000"
            if (agentAlreadyExists) {
                logs.push({
                    kind: ValidationKind.Error,
                    reason: ValidationReason.MinimumAmountValidation,
                    message: `Agent with ID ${agentId} already exists at address ${existingAgent}.`,
                })
            }
        } catch {
            agentAlreadyExists = false
        }

        const etherBalance = await ethereum.getBalance(sourceAccount)

        let feeInfo
        if (logs.length === 0 || !agentAlreadyExists) {
            const [estimatedGas, feeData] = await Promise.all([
                ethereum.estimateGas(tx),
                ethereum.getFeeData(),
            ])
            const executionFee = (feeData.gasPrice ?? 0n) * estimatedGas
            if (executionFee === 0n) {
                logs.push({
                    kind: ValidationKind.Error,
                    reason: ValidationReason.FeeEstimationError,
                    message: "Could not fetch fee details.",
                })
            }
            if (etherBalance < executionFee) {
                logs.push({
                    kind: ValidationKind.Error,
                    reason: ValidationReason.InsufficientEther,
                    message: "Insufficient ether to submit transaction.",
                })
            }
            feeInfo = {
                estimatedGas,
                feeData,
                executionFee,
                totalTxCost: executionFee,
            }
        }

        const success = logs.find((l) => l.kind === ValidationKind.Error) === undefined

        return {
            logs,
            success,
            data: {
                etherBalance,
                feeInfo,
                agentAlreadyExists,
                agentAddress: agentAlreadyExists ? existingAgent : undefined,
            },
            creation,
        }
    }
}

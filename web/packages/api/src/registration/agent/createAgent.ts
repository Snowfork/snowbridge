import { AssetRegistry } from "@snowbridge/base-types"
import {
    AgentConnections,
    AgentCreationInterface,
    AgentCreation,
    AgentCreationValidationResult,
} from "./agentInterface"
import { IGatewayV2__factory as IGateway__factory } from "../../contracts"
import { Context } from "../../index"
import { ValidationKind } from "../../toPolkadotSnowbridgeV2"
import { ValidationLog, ValidationReason } from "../../toPolkadot_v2"
import { AbstractProvider, Contract } from "ethers"

export class CreateAgent implements AgentCreationInterface {
    async createAgentCreation(
        context:
            | Context
            | {
                  ethereum: AbstractProvider
              },
        registry: AssetRegistry,
        sourceAccount: string,
        agentId: string,
    ): Promise<AgentCreation> {
        const ifce = IGateway__factory.createInterface()
        const con = new Contract(registry.gatewayAddress, ifce)

        const tx = await con.getFunction("v2_createAgent").populateTransaction(agentId, {
            from: sourceAccount,
        })

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
        context: Context | AgentConnections,
        creation: AgentCreation,
    ): Promise<AgentCreationValidationResult> {
        const { tx } = creation
        const { sourceAccount, agentId } = creation.input
        const { ethereum, gateway } =
            context instanceof Context
                ? {
                      ethereum: context.ethereum(),
                      gateway: context.gatewayV2(),
                  }
                : context

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

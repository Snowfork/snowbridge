import {
    AgentCreationInterface,
    AgentCreation,
    AgentCreationValidationResult,
} from "./agentInterface"
import type { Context } from "../../index"
import { ValidationKind } from "../../toPolkadotSnowbridgeV2"
import { ValidationLog, ValidationReason } from "../../toPolkadot_v2"
import { AssetRegistry } from "@snowbridge/base-types"

export class CreateAgent<
    EConnection,
    EContract,
    EAbi,
    EInterface,
    ETransactionReceipt,
    EContractTransaction,
> implements
        AgentCreationInterface<
            Context<
                EConnection,
                EContract,
                EAbi,
                EInterface,
                ETransactionReceipt,
                EContractTransaction
            >,
            EContractTransaction
        >
{
    constructor(
        readonly context: Context<
            EConnection,
            EContract,
            EAbi,
            EInterface,
            ETransactionReceipt,
            EContractTransaction
        >,
        private readonly registry: AssetRegistry,
    ) {}

    async rawTx(
        sourceAccount: string,
        agentId: string,
    ): Promise<AgentCreation<EContractTransaction>> {
        const con = this.context.gatewayV2()

        const tx = await this.context.ethereumProvider.populateTransaction(
            con,
            "v2_createAgent",
            agentId,
            {
                from: sourceAccount,
            },
        )

        return {
            input: {
                sourceAccount,
                agentId,
            },
            computed: {
                gatewayAddress: this.registry.gatewayAddress,
            },
            tx,
        }
    }

    async validateTx(
        creation: AgentCreation<EContractTransaction>,
    ): Promise<AgentCreationValidationResult<EContractTransaction>> {
        const { tx } = creation
        const { sourceAccount, agentId } = creation.input
        const ethereum = this.context.ethereum()
        const gateway = this.context.gatewayV2()

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

        const etherBalance = await this.context.ethereumProvider.getBalance(ethereum, sourceAccount)

        let feeInfo
        if (logs.length === 0 || !agentAlreadyExists) {
            const [estimatedGas, feeData] = await Promise.all([
                this.context.ethereumProvider.estimateGas(ethereum, tx),
                this.context.ethereumProvider.getFeeData(ethereum),
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

    async tx(
        creation: AgentCreation<EContractTransaction>,
    ): Promise<AgentCreation<EContractTransaction>> {
        await this.validateTx(creation)
        return creation
    }
}

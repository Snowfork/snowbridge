import { AgentCreationInterface, AgentCreation, ValidatedCreateAgent } from "./agentInterface"
import type { Context } from "../../index"
import { ValidationKind } from "../../toPolkadotSnowbridgeV2"
import { ValidationLog, ValidationReason } from "../../toPolkadot_v2"
import { AssetRegistry, EthereumProviderTypes } from "@snowbridge/base-types"
import { ensureValidationSuccess } from "../../utils"
import { hexToU8a, isHex, u8aToHex } from "@polkadot/util"
import { decodeAddress } from "@polkadot/util-crypto"

export class CreateAgent<T extends EthereumProviderTypes>
    implements AgentCreationInterface<T["ContractTransaction"]>
{
    constructor(
        readonly context: Context<T>,
        private readonly registry: AssetRegistry,
    ) {}

    async agentIdForAccount(parachainId: number, account: string): Promise<string> {
        let decoded: Uint8Array
        if (isHex(account)) {
            if (account.length !== 42 && account.length !== 66) {
                throw new Error(
                    `Unsupported account hex length ${account.length}. Expected 20-byte or 32-byte hex.`,
                )
            }
            decoded = hexToU8a(account)
        } else {
            decoded = decodeAddress(account)
        }

        let sourceAccountLocation
        if (decoded.length === 32) {
            sourceAccountLocation = {
                accountId32: {
                    id: u8aToHex(decoded),
                },
            }
        } else if (decoded.length === 20) {
            sourceAccountLocation = {
                accountKey20: {
                    key: u8aToHex(decoded),
                },
            }
        } else {
            throw new Error(
                `Unsupported account length ${decoded.length}. Expected 20-byte or 32-byte account.`,
            )
        }

        const bridgeHub = await this.context.bridgeHub()
        const versionedLocation = bridgeHub.registry.createType("XcmVersionedLocation", {
            v5: {
                parents: 1,
                interior: {
                    x2: [{ parachain: parachainId }, sourceAccountLocation],
                },
            },
        })

        return (await bridgeHub.call.controlV2Api.agentId(versionedLocation)).toHex()
    }

    async tx(
        sourceAccount: string,
        agentId: string,
    ): Promise<AgentCreation<T["ContractTransaction"]>> {
        const tx = await this.context.ethereumProvider.gatewayV2CreateAgent(
            this.context.ethereum(),
            this.context.environment.gatewayContract,
            agentId,
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

    async validate(
        creation: AgentCreation<T["ContractTransaction"]>,
    ): Promise<ValidatedCreateAgent<T["ContractTransaction"]>> {
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
            ...creation,
        }
    }

    async build(
        sourceAccount: string,
        agentId: string,
    ): Promise<ValidatedCreateAgent<T["ContractTransaction"]>> {
        const creation = await this.tx(sourceAccount, agentId)
        return ensureValidationSuccess(await this.validate(creation))
    }
}

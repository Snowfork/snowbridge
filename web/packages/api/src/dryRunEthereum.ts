import { Context } from "./index"
import {
    Transfer,
    ValidationLog,
    ValidationKind,
    ValidationReason,
} from "./types/toEthereum"
import { EthereumProviderTypes, V2CommandStruct } from "@snowbridge/base-types"
import { sourceAgentId } from "./toEthereumSnowbridgeV2"

const V2_DISPATCH_OVERHEAD_GAS = 24_000n
const DRY_RUN_GAS_BUFFER = 100_000_000n
const DEFAULT_FORK_RPC_CONNECT_TIMEOUT_MS = 15_000
const DEFAULT_FORK_RPC_SEND_TIMEOUT_MS = 30_000

export function dryRunCommandGasBudgets(transfer: Transfer): bigint[] {
    if (transfer.input.contractCall) {
        return [10_000_000n, 10_000_000n]
    }
    return [10_000_000n]
}

const L1_ADAPTOR_DEPOSIT_CALL_INVOKED_TOPIC0 =
    "0x14bfd4fd7e654256d3222db5d1ec5e59cd23dd5df10bd8faccc1cabe984b3508"
const L1_ADAPTOR_DEPOSIT_CALL_FAILED_TOPIC0 =
    "0x759aee2ba41080c1e3a57140ba7b446c1347cff289214a2fd1c81554ddc17380"

type EthereumDryRunTx = {
    from?: string
    to?: string
    data?: string
    value?: bigint | string | number
}

type ForkedRpcProvider = {
    send(method: string, params: unknown[]): Promise<unknown>
    waitForTransaction(txHash: string): Promise<{
        status?: number | bigint | string | null
        logs: unknown[]
    } | null>
}

function forkRpcTimeoutMs(envKey: string, defaultMs: number): number {
    const raw = process.env[envKey]
    if (!raw) {
        return defaultMs
    }
    const parsed = Number(raw)
    if (!Number.isFinite(parsed) || parsed <= 0) {
        return defaultMs
    }
    return Math.floor(parsed)
}

async function withTimeout<T>(
    promise: Promise<T>,
    timeoutMs: number,
    operationName: string,
): Promise<T> {
    let timeoutHandle: ReturnType<typeof setTimeout> | undefined
    const timeoutPromise = new Promise<never>((_, reject) => {
        timeoutHandle = setTimeout(() => {
            reject(new Error(`${operationName} timed out after ${timeoutMs}ms`))
        }, timeoutMs)
    })

    try {
        return await Promise.race([promise, timeoutPromise])
    } finally {
        if (timeoutHandle) {
            clearTimeout(timeoutHandle)
        }
    }
}

async function tryImpersonateForkedSigner(
    forkedProvider: ForkedRpcProvider,
    from?: string,
): Promise<boolean> {
    if (!from) {
        return false
    }

    // Ensure the impersonated account has balance to pass intrinsic checks.
    try {
        await forkedProvider.send("anvil_setBalance", [from, "0x56BC75E2D63100000"])
    } catch (e) {
        console.warn("Failed to set balance for impersonated account:", e)
    }

    // Sync the forked node's time with the real clock + 1 hour to pass deadline checks in bridge calls.
    try {
        const nowPlusHour = Math.floor(Date.now() / 1000) + 3600
        await forkedProvider.send("evm_setNextBlockTimestamp", [nowPlusHour])
        await forkedProvider.send("evm_mine", [])
    } catch (e) {
        console.warn("Failed to set forked chain timestamp:", e)
    }

    // Raise block gas limit on the fork: the dry-run dispatch tx asks for ~120M
    // gas to cover both commands plus a generous safety buffer, which exceeds the
    // ~30-60M cap inherited from mainnet. Hex-encoded uint256, value = 480M.
    try {
        await forkedProvider.send("anvil_setBlockGasLimit", ["0x1c9c3800"])
    } catch (e) {
        console.warn("Failed to raise forked chain block gas limit:", e)
    }

    try {
        await forkedProvider.send("anvil_impersonateAccount", [from])
    } catch (e) {
        console.warn("Failed to impersonate account on Anvil:", e)
        return false
    }

    return true
}

export async function buildEthereumDryRunCall<T extends EthereumProviderTypes>(
    context: Context<T>,
    parachainId: number,
    sourceAccountHex: string,
    transfer: Transfer,
): Promise<T["ContractTransaction"]> {
    let commands: V2CommandStruct[] = []
    const agentID = await sourceAgentId(context, parachainId, sourceAccountHex)
    if (transfer.computed.sourceAssetMetadata.foreignId) {
        // PNA
        const mintForeignParams = context.ethereumProvider.encodeAbiParameters(
            ["bytes32", "address", "uint128"],
            [
                transfer.computed.sourceAssetMetadata.foreignId,
                transfer.input.beneficiaryAccount,
                transfer.input.amount,
            ],
        )
        const mintCommand: V2CommandStruct = {
            kind: 4,
            gas: 10_000_000n,
            payload: mintForeignParams,
        }
        commands.push(mintCommand)
    } else {
        // ENA
        const unlockNativeParams = context.ethereumProvider.encodeAbiParameters(
            ["address", "address", "uint128"],
            [transfer.input.tokenAddress, transfer.input.beneficiaryAccount, transfer.input.amount],
        )
        const unlockCommand: V2CommandStruct = {
            kind: 2,
            gas: 10_000_000n,
            payload: unlockNativeParams,
        }
        commands.push(unlockCommand)
    }

    if (transfer.input.contractCall) {
        // Match `abi.encode(CallContractParams)` from Solidity: because
        // CallContractParams contains a dynamic field (`bytes data`), the
        // struct itself is a dynamic tuple, so its ABI encoding is prefixed
        // with a 32-byte offset (= 0x20). Encoding the three fields as a
        // flat tuple omits that leading word and the deployed handler then
        // reads `target` as the bytes offset, fails the uint64 bound check,
        // and reverts.
        const contractCallParams = context.ethereumProvider.encodeAbiParameters(
            ["(address,bytes,uint256)"],
            [
                [
                    transfer.input.contractCall.target,
                    transfer.input.contractCall.calldata,
                    transfer.input.contractCall.value,
                ],
            ],
        )
        const contractCallCommand: V2CommandStruct = {
            kind: 5,
            gas: 10_000_000n,
            payload: contractCallParams,
        }
        commands.push(contractCallCommand)
    }

    const nonce = BigInt(Math.floor(Math.random() * 1000000) + 1000000)
    console.log(`Dry-run: Using nonce ${nonce} and agentID ${agentID}`)
    const ethereumTx = (await context
        .gatewayV2()
        .getFunction("v2_dispatch")
        .populateTransaction(commands, agentID, nonce, {
            from: context.environment.gatewayContract,
        })) as T["ContractTransaction"]

    return ethereumTx
}

function computeDryRunDispatchGasLimit(commandGasBudgets: bigint[]): bigint {
    const requiredGas = commandGasBudgets.reduce((acc, commandGas) => {
        return acc + commandGas + V2_DISPATCH_OVERHEAD_GAS
    }, 0n)
    // Account for the 63/64 forwarding rule used in Gateway.v2_dispatch gas checks.
    const minGas = (requiredGas * 64n + 62n) / 63n
    return minGas + DRY_RUN_GAS_BUFFER
}

function parseL1AdaptorDryRunEvent(
    log: any,
    l1AdapterAddress?: string,
): {
    name: "DepositCallInvoked" | "DepositCallFailed"
    topic?: string
    depositId?: bigint
} | null {
    if (!l1AdapterAddress) {
        return null
    }
    const address = String(log?.address || "").toLowerCase()
    if (!address || address !== l1AdapterAddress.toLowerCase()) {
        return null
    }

    const topics = Array.isArray(log?.topics) ? log.topics.map((t: any) => String(t)) : []
    const topic0 = (topics[0] || "").toLowerCase()
    const data = String(log?.data || "0x")

    if (topic0 === L1_ADAPTOR_DEPOSIT_CALL_INVOKED_TOPIC0 && data.length >= 130) {
        const topic = "0x" + data.slice(2, 66)
        const depositIdHex = data.slice(66, 130)
        return {
            name: "DepositCallInvoked",
            topic,
            depositId: BigInt("0x" + depositIdHex),
        }
    }

    if (topic0 === L1_ADAPTOR_DEPOSIT_CALL_FAILED_TOPIC0 && data.length >= 66) {
        const topic = "0x" + data.slice(2, 66)
        return {
            name: "DepositCallFailed",
            topic,
        }
    }

    return null
}

export async function runEthereumDryRun<T extends EthereumProviderTypes>(
    context: Context<T>,
    sourceParaId: number,
    sourceAccountHex: string,
    transfer: Transfer,
    logs: ValidationLog[],
): Promise<{ ethereumDryRunError?: string }> {
    let ethereumDryRunError: string | undefined

    try {
        const ethereumTx = await buildEthereumDryRunCall(
            context,
            sourceParaId,
            sourceAccountHex,
            transfer,
        )
        const txGasLimit = computeDryRunDispatchGasLimit(dryRunCommandGasBudgets(transfer))
        try {
            const forkedProvider = context.forkedProvider() as unknown as ForkedRpcProvider
            const connectTimeoutMs = forkRpcTimeoutMs(
                "FORKED_PROVIDER_CONNECT_TIMEOUT_MS",
                DEFAULT_FORK_RPC_CONNECT_TIMEOUT_MS,
            )
            const sendTimeoutMs = forkRpcTimeoutMs(
                "FORKED_PROVIDER_SEND_TIMEOUT_MS",
                DEFAULT_FORK_RPC_SEND_TIMEOUT_MS,
            )

            await withTimeout(
                forkedProvider.send("eth_blockNumber", []),
                connectTimeoutMs,
                "Forked RPC eth_blockNumber",
            )
            const txRequest = {
                from: (ethereumTx as EthereumDryRunTx).from,
                to: (ethereumTx as EthereumDryRunTx).to,
                data: (ethereumTx as EthereumDryRunTx).data,
                value: (ethereumTx as EthereumDryRunTx).value ?? "0x0",
                gas: "0x" + txGasLimit.toString(16),
            }

            let txHash: string
            try {
                await tryImpersonateForkedSigner(forkedProvider, txRequest.from)
                await context.ethereumProvider.estimateGas(forkedProvider, ethereumTx)
                txHash = String(
                    await withTimeout(
                        forkedProvider.send("eth_sendTransaction", [txRequest]),
                        sendTimeoutMs,
                        "Forked RPC eth_sendTransaction",
                    ),
                )
            } catch (sendError) {
                const sendErrorMessage = String((sendError as Error).message || sendError)
                const shouldImpersonate =
                    sendErrorMessage.includes("No Signer available") ||
                    sendErrorMessage.includes("unknown account")
                const impersonated = shouldImpersonate
                    ? await tryImpersonateForkedSigner(forkedProvider, txRequest.from)
                    : false
                if (!impersonated) {
                    throw sendError
                }
                await context.ethereumProvider.estimateGas(forkedProvider, ethereumTx)
                txHash = String(
                    await withTimeout(
                        forkedProvider.send("eth_sendTransaction", [txRequest]),
                        sendTimeoutMs,
                        "Forked RPC eth_sendTransaction",
                    ),
                )
            }

            console.log("Tx hash:", txHash)

            const receipt = await withTimeout(
                forkedProvider.waitForTransaction(txHash),
                sendTimeoutMs,
                `Forked RPC waitForTransaction(${txHash})`,
            )
            if (!receipt) {
                ethereumDryRunError =
                    "Dry run transaction simulation on forked Ethereum did not return a receipt (node may be unavailable/out of sync)."
                logs.push({
                    kind: ValidationKind.Warning,
                    reason: ValidationReason.DryRunFailed,
                    message: ethereumDryRunError,
                })
            } else {
                const receiptStatus = receipt.status
                const isRevertedReceipt =
                    receiptStatus === 0 ||
                    receiptStatus === 0n ||
                    receiptStatus === "0x0" ||
                    receiptStatus === "0"
                if (isRevertedReceipt) {
                    ethereumDryRunError =
                        "Dry run transaction simulation reverted on Ethereum (receipt status 0)."
                    logs.push({
                        kind: ValidationKind.Error,
                        reason: ValidationReason.DryRunFailed,
                        message: ethereumDryRunError,
                    })
                }

                console.log("Logs:", receipt.logs)
                const parsedLogs = receipt.logs
                    .map((log: any) => {
                        try {
                            return context.gatewayV2().interface.parseLog(log)
                        } catch (e) {
                            return null
                        }
                    })
                    .filter((log: any) => log !== null)
                const errorLogs = parsedLogs.filter((log: any) => log.name === "CommandFailed")
                console.log("CommandFailed logs:", errorLogs)
                if (errorLogs.length > 0) {
                    const failedIndex = errorLogs[0]?.args?.index
                    ethereumDryRunError =
                        "Dry run v2_dispatch simulation reported CommandFailed at index: " +
                        String(failedIndex)
                    logs.push({
                        kind: ValidationKind.Error,
                        reason: ValidationReason.DryRunFailed,
                        message: ethereumDryRunError,
                    })
                }

                const l1AdapterAddress = context.environment.l2Bridge?.l1AdapterAddress
                const adaptorEvents = receipt.logs
                    .map((log: any) => parseL1AdaptorDryRunEvent(log, l1AdapterAddress))
                    .filter(
                        (
                            event,
                        ): event is {
                            name: "DepositCallInvoked" | "DepositCallFailed"
                            topic?: string
                            depositId?: bigint
                        } => event !== null,
                    )
                const depositCallFailed = adaptorEvents.filter(
                    (event) => event.name === "DepositCallFailed",
                )
                if (depositCallFailed.length > 0) {
                    const topic = depositCallFailed[0]?.topic
                    ethereumDryRunError =
                        "Dry run failed on Ethereum: L1 adaptor emitted DepositCallFailed" +
                        (topic ? ` (topic: ${topic})` : "")
                    logs.push({
                        kind: ValidationKind.Error,
                        reason: ValidationReason.DryRunFailed,
                        message: ethereumDryRunError,
                    })
                }

                const depositCallInvoked = adaptorEvents.find(
                    (event) => event.name === "DepositCallInvoked",
                )
                if (depositCallInvoked) {
                    console.log("L1 adaptor event:", {
                        name: depositCallInvoked.name,
                        topic: depositCallInvoked.topic,
                        depositId: depositCallInvoked.depositId?.toString(),
                    })
                } else if (errorLogs.length > 0 && adaptorEvents.length === 0) {
                    logs.push({
                        kind: ValidationKind.Warning,
                        reason: ValidationReason.DryRunFailed,
                        message:
                            "Synthetic v2_dispatch dry run reported upstream failure before adaptor events; this may diverge from proof-based production execution.",
                    })
                }
            }
        } catch (forkUnavailableError) {
            console.error("Ethereum forked RPC dry-run failed:", forkUnavailableError)
            ethereumDryRunError =
                "Skipping Ethereum dry-run transaction simulation because the forked RPC node is unavailable."
            logs.push({
                kind: ValidationKind.Warning,
                reason: ValidationReason.DryRunFailed,
                message: ethereumDryRunError,
            })
        }
    } catch (e) {
        console.error("Ethereum gas estimation failed:", e)
        ethereumDryRunError = "Could not estimate gas on Ethereum."
        logs.push({
            kind: ValidationKind.Error,
            reason: ValidationReason.FeeEstimationError,
            message: ethereumDryRunError,
        })
    }

    return { ethereumDryRunError }
}

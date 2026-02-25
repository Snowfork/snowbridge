import {
    AbiCoder,
    AbstractProvider,
    Contract,
    ContractTransaction,
    FeeData as EthersFeeData,
    Interface,
    InterfaceAbi,
    JsonRpcProvider,
    parseUnits,
    TransactionReceipt,
    WebSocketProvider,
} from "ethers"
import { isHex, u8aToHex } from "@polkadot/util"
import { decodeAddress } from "@polkadot/util-crypto"
import { IERC20, IERC20_ABI, IGATEWAY_V1_ABI, IGATEWAY_V2_ABI } from "./contracts"
import { MultiAddressStruct } from "./contracts"
import { Context } from "./"

export type GatewayV1OutboundMessageAccepted = {
    channelId: string
    nonce: bigint
    messageId: string
}

export type GatewayV2OutboundMessageAccepted = {
    nonce: bigint
    payload: string
}

export type FeeData = {
    gasPrice: bigint | null
    maxFeePerGas: bigint | null
    maxPriorityFeePerGas: bigint | null
}

export type EncodedMultiAddress = {
    address: MultiAddressStruct
    hexAddress: string
}

export const encodeAssetsArray = (encodedAssets: readonly string[]): string => {
    return AbiCoder.defaultAbiCoder().encode(["bytes[]"], [encodedAssets])
}

export const beneficiaryMultiAddress = (beneficiary: string): EncodedMultiAddress => {
    const abi = AbiCoder.defaultAbiCoder()

    let address: MultiAddressStruct
    let hexAddress: string
    if (isHex(beneficiary)) {
        hexAddress = beneficiary
        if (beneficiary.length === 42) {
            address = {
                kind: 2,
                data: abi.encode(["bytes20"], [hexAddress]),
            }
        } else if (beneficiary.length === 66) {
            address = {
                kind: 1,
                data: abi.encode(["bytes32"], [hexAddress]),
            }
        } else {
            throw new Error("Unknown Beneficiary address format.")
        }
    } else {
        hexAddress = u8aToHex(decodeAddress(beneficiary))
        address = {
            kind: 1,
            data: abi.encode(["bytes32"], [hexAddress]),
        }
    }

    return { address, hexAddress }
}

export interface EthereumProvider<Connection, Contract, Abi, Interface, Transaction> {
    createProvider(url: string): Connection
    destroyProvider(provider: Connection): void
    destroyContract(contract: Contract): Promise<void>
    connectContract<T extends Contract>(
        address: string,
        abi: Abi,
        provider: Connection,
    ): Contract & T
    erc20Balance(
        provider: Connection,
        tokenAddress: string,
        owner: string,
        spender: string,
    ): Promise<{ balance: bigint; gatewayAllowance: bigint }>
    populateTransaction(contract: Contract, method: string, ...args: any[]): Promise<Transaction>
    encodeFunctionData(abi: Abi, method: string, args: readonly unknown[]): string
    decodeFunctionResult<T = unknown>(abi: Abi, method: string, data: string): T
    encodeAssetsArray(encodedAssets: readonly string[]): string
    beneficiaryMultiAddress(beneficiary: string): EncodedMultiAddress
    estimateGas(provider: Connection, tx: Transaction): Promise<bigint>
    getFeeData(provider: Connection): Promise<FeeData>
    parseUnits(value: string, decimals: number): bigint
    isContractAddress(provider: Connection, address: string): Promise<boolean>
    scanGatewayV1OutboundMessageAccepted(
        receipt: TransactionReceipt,
    ): GatewayV1OutboundMessageAccepted | null
    scanGatewayV2OutboundMessageAccepted(
        receipt: TransactionReceipt,
    ): GatewayV2OutboundMessageAccepted | null
}

export class EthersEthereumProvider
    implements
        EthereumProvider<AbstractProvider, Contract, InterfaceAbi, Interface, ContractTransaction>
{
    static gatewayV1Interface = new Interface(IGATEWAY_V1_ABI)
    static gatewayV2Interface = new Interface(IGATEWAY_V2_ABI)

    createProvider(url: string): AbstractProvider {
        if (url.startsWith("http")) {
            return new JsonRpcProvider(url)
        }
        return new WebSocketProvider(url)
    }

    destroyProvider(provider: AbstractProvider): void {
        provider.destroy()
    }

    async destroyContract(contract: Contract): Promise<void> {
        await contract.removeAllListeners()
    }

    connectContract<T extends Contract>(
        address: string,
        abi: InterfaceAbi,
        provider: AbstractProvider,
    ): Contract & T {
        return new Contract(address, abi, provider) as T
    }

    async erc20Balance(
        provider: AbstractProvider,
        tokenAddress: string,
        owner: string,
        spender: string,
    ): Promise<{ balance: bigint; gatewayAllowance: bigint }> {
        const tokenContract = this.connectContract<Contract & IERC20>(
            tokenAddress,
            IERC20_ABI,
            provider,
        )
        const [balance, gatewayAllowance] = await Promise.all([
            tokenContract.balanceOf(owner),
            tokenContract.allowance(owner, spender),
        ])
        return {
            balance,
            gatewayAllowance,
        }
    }

    async populateTransaction(
        contract: Contract,
        method: string,
        ...args: any[]
    ): Promise<ContractTransaction> {
        return await contract.getFunction(method).populateTransaction(...args)
    }

    encodeFunctionData(abi: InterfaceAbi, method: string, args: readonly unknown[]): string {
        return new Interface(abi).encodeFunctionData(method, args)
    }

    decodeFunctionResult<T = unknown>(abi: InterfaceAbi, method: string, data: string): T {
        return new Interface(abi).decodeFunctionResult(method, data) as T
    }

    encodeAssetsArray(encodedAssets: readonly string[]): string {
        return encodeAssetsArray(encodedAssets)
    }

    beneficiaryMultiAddress(beneficiary: string): EncodedMultiAddress {
        return beneficiaryMultiAddress(beneficiary)
    }

    async estimateGas(provider: AbstractProvider, tx: ContractTransaction): Promise<bigint> {
        return await provider.estimateGas(tx)
    }

    async getFeeData(provider: AbstractProvider): Promise<FeeData> {
        const feeData: EthersFeeData = await provider.getFeeData()
        return {
            gasPrice: feeData.gasPrice ?? null,
            maxFeePerGas: feeData.maxFeePerGas ?? null,
            maxPriorityFeePerGas: feeData.maxPriorityFeePerGas ?? null,
        }
    }

    parseUnits(value: string, decimals: number): bigint {
        return parseUnits(value, decimals)
    }

    async isContractAddress(provider: AbstractProvider, address: string): Promise<boolean> {
        if (!/^0x[a-fA-F0-9]{40}$/.test(address)) {
            return false
        }
        try {
            const code = await provider.getCode(address)
            return code !== "0x"
        } catch {
            return false
        }
    }

    scanGatewayV1OutboundMessageAccepted(
        receipt: TransactionReceipt,
    ): GatewayV1OutboundMessageAccepted | null {
        for (const log of receipt.logs) {
            try {
                const event = EthersEthereumProvider.gatewayV1Interface.parseLog({
                    topics: [...log.topics],
                    data: log.data,
                })
                if (event && event.name === "OutboundMessageAccepted") {
                    return {
                        channelId: String(event.args[0]),
                        nonce: BigInt(event.args[1]),
                        messageId: String(event.args[2]),
                    }
                }
            } catch {
                // Ignore logs that don't match the gateway ABI.
            }
        }
        return null
    }

    scanGatewayV2OutboundMessageAccepted(
        receipt: TransactionReceipt,
    ): GatewayV2OutboundMessageAccepted | null {
        for (const log of receipt.logs) {
            try {
                const event = EthersEthereumProvider.gatewayV2Interface.parseLog({
                    topics: [...log.topics],
                    data: log.data,
                })
                if (event && event.name === "OutboundMessageAccepted") {
                    return {
                        nonce: BigInt(event.args[0]),
                        payload: event.args[1],
                    }
                }
            } catch {
                // Ignore logs that don't match the gateway ABI.
            }
        }
        return null
    }
}

export type EthersContext = Context<
    AbstractProvider,
    Contract,
    InterfaceAbi,
    Interface,
    ContractTransaction
>

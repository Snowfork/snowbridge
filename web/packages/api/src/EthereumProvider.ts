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
import {
    IERC20,
    IERC20_ABI,
    IGATEWAY_V1_ABI,
    IGATEWAY_V2_ABI,
    SNOWBRIDGE_L2_ADAPTOR_ABI,
} from "./contracts"
import { MultiAddressStruct } from "./contracts"
import { Context } from "./"

export type GatewayV1OutboundMessageAccepted = {
    channelId: string
    nonce: bigint
    messageId: string

    blockNumber: number
    blockHash: string
    txHash: string
    txIndex: number
}

export type GatewayV2OutboundMessageAccepted = {
    nonce: bigint
    payload: string

    blockNumber: number
    blockHash: string
    txHash: string
    txIndex: number
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

export const encodeNativeAsset = (tokenAddress: string, amount: bigint): string => {
    return AbiCoder.defaultAbiCoder().encode(
        ["uint8", "address", "uint128"],
        [0, tokenAddress, amount],
    )
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

const PALLET_XCM_PRECOMPILE_ABI: InterfaceAbi = [
    {
        inputs: [
            {
                components: [
                    { internalType: "uint8", name: "parents", type: "uint8" },
                    { internalType: "bytes[]", name: "interior", type: "bytes[]" },
                ],
                internalType: "struct XCM.Location",
                name: "dest",
                type: "tuple",
            },
            {
                components: [
                    { internalType: "address", name: "asset", type: "address" },
                    { internalType: "uint256", name: "amount", type: "uint256" },
                ],
                internalType: "struct XCM.AssetAddressInfo[]",
                name: "assets",
                type: "tuple[]",
            },
            {
                internalType: "enum XCM.TransferType",
                name: "assetsTransferType",
                type: "uint8",
            },
            { internalType: "uint8", name: "remoteFeesIdIndex", type: "uint8" },
            {
                internalType: "enum XCM.TransferType",
                name: "feesTransferType",
                type: "uint8",
            },
            { internalType: "bytes", name: "customXcmOnDest", type: "bytes" },
        ],
        name: "transferAssetsUsingTypeAndThenAddress",
        outputs: [],
        stateMutability: "nonpayable",
        type: "function",
    },
]

export interface EthereumProvider<
    Connection,
    Contract,
    Abi,
    Interface,
    TransactionReceipt,
    ContractTransaction,
> {
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
    populateTransaction(
        contract: Contract,
        method: string,
        ...args: any[]
    ): Promise<ContractTransaction>
    encodeFunctionData(abi: Abi, method: string, args: readonly unknown[]): string
    decodeFunctionResult<T = unknown>(abi: Abi, method: string, data: string): T
    encodeNativeAsset(tokenAddress: string, amount: bigint): string
    encodeAssetsArray(encodedAssets: readonly string[]): string
    beneficiaryMultiAddress(beneficiary: string): EncodedMultiAddress
    estimateGas(provider: Connection, tx: ContractTransaction): Promise<bigint>
    gatewayV1SendToken(
        provider: Connection,
        gatewayAddress: string,
        sourceAccount: string,
        tokenAddress: string,
        destinationParaId: number,
        beneficiary: MultiAddressStruct,
        totalFeeDot: bigint,
        amount: bigint,
        value: bigint,
    ): Promise<ContractTransaction>
    gatewayV2RegisterToken(
        provider: Connection,
        gatewayAddress: string,
        sourceAccount: string,
        tokenAddress: string,
        network: number,
        assetHubExecutionFeeEther: bigint,
        relayerFee: bigint,
        totalValue: bigint,
    ): Promise<ContractTransaction>
    gatewayV2CreateAgent(
        provider: Connection,
        gatewayAddress: string,
        sourceAccount: string,
        agentId: string,
    ): Promise<ContractTransaction>
    gatewayV2SendMessage(
        provider: Connection,
        gatewayAddress: string,
        sourceAccount: string,
        xcm: Uint8Array,
        assets: any[],
        claimer: Uint8Array,
        assetHubExecutionFeeEther: bigint,
        relayerFee: bigint,
        value: bigint,
    ): Promise<ContractTransaction>
    l2AdapterSendEtherAndCall(
        provider: Connection,
        adapterAddress: string,
        sourceAccount: string,
        depositParams: any,
        sendParams: any,
        refundAddress: string,
        topic: string,
        value?: bigint,
    ): Promise<ContractTransaction>
    l2AdapterSendTokenAndCall(
        provider: Connection,
        adapterAddress: string,
        sourceAccount: string,
        depositParams: any,
        swapParams: any,
        sendParams: any,
        refundAddress: string,
        topic: string,
    ): Promise<ContractTransaction>
    evmParachainTransferAssetsUsingTypeAndThenAddress(
        provider: Connection,
        precompileAddress: string,
        sourceAccount: string,
        destination: [number, string[]],
        assets: [string, bigint][],
        assetsTransferType: number,
        remoteFeesIdIndex: number,
        feesTransferType: number,
        customXcmHex: string,
    ): Promise<ContractTransaction>
    getBalance(provider: Connection, address: string): Promise<bigint>
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
        EthereumProvider<
            AbstractProvider,
            Contract,
            InterfaceAbi,
            Interface,
            TransactionReceipt,
            ContractTransaction
        >
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

    async gatewayV1SendToken(
        provider: AbstractProvider,
        gatewayAddress: string,
        sourceAccount: string,
        tokenAddress: string,
        destinationParaId: number,
        beneficiary: MultiAddressStruct,
        totalFeeDot: bigint,
        amount: bigint,
        value: bigint,
    ): Promise<ContractTransaction> {
        const gateway = this.connectContract(gatewayAddress, IGATEWAY_V1_ABI, provider)
        return await this.populateTransaction(
            gateway,
            "sendToken",
            tokenAddress,
            destinationParaId,
            beneficiary,
            totalFeeDot,
            amount,
            { value, from: sourceAccount },
        )
    }

    async gatewayV2RegisterToken(
        provider: AbstractProvider,
        gatewayAddress: string,
        sourceAccount: string,
        tokenAddress: string,
        network: number,
        assetHubExecutionFeeEther: bigint,
        relayerFee: bigint,
        totalValue: bigint,
    ): Promise<ContractTransaction> {
        const gateway = this.connectContract(gatewayAddress, IGATEWAY_V2_ABI, provider)
        return await this.populateTransaction(
            gateway,
            "v2_registerToken",
            tokenAddress,
            network,
            assetHubExecutionFeeEther,
            relayerFee,
            { value: totalValue, from: sourceAccount },
        )
    }

    async gatewayV2CreateAgent(
        provider: AbstractProvider,
        gatewayAddress: string,
        sourceAccount: string,
        agentId: string,
    ): Promise<ContractTransaction> {
        const gateway = this.connectContract(gatewayAddress, IGATEWAY_V2_ABI, provider)
        return await this.populateTransaction(gateway, "v2_createAgent", agentId, {
            from: sourceAccount,
        })
    }

    async gatewayV2SendMessage(
        provider: AbstractProvider,
        gatewayAddress: string,
        sourceAccount: string,
        xcm: Uint8Array,
        assets: any[],
        claimer: Uint8Array,
        assetHubExecutionFeeEther: bigint,
        relayerFee: bigint,
        value: bigint,
    ): Promise<ContractTransaction> {
        const gateway = this.connectContract(gatewayAddress, IGATEWAY_V2_ABI, provider)
        return await this.populateTransaction(
            gateway,
            "v2_sendMessage",
            xcm,
            assets,
            claimer,
            assetHubExecutionFeeEther,
            relayerFee,
            { value, from: sourceAccount },
        )
    }

    async l2AdapterSendEtherAndCall(
        provider: AbstractProvider,
        adapterAddress: string,
        sourceAccount: string,
        depositParams: any,
        sendParams: any,
        refundAddress: string,
        topic: string,
        value?: bigint,
    ): Promise<ContractTransaction> {
        const adapter = this.connectContract(adapterAddress, SNOWBRIDGE_L2_ADAPTOR_ABI, provider)
        const txOptions = value === undefined ? { from: sourceAccount } : { from: sourceAccount, value }
        return await this.populateTransaction(
            adapter,
            "sendEtherAndCall",
            depositParams,
            sendParams,
            refundAddress,
            topic,
            txOptions,
        )
    }

    async l2AdapterSendTokenAndCall(
        provider: AbstractProvider,
        adapterAddress: string,
        sourceAccount: string,
        depositParams: any,
        swapParams: any,
        sendParams: any,
        refundAddress: string,
        topic: string,
    ): Promise<ContractTransaction> {
        const adapter = this.connectContract(adapterAddress, SNOWBRIDGE_L2_ADAPTOR_ABI, provider)
        return await this.populateTransaction(
            adapter,
            "sendTokenAndCall",
            depositParams,
            swapParams,
            sendParams,
            refundAddress,
            topic,
            { from: sourceAccount },
        )
    }

    async evmParachainTransferAssetsUsingTypeAndThenAddress(
        provider: AbstractProvider,
        precompileAddress: string,
        sourceAccount: string,
        destination: [number, string[]],
        assets: [string, bigint][],
        assetsTransferType: number,
        remoteFeesIdIndex: number,
        feesTransferType: number,
        customXcmHex: string,
    ): Promise<ContractTransaction> {
        const precompile = this.connectContract(precompileAddress, PALLET_XCM_PRECOMPILE_ABI, provider)
        const tx = await this.populateTransaction(
            precompile,
            "transferAssetsUsingTypeAndThenAddress((uint8,bytes[]),(address,uint256)[],uint8,uint8,uint8,bytes)",
            destination,
            assets,
            assetsTransferType,
            remoteFeesIdIndex,
            feesTransferType,
            customXcmHex,
        )
        tx.from = sourceAccount
        return tx
    }

    encodeFunctionData(abi: InterfaceAbi, method: string, args: readonly unknown[]): string {
        return new Interface(abi).encodeFunctionData(method, args)
    }

    decodeFunctionResult<T = unknown>(abi: InterfaceAbi, method: string, data: string): T {
        return new Interface(abi).decodeFunctionResult(method, data) as T
    }

    encodeNativeAsset(tokenAddress: string, amount: bigint): string {
        return encodeNativeAsset(tokenAddress, amount)
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

    async getBalance(provider: AbstractProvider, address: string): Promise<bigint> {
        return await provider.getBalance(address)
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

                        blockHash: receipt.blockHash,
                        blockNumber: receipt.blockNumber,
                        txHash: receipt.hash,
                        txIndex: receipt.index
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
                        blockHash: receipt.blockHash,
                        blockNumber: receipt.blockNumber,
                        txHash: receipt.hash,
                        txIndex: receipt.index
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
    TransactionReceipt,
    ContractTransaction
>

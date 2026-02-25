import {
    AbstractProvider,
    Contract,
    ContractTransaction,
    Interface,
    InterfaceAbi,
    JsonRpcProvider,
    WebSocketProvider,
} from "ethers"
import { IERC20, IERC20_ABI } from "./contracts"
import { Context } from "./"

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
    populateTransaction(
        contract: Contract,
        method: string,
        ...args: any[]
    ): Promise<Transaction>
}

export class EthersEthereumProvider
    implements
        EthereumProvider<AbstractProvider, Contract, InterfaceAbi, Interface, ContractTransaction>
{
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
}

export type EthersContext = Context<
    AbstractProvider,
    Contract,
    InterfaceAbi,
    Interface,
    ContractTransaction
>

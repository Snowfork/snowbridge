import {
    AbstractProvider,
    Contract,
    Interface,
    InterfaceAbi,
    JsonRpcProvider,
    WebSocketProvider,
} from "ethers"
import {
    BeefyClient,
    BEEFY_CLIENT_ABI,
    IGatewayV1,
    IGATEWAY_V1_ABI,
    IGatewayV2,
    IGATEWAY_V2_ABI,
    ISwapLegacyRouter,
    SWAP_LEGACY_ROUTER_ABI,
    ISwapQuoter,
    SWAP_QUOTER_ABI,
    ISwapRouter,
    SWAP_ROUTER_ABI,
    SnowbridgeL1Adaptor,
    SNOWBRIDGE_L1_ADAPTOR_ABI,
    SnowbridgeL2Adaptor,
    SNOWBRIDGE_L2_ADAPTOR_ABI,
    IERC20,
    IERC20_ABI,
} from "./contracts"

export interface EthereumProvider<Connection, Contract, Abi, Interface> {
    createProvider(url: string): Connection
    destroyProvider(provider: Connection): void
    destroyContract(contract: Contract): Promise<void>
    createInterface(abi: Abi): Interface
    connectContract<T extends Contract>(address: string, abi: Abi, provider: Connection): T
    connectGatewayV1(address: string, provider: Connection): Contract & IGatewayV1
    connectGatewayV2(address: string, provider: Connection): Contract & IGatewayV2
    connectBeefyClient(address: string, provider: Connection): Contract & BeefyClient
    connectL1Adapter(address: string, provider: Connection): Contract & SnowbridgeL1Adaptor
    connectL1SwapQuoter(address: string, provider: Connection): Contract & ISwapQuoter
    connectL1SwapRouter(address: string, provider: Connection): Contract & ISwapRouter
    connectL1LegacySwapRouter(address: string, provider: Connection): Contract & ISwapLegacyRouter
    connectL2Adapter(address: string, provider: Connection): Contract & SnowbridgeL2Adaptor
    erc20Balance(
        provider: Connection,
        tokenAddress: string,
        owner: string,
        spender: string,
    ): Promise<{ balance: bigint; gatewayAllowance: bigint }>
}

export class EthersEthereumProvider
    implements EthereumProvider<AbstractProvider, Contract, InterfaceAbi, Interface>
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

    createInterface(abi: unknown): Interface {
        return new Interface(abi as any)
    }

    connectContract<T extends Contract>(
        address: string,
        abi: InterfaceAbi,
        provider: AbstractProvider,
    ): T {
        return new Contract(address, abi, provider) as T
    }

    connectGatewayV1(address: string, provider: AbstractProvider): Contract & IGatewayV1 {
        return this.connectContract<Contract & IGatewayV1>(address, IGATEWAY_V1_ABI, provider)
    }

    connectGatewayV2(address: string, provider: AbstractProvider): Contract & IGatewayV2 {
        return this.connectContract<Contract & IGatewayV2>(address, IGATEWAY_V2_ABI, provider)
    }

    connectBeefyClient(address: string, provider: AbstractProvider): Contract & BeefyClient {
        return this.connectContract<Contract & BeefyClient>(address, BEEFY_CLIENT_ABI, provider)
    }

    connectL1Adapter(address: string, provider: AbstractProvider): Contract & SnowbridgeL1Adaptor {
        return this.connectContract<Contract & SnowbridgeL1Adaptor>(
            address,
            SNOWBRIDGE_L1_ADAPTOR_ABI,
            provider,
        )
    }

    connectL1SwapQuoter(address: string, provider: AbstractProvider): Contract & ISwapQuoter {
        return this.connectContract<Contract & ISwapQuoter>(address, SWAP_QUOTER_ABI, provider)
    }

    connectL1SwapRouter(address: string, provider: AbstractProvider): Contract & ISwapRouter {
        return this.connectContract<Contract & ISwapRouter>(address, SWAP_ROUTER_ABI, provider)
    }

    connectL1LegacySwapRouter(
        address: string,
        provider: AbstractProvider,
    ): Contract & ISwapLegacyRouter {
        return this.connectContract<Contract & ISwapLegacyRouter>(
            address,
            SWAP_LEGACY_ROUTER_ABI,
            provider,
        )
    }

    connectL2Adapter(address: string, provider: AbstractProvider): Contract & SnowbridgeL2Adaptor {
        return this.connectContract<Contract & SnowbridgeL2Adaptor>(
            address,
            SNOWBRIDGE_L2_ADAPTOR_ABI,
            provider,
        )
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
}

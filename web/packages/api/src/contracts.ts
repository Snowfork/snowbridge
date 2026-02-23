import { Contract, ContractRunner, Interface } from "ethers"

export type MultiAddressStruct = {
    kind: number
    data: string
}

export type QuoteExactOutputSingleParamsStruct = {
    tokenIn: string
    tokenOut: string
    amount: bigint
    fee: number
    sqrtPriceLimitX96: bigint | number
}

export type DepositParamsStruct = {
    inputToken: string
    outputToken: string
    inputAmount: bigint
    outputAmount: bigint
    destinationChainId: bigint
    fillDeadlineBuffer: bigint
}

export type SendParamsStruct = {
    xcm: Uint8Array
    assets: any[]
    claimer: Uint8Array
    executionFee: bigint
    relayerFee: bigint
}

export type SwapParamsStruct = {
    inputAmount: bigint
    router: string
    callData: string
}

export type IGatewayV1 = Contract & {
    quoteSendTokenFee(
        tokenAddress: string,
        destinationParaId: number,
        totalFeeInDot: bigint,
    ): Promise<bigint>
    operatingMode(): Promise<bigint>
    channelNoncesOf(channelId: string): Promise<[bigint, bigint]>
    channelOperatingModeOf(channelId: string): Promise<bigint>
    agentOf(agentId: string): Promise<string>
}

export type IGatewayV2 = Contract & {
    operatingMode(): Promise<bigint>
    v2_outboundNonce(): Promise<bigint>
    isTokenRegistered(tokenAddress: string): Promise<boolean>
    agentOf(agentId: string): Promise<string>
}

export type BeefyClient = Contract & {
    latestBeefyBlock(): Promise<bigint>
}

export type SnowbridgeL1Adaptor = Contract
export type ISwapRouter = Contract
export type ISwapLegacyRouter = Contract

export type ISwapQuoter = Contract & {
    quoteExactOutputSingle: {
        staticCall(params: QuoteExactOutputSingleParamsStruct): Promise<readonly [bigint, ...any[]]>
    }
}

export type SnowbridgeL2Adaptor = Contract

export type IERC20 = Contract & {
    balanceOf(owner: string): Promise<bigint>
    allowance(owner: string, spender: string): Promise<bigint>
}

const IGATEWAY_V1_ABI = [
    "function quoteSendTokenFee(address tokenAddress, uint32 destinationParaId, uint128 totalFeeInDot) view returns (uint256)",
    "function operatingMode() view returns (uint8)",
    "function channelNoncesOf(bytes32 channelId) view returns (uint64 inbound, uint64 outbound)",
    "function channelOperatingModeOf(bytes32 channelId) view returns (uint8)",
    "function agentOf(bytes32 agentId) view returns (address)",
    "function sendToken(address tokenAddress, uint32 destinationParaId, (uint8 kind, bytes data) beneficiary, uint128 destinationFeeInDOT, uint128 amount) payable",
    "event OutboundMessageAccepted(bytes32 indexed channelId, uint64 nonce, bytes32 messageId)",
]

const IGATEWAY_V2_ABI = [
    "function operatingMode() view returns (uint8)",
    "function v2_outboundNonce() view returns (uint64)",
    "function isTokenRegistered(address tokenAddress) view returns (bool)",
    "function agentOf(bytes32 agentId) view returns (address)",
    "function v2_createAgent(bytes32 agentId) payable",
    "function v2_registerToken(address tokenAddress, uint8 network, uint128 executionFee, uint128 relayerFee) payable",
    "function v2_sendMessage(bytes xcm, bytes[] assets, bytes claimer, uint128 executionFee, uint128 relayerFee) payable",
    "event OutboundMessageAccepted(uint64 nonce, bytes payload)",
]

const IERC20_ABI = [
    "function balanceOf(address owner) view returns (uint256)",
    "function allowance(address owner, address spender) view returns (uint256)",
    "function approve(address spender, uint256 amount) returns (bool)",
]

const BEEFY_CLIENT_ABI = ["function latestBeefyBlock() view returns (uint32)"]

const SWAP_QUOTER_ABI = [
    "function quoteExactOutputSingle((address tokenIn,address tokenOut,uint256 amount,uint24 fee,uint160 sqrtPriceLimitX96) params) view returns (uint256 amountIn, uint160 sqrtPriceX96After, uint32 initializedTicksCrossed, uint256 gasEstimate)",
]

const SWAP_ROUTER_ABI = [
    "function exactOutputSingle((address tokenIn,address tokenOut,uint24 fee,address recipient,uint256 deadline,uint256 amountOut,uint256 amountInMaximum,uint160 sqrtPriceLimitX96) params) payable returns (uint256 amountIn)",
]

const SWAP_LEGACY_ROUTER_ABI = [
    "function exactOutputSingle((address tokenIn,address tokenOut,uint24 fee,address recipient,uint256 amountOut,uint256 amountInMaximum,uint160 sqrtPriceLimitX96) params) payable returns (uint256 amountIn)",
]

const SNOWBRIDGE_L1_ADAPTOR_ABI = [
    "function depositNativeEther((address inputToken,address outputToken,uint256 inputAmount,uint256 outputAmount,uint256 destinationChainId,uint256 fillDeadlineBuffer) params, address recipient, bytes32 topic) payable",
    "function depositToken((address inputToken,address outputToken,uint256 inputAmount,uint256 outputAmount,uint256 destinationChainId,uint256 fillDeadlineBuffer) params, address recipient, bytes32 topic)",
]

const SNOWBRIDGE_L2_ADAPTOR_ABI = [
    "function sendEtherAndCall((address inputToken,address outputToken,uint256 inputAmount,uint256 outputAmount,uint256 destinationChainId,uint256 fillDeadlineBuffer) depositParams, (bytes xcm, bytes[] assets, bytes claimer, uint128 executionFee, uint128 relayerFee) sendParams, address sourceAccount, bytes32 topic) payable",
    "function sendTokenAndCall((address inputToken,address outputToken,uint256 inputAmount,uint256 outputAmount,uint256 destinationChainId,uint256 fillDeadlineBuffer) depositParams, (uint256 inputAmount, address router, bytes callData) swapParams, (bytes xcm, bytes[] assets, bytes claimer, uint128 executionFee, uint128 relayerFee) sendParams, address sourceAccount, bytes32 topic)",
]

const connect = <T extends Contract>(
    abi: readonly string[],
    address: string,
    runner?: ContractRunner,
) => new Contract(address, abi, runner) as unknown as T

export const IGatewayV1__factory = {
    connect(address: string, runner?: ContractRunner): IGatewayV1 {
        return connect<IGatewayV1>(IGATEWAY_V1_ABI, address, runner)
    },
    createInterface(): Interface {
        return new Interface(IGATEWAY_V1_ABI)
    },
}

export const IGatewayV2__factory = {
    connect(address: string, runner?: ContractRunner): IGatewayV2 {
        return connect<IGatewayV2>(IGATEWAY_V2_ABI, address, runner)
    },
    createInterface(): Interface {
        return new Interface(IGATEWAY_V2_ABI)
    },
}

export const IERC20__factory = {
    connect(address: string, runner?: ContractRunner): IERC20 {
        return connect<IERC20>(IERC20_ABI, address, runner)
    },
}

export const BeefyClient__factory = {
    connect(address: string, runner?: ContractRunner): BeefyClient {
        return connect<BeefyClient>(BEEFY_CLIENT_ABI, address, runner)
    },
}

export const ISwapQuoter__factory = {
    connect(address: string, runner?: ContractRunner): ISwapQuoter {
        return connect<ISwapQuoter>(SWAP_QUOTER_ABI, address, runner)
    },
}

export const ISwapRouter__factory = {
    connect(address: string, runner?: ContractRunner): ISwapRouter {
        return connect<ISwapRouter>(SWAP_ROUTER_ABI, address, runner)
    },
}

export const ISwapLegacyRouter__factory = {
    connect(address: string, runner?: ContractRunner): ISwapLegacyRouter {
        return connect<ISwapLegacyRouter>(SWAP_LEGACY_ROUTER_ABI, address, runner)
    },
}

export const SnowbridgeL1Adaptor__factory = {
    connect(address: string, runner?: ContractRunner): SnowbridgeL1Adaptor {
        return connect<SnowbridgeL1Adaptor>(SNOWBRIDGE_L1_ADAPTOR_ABI, address, runner)
    },
}

export const SnowbridgeL2Adaptor__factory = {
    connect(address: string, runner?: ContractRunner): SnowbridgeL2Adaptor {
        return connect<SnowbridgeL2Adaptor>(SNOWBRIDGE_L2_ADAPTOR_ABI, address, runner)
    },
}

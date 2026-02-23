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

export type ContractFunctionLike = {
    populateTransaction(...args: any[]): Promise<any>
}

export type ContractLike = {
    removeAllListeners(): Promise<any>
    getFunction(name: string): ContractFunctionLike
    interface: {
        encodeFunctionData(name: string, values?: ReadonlyArray<any>): string
    }
    target: unknown
    getAddress(): Promise<string>
}

export type IGatewayV1 = ContractLike & {
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

export type IGatewayV2 = ContractLike & {
    operatingMode(): Promise<bigint>
    v2_outboundNonce(): Promise<bigint>
    isTokenRegistered(tokenAddress: string): Promise<boolean>
    agentOf(agentId: string): Promise<string>
}

export type BeefyClient = ContractLike & {
    latestBeefyBlock(): Promise<bigint>
}

export type SnowbridgeL1Adaptor = ContractLike
export type ISwapRouter = ContractLike
export type ISwapLegacyRouter = ContractLike

export type ISwapQuoter = ContractLike & {
    quoteExactOutputSingle: {
        staticCall(params: QuoteExactOutputSingleParamsStruct): Promise<readonly [bigint, ...any[]]>
    }
}

export type SnowbridgeL2Adaptor = ContractLike

export type IERC20 = ContractLike & {
    balanceOf(owner: string): Promise<bigint>
    allowance(owner: string, spender: string): Promise<bigint>
}

export const IGATEWAY_V1_ABI = [
    "function quoteSendTokenFee(address tokenAddress, uint32 destinationParaId, uint128 totalFeeInDot) view returns (uint256)",
    "function operatingMode() view returns (uint8)",
    "function channelNoncesOf(bytes32 channelId) view returns (uint64 inbound, uint64 outbound)",
    "function channelOperatingModeOf(bytes32 channelId) view returns (uint8)",
    "function agentOf(bytes32 agentId) view returns (address)",
    "function sendToken(address tokenAddress, uint32 destinationParaId, (uint8 kind, bytes data) beneficiary, uint128 destinationFeeInDOT, uint128 amount) payable",
    "event OutboundMessageAccepted(bytes32 indexed channelId, uint64 nonce, bytes32 messageId)",
] as const

export const IGATEWAY_V2_ABI = [
    "function operatingMode() view returns (uint8)",
    "function v2_outboundNonce() view returns (uint64)",
    "function isTokenRegistered(address tokenAddress) view returns (bool)",
    "function agentOf(bytes32 agentId) view returns (address)",
    "function v2_createAgent(bytes32 agentId) payable",
    "function v2_registerToken(address tokenAddress, uint8 network, uint128 executionFee, uint128 relayerFee) payable",
    "function v2_sendMessage(bytes xcm, bytes[] assets, bytes claimer, uint128 executionFee, uint128 relayerFee) payable",
    "event OutboundMessageAccepted(uint64 nonce, bytes payload)",
] as const

export const IERC20_ABI = [
    "function balanceOf(address owner) view returns (uint256)",
    "function allowance(address owner, address spender) view returns (uint256)",
    "function approve(address spender, uint256 amount) returns (bool)",
] as const

export const BEEFY_CLIENT_ABI = ["function latestBeefyBlock() view returns (uint32)"] as const

export const SWAP_QUOTER_ABI = [
    "function quoteExactOutputSingle((address tokenIn,address tokenOut,uint256 amount,uint24 fee,uint160 sqrtPriceLimitX96) params) view returns (uint256 amountIn, uint160 sqrtPriceX96After, uint32 initializedTicksCrossed, uint256 gasEstimate)",
] as const

export const SWAP_ROUTER_ABI = [
    "function exactOutputSingle((address tokenIn,address tokenOut,uint24 fee,address recipient,uint256 deadline,uint256 amountOut,uint256 amountInMaximum,uint160 sqrtPriceLimitX96) params) payable returns (uint256 amountIn)",
] as const

export const SWAP_LEGACY_ROUTER_ABI = [
    "function exactOutputSingle((address tokenIn,address tokenOut,uint24 fee,address recipient,uint256 amountOut,uint256 amountInMaximum,uint160 sqrtPriceLimitX96) params) payable returns (uint256 amountIn)",
] as const

export const SNOWBRIDGE_L1_ADAPTOR_ABI = [
    "function depositNativeEther((address inputToken,address outputToken,uint256 inputAmount,uint256 outputAmount,uint256 destinationChainId,uint256 fillDeadlineBuffer) params, address recipient, bytes32 topic) payable",
    "function depositToken((address inputToken,address outputToken,uint256 inputAmount,uint256 outputAmount,uint256 destinationChainId,uint256 fillDeadlineBuffer) params, address recipient, bytes32 topic)",
] as const

export const SNOWBRIDGE_L2_ADAPTOR_ABI = [
    "function sendEtherAndCall((address inputToken,address outputToken,uint256 inputAmount,uint256 outputAmount,uint256 destinationChainId,uint256 fillDeadlineBuffer) depositParams, (bytes xcm, bytes[] assets, bytes claimer, uint128 executionFee, uint128 relayerFee) sendParams, address sourceAccount, bytes32 topic) payable",
    "function sendTokenAndCall((address inputToken,address outputToken,uint256 inputAmount,uint256 outputAmount,uint256 destinationChainId,uint256 fillDeadlineBuffer) depositParams, (uint256 inputAmount, address router, bytes callData) swapParams, (bytes xcm, bytes[] assets, bytes claimer, uint128 executionFee, uint128 relayerFee) sendParams, address sourceAccount, bytes32 topic)",
] as const

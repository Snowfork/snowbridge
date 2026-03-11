export type IERC20 = {
  balanceOf(owner: string): Promise<bigint>;
  allowance(owner: string, spender: string): Promise<bigint>;
};

export const IGATEWAY_V1_ABI = [
  "function sendToken(address tokenAddress, uint32 destinationParaId, (uint8 kind, bytes data) beneficiary, uint128 destinationFeeInDOT, uint128 amount) payable",
  "event OutboundMessageAccepted(bytes32 indexed channelId, uint64 nonce, bytes32 messageId)",
] as const;

export const IGATEWAY_V2_ABI = [
  "function v2_createAgent(bytes32 agentId) payable",
  "function v2_registerToken(address tokenAddress, uint8 network, uint128 executionFee, uint128 relayerFee) payable",
  "function v2_sendMessage(bytes xcm, bytes[] assets, bytes claimer, uint128 executionFee, uint128 relayerFee) payable",
  "event OutboundMessageAccepted(uint64 nonce, bytes payload)",
] as const;

export const IERC20_ABI = [
  "function balanceOf(address owner) view returns (uint256)",
  "function allowance(address owner, address spender) view returns (uint256)",
] as const;

export const SWAP_ROUTER_ABI = [
  "function exactOutputSingle((address tokenIn,address tokenOut,uint24 fee,address recipient,uint256 deadline,uint256 amountOut,uint256 amountInMaximum,uint160 sqrtPriceLimitX96) params) payable returns (uint256 amountIn)",
] as const;

export const SWAP_LEGACY_ROUTER_ABI = [
  "function exactOutputSingle((address tokenIn,address tokenOut,uint24 fee,address recipient,uint256 amountOut,uint256 amountInMaximum,uint160 sqrtPriceLimitX96) params) payable returns (uint256 amountIn)",
] as const;

export const SNOWBRIDGE_L1_ADAPTOR_ABI = [
  "function depositNativeEther((address inputToken,address outputToken,uint256 inputAmount,uint256 outputAmount,uint256 destinationChainId,uint256 fillDeadlineBuffer) params, address recipient, bytes32 topic) payable",
  "function depositToken((address inputToken,address outputToken,uint256 inputAmount,uint256 outputAmount,uint256 destinationChainId,uint256 fillDeadlineBuffer) params, address recipient, bytes32 topic)",
] as const;

export const SNOWBRIDGE_L2_ADAPTOR_ABI = [
  "function sendEtherAndCall((address inputToken,address outputToken,uint256 inputAmount,uint256 outputAmount,uint256 destinationChainId,uint256 fillDeadlineBuffer) depositParams, (bytes xcm, bytes[] assets, bytes claimer, uint128 executionFee, uint128 relayerFee) sendParams, address sourceAccount, bytes32 topic) payable",
  "function sendTokenAndCall((address inputToken,address outputToken,uint256 inputAmount,uint256 outputAmount,uint256 destinationChainId,uint256 fillDeadlineBuffer) depositParams, (uint256 inputAmount, address router, bytes callData) swapParams, (bytes xcm, bytes[] assets, bytes claimer, uint128 executionFee, uint128 relayerFee) sendParams, address sourceAccount, bytes32 topic)",
] as const;

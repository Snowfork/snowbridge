export type MultiAddressStruct = {
  kind: number;
  data: string;
};

export type GatewayV1OutboundMessageAccepted = {
  channelId: string;
  nonce: bigint;
  messageId: string;
  blockNumber: number;
  blockHash: string;
  txHash: string;
  txIndex: number;
};

export type GatewayV2OutboundMessageAccepted = {
  nonce: bigint;
  payload: string;
  blockNumber: number;
  blockHash: string;
  txHash: string;
  txIndex: number;
};

export type FeeData = {
  gasPrice: bigint | null;
  maxFeePerGas: bigint | null;
  maxPriorityFeePerGas: bigint | null;
};

export type EncodedMultiAddress = MultiAddressStruct;

export type L1AdapterDepositParams = {
  inputToken: string;
  outputToken: string;
  inputAmount: bigint;
  outputAmount: bigint;
  destinationChainId: number;
  fillDeadlineBuffer: bigint;
};

export type L1SwapRouterExactOutputSingleParams = {
  tokenIn: string;
  tokenOut: string;
  fee: bigint;
  recipient: string;
  deadline: bigint;
  amountOut: bigint;
  amountInMaximum: bigint;
  sqrtPriceLimitX96: bigint;
};

export type L1LegacySwapRouterExactOutputSingleParams = {
  tokenIn: string;
  tokenOut: string;
  fee: bigint;
  recipient: string;
  amountOut: bigint;
  amountInMaximum: bigint;
  sqrtPriceLimitX96: bigint;
};

export interface EthereumProviderTypes {
  Connection: unknown;
  Contract: unknown;
  Abi: unknown;
  TransactionReceipt: unknown;
  ContractTransaction: unknown;
}

export interface EthereumProvider<T extends EthereumProviderTypes> {
  readonly providerTypes: T;
  createProvider(url: string): T["Connection"];
  destroyProvider(provider: T["Connection"]): void;
  destroyContract(contract: T["Contract"]): Promise<void>;
  connectContract(
    address: string,
    abi: T["Abi"],
    provider: T["Connection"],
  ): T["Contract"];
  erc20Balance(
    provider: T["Connection"],
    tokenAddress: string,
    owner: string,
    spender: string,
  ): Promise<{ balance: bigint; gatewayAllowance: bigint }>;
  encodeFunctionData(
    abi: T["Abi"],
    method: string,
    args: readonly unknown[],
  ): string;
  decodeFunctionResult<U = unknown>(
    abi: T["Abi"],
    method: string,
    data: string,
  ): U;
  encodeNativeAsset(tokenAddress: string, amount: bigint): string;
  l1AdapterDepositNativeEther(
    params: L1AdapterDepositParams,
    recipient: string,
    topic: string,
  ): string;
  l1AdapterDepositToken(
    params: L1AdapterDepositParams,
    recipient: string,
    topic: string,
  ): string;
  l1SwapRouterExactOutputSingle(
    params: L1SwapRouterExactOutputSingleParams,
  ): string;
  l1LegacySwapRouterExactOutputSingle(
    params: L1LegacySwapRouterExactOutputSingleParams,
  ): string;
  beneficiaryMultiAddress(beneficiary: string): MultiAddressStruct;
  estimateGas(
    provider: T["Connection"],
    tx: T["ContractTransaction"],
  ): Promise<bigint>;
  getTransactionCount(
    provider: T["Connection"],
    address: string,
    blockTag?: "latest" | "pending",
  ): Promise<number>;
  gatewayV1SendToken(
    provider: T["Connection"],
    gatewayAddress: string,
    sourceAccount: string,
    tokenAddress: string,
    destinationParaId: number,
    beneficiary: MultiAddressStruct,
    totalFeeDot: bigint,
    amount: bigint,
    value: bigint,
  ): Promise<T["ContractTransaction"]>;
  gatewayV2RegisterToken(
    provider: T["Connection"],
    gatewayAddress: string,
    sourceAccount: string,
    tokenAddress: string,
    network: number,
    assetHubExecutionFeeEther: bigint,
    relayerFee: bigint,
    totalValue: bigint,
  ): Promise<T["ContractTransaction"]>;
  gatewayV2CreateAgent(
    provider: T["Connection"],
    gatewayAddress: string,
    agentId: string,
  ): Promise<T["ContractTransaction"]>;
  gatewayV2SendMessage(
    provider: T["Connection"],
    gatewayAddress: string,
    sourceAccount: string,
    xcm: Uint8Array,
    assets: string[],
    claimer: Uint8Array,
    assetHubExecutionFeeEther: bigint,
    relayerFee: bigint,
    value: bigint,
  ): Promise<T["ContractTransaction"]>;
  l2AdapterSendEtherAndCall(
    provider: T["Connection"],
    adapterAddress: string,
    sourceAccount: string,
    depositParams: any,
    sendParams: any,
    refundAddress: string,
    topic: string,
    value?: bigint,
  ): Promise<T["ContractTransaction"]>;
  l2AdapterSendTokenAndCall(
    provider: T["Connection"],
    adapterAddress: string,
    sourceAccount: string,
    depositParams: any,
    swapParams: any,
    sendParams: any,
    refundAddress: string,
    topic: string,
  ): Promise<T["ContractTransaction"]>;
  evmParachainTransferAssetsUsingTypeAndThenAddress(
    provider: T["Connection"],
    precompileAddress: string,
    sourceAccount: string,
    destination: [number, string[]],
    assets: [string, bigint][],
    assetsTransferType: number,
    remoteFeesIdIndex: number,
    feesTransferType: number,
    customXcmHex: string,
  ): Promise<T["ContractTransaction"]>;
  getBalance(provider: T["Connection"], address: string): Promise<bigint>;
  getFeeData(provider: T["Connection"]): Promise<FeeData>;
  parseUnits(value: string, decimals: number): bigint;
  isContractAddress(
    provider: T["Connection"],
    address: string,
  ): Promise<boolean>;
  scanGatewayV1OutboundMessageAccepted(
    receipt: T["TransactionReceipt"],
  ): GatewayV1OutboundMessageAccepted | null;
  scanGatewayV2OutboundMessageAccepted(
    receipt: T["TransactionReceipt"],
  ): GatewayV2OutboundMessageAccepted | null;
}

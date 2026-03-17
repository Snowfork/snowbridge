import {
  createPublicClient,
  decodeEventLog,
  decodeFunctionResult,
  encodeAbiParameters,
  encodeFunctionData,
  getAddress,
  http,
  parseAbi,
  parseAbiParameters,
  parseUnits,
  toHex,
  webSocket,
} from "viem";
import type {
  Abi,
  Address,
  Hex,
  Log,
  PublicClient,
  TransactionReceipt,
  TransactionRequest,
} from "viem";
import type {
  BeefyClient,
  DepositParamsStruct,
  EthereumProvider,
  EthereumProviderTypes,
  FeeData,
  GatewayV1OutboundMessageAccepted,
  GatewayV2OutboundMessageAccepted,
  IGatewayV1,
  IGatewayV2,
  IERC20,
  ISwapQuoter,
  L1AdapterDepositParams,
  L1LegacySwapRouterExactOutputSingleParams,
  L1SwapRouterExactOutputSingleParams,
  MultiAddressStruct,
  SendParamsStruct,
  SwapParamsStruct,
} from "@snowbridge/base-types";
import {
  IERC20_ABI,
  IGATEWAY_V1_ABI,
  IGATEWAY_V2_ABI,
  MOONBEAM_PALLET_XCM_PRECOMPILE_ABI,
  SNOWBRIDGE_L1_ADAPTOR_ABI,
  SNOWBRIDGE_L2_ADAPTOR_ABI,
  SWAP_LEGACY_ROUTER_ABI,
  SWAP_ROUTER_ABI,
} from "@snowbridge/base-types";

type ReadonlyFunction = ((...args: unknown[]) => Promise<unknown>) & {
  staticCall: (...args: unknown[]) => Promise<unknown>;
};

export type ViemContractTransaction = TransactionRequest & {
  account?: Address;
};

export type ViemContract = {
  getAddress(): Promise<Address>;
  removeAllListeners(): Promise<void>;
  [key: string]: unknown;
};

export interface ViemProviderTypes extends EthereumProviderTypes {
  Connection: PublicClient;
  Contract: ViemContract;
  Abi: Abi;
  TransactionReceipt: TransactionReceipt;
  ContractTransaction: ViemContractTransaction;
}

function toAddress(value: string): Address {
  return getAddress(value);
}

function asAbi<TAbi extends Abi>(abi: TAbi): TAbi;
function asAbi(abi: readonly [string, ...string[]]): Abi;
function asAbi(abi: Abi | readonly [string, ...string[]]): Abi;
function asAbi(abi: Abi | readonly [string, ...string[]]): Abi {
  if (
    Array.isArray(abi) &&
    abi.every((item) => typeof item === "string")
  ) {
    return parseAbi(abi);
  }
  return abi as Abi;
}

function toBytesHex(value: string | Uint8Array): Hex {
  return typeof value === "string" ? (value as Hex) : toHex(value);
}

function normalizeMultiAddress(address: MultiAddressStruct): MultiAddressStruct {
  return {
    kind: address.kind,
    data: address.data as Hex,
  };
}

function normalizeDepositParams<
  T extends { inputToken: string; outputToken: string },
>(params: T): T {
  return {
    ...params,
    inputToken: toAddress(params.inputToken),
    outputToken: toAddress(params.outputToken),
  };
}

function normalizeSendParams(sendParams: SendParamsStruct): {
  xcm: Hex;
  assets: Hex[];
  claimer: Hex;
  executionFee: bigint;
  relayerFee: bigint;
} {
  return {
    xcm: toBytesHex(sendParams.xcm),
    assets: sendParams.assets.map((asset) => asset as Hex),
    claimer: toBytesHex(sendParams.claimer),
    executionFee: sendParams.executionFee,
    relayerFee: sendParams.relayerFee,
  };
}

function normalizeSwapParams(swapParams: SwapParamsStruct): SwapParamsStruct & {
  callData: Hex;
} {
  return {
    ...swapParams,
    router: toAddress(swapParams.router),
    callData: swapParams.callData as Hex,
  };
}

function normalizeDestination(destination: [number, string[]]): [number, Hex[]] {
  return [destination[0], destination[1].map((item) => item as Hex)];
}

function createReadOnlyContract(
  address: Address,
  abi: Abi | readonly [string, ...string[]],
  provider: PublicClient,
): ViemContract {
  const normalizedAbi = asAbi(abi);
  return new Proxy(
    {
      async getAddress() {
        return address;
      },
      async removeAllListeners() {},
    } as ViemContract,
    {
      get(target, prop, receiver) {
        if (typeof prop !== "string") {
          return Reflect.get(target, prop, receiver);
        }
        if (prop in target) {
          return Reflect.get(target, prop, receiver);
        }

        const fn = (async (...args: unknown[]) => {
          return await provider.readContract({
            address,
            abi: normalizedAbi,
            functionName: prop,
            args,
          } as never);
        }) as ReadonlyFunction;

        fn.staticCall = async (...args: unknown[]) => {
          const result = await provider.simulateContract({
            address,
            abi: normalizedAbi,
            functionName: prop,
            args,
          } as never);
          return result.result;
        };

        return fn;
      },
    },
  );
}

export class ViemEthereumProvider
  implements EthereumProvider<ViemProviderTypes>
{
  declare readonly providerTypes: ViemProviderTypes;

  createProvider(url: string): PublicClient {
    return createPublicClient({
      transport: url.startsWith("http") ? http(url) : webSocket(url),
    });
  }

  destroyProvider(provider: PublicClient): void {
    const transport = provider.transport as { value?: { close?: () => void } };
    transport.value?.close?.();
  }

  async destroyContract(_contract: ViemContract): Promise<void> {}

  connectContract(
    address: string,
    abi: Abi,
    provider: PublicClient,
  ): ViemContract {
    return createReadOnlyContract(toAddress(address), asAbi(abi), provider);
  }

  async erc20Balance(
    provider: PublicClient,
    tokenAddress: string,
    owner: string,
    spender: string,
  ): Promise<{ balance: bigint; gatewayAllowance: bigint }> {
    const [balance, gatewayAllowance] = await Promise.all([
      provider.readContract({
        address: toAddress(tokenAddress),
        abi: asAbi(IERC20_ABI),
        functionName: "balanceOf",
        args: [toAddress(owner)],
      }),
      provider.readContract({
        address: toAddress(tokenAddress),
        abi: asAbi(IERC20_ABI),
        functionName: "allowance",
        args: [toAddress(owner), toAddress(spender)],
      }),
    ]);
    return {
      balance: balance as bigint,
      gatewayAllowance: gatewayAllowance as bigint,
    };
  }

  async gatewayV1SendToken(
    _provider: PublicClient,
    gatewayAddress: string,
    sender: string,
    token: string,
    destinationChain: number,
    destinationAddress: MultiAddressStruct,
    destinationFee: bigint,
    amount: bigint,
    value: bigint,
  ): Promise<ViemContractTransaction> {
    return {
      to: toAddress(gatewayAddress),
      account: toAddress(sender),
      value,
      data: this.encodeFunctionData(asAbi(IGATEWAY_V1_ABI), "sendToken", [
        toAddress(token),
        BigInt(destinationChain),
        normalizeMultiAddress(destinationAddress),
        destinationFee,
        amount,
      ]),
    };
  }

  async gatewayV2RegisterToken(
    _provider: PublicClient,
    gatewayAddress: string,
    sender: string,
    token: string,
    network: number,
    executionFee: bigint,
    relayerFee: bigint,
    value: bigint,
  ): Promise<ViemContractTransaction> {
    return {
      to: toAddress(gatewayAddress),
      account: toAddress(sender),
      value,
      data: this.encodeFunctionData(
        asAbi(IGATEWAY_V2_ABI),
        "v2_registerToken",
        [toAddress(token), BigInt(network), executionFee, relayerFee],
      ),
    };
  }

  async gatewayV2CreateAgent(
    _provider: PublicClient,
    gatewayAddress: string,
    id: string,
  ): Promise<ViemContractTransaction> {
    return {
      to: toAddress(gatewayAddress),
      data: this.encodeFunctionData(asAbi(IGATEWAY_V2_ABI), "v2_createAgent", [
        id as Hex,
      ]),
    };
  }

  async gatewayV2SendMessage(
    _provider: PublicClient,
    gatewayAddress: string,
    sender: string,
    xcm: Uint8Array,
    assets: string[],
    claimer: Uint8Array,
    executionFee: bigint,
    relayerFee: bigint,
    value: bigint,
  ): Promise<ViemContractTransaction> {
    return {
      to: toAddress(gatewayAddress),
      account: toAddress(sender),
      value,
      data: this.encodeFunctionData(asAbi(IGATEWAY_V2_ABI), "v2_sendMessage", [
        toBytesHex(xcm),
        assets.map((asset) => asset as Hex),
        toBytesHex(claimer),
        executionFee,
        relayerFee,
      ]),
    };
  }

  async l2AdapterSendEtherAndCall(
    _provider: PublicClient,
    adapterAddress: string,
    sender: string,
    params: DepositParamsStruct,
    sendParams: SendParamsStruct,
    recipient: string,
    topic: string,
    value?: bigint,
  ): Promise<ViemContractTransaction> {
    return {
      to: toAddress(adapterAddress),
      account: toAddress(sender),
      value,
      data: this.encodeFunctionData(
        asAbi(SNOWBRIDGE_L2_ADAPTOR_ABI),
        "sendEtherAndCall",
        [
          normalizeDepositParams(params),
          normalizeSendParams(sendParams),
          toAddress(recipient),
          topic as Hex,
        ],
      ),
    };
  }

  async l2AdapterSendTokenAndCall(
    _provider: PublicClient,
    adapterAddress: string,
    sender: string,
    params: DepositParamsStruct,
    swapParams: SwapParamsStruct,
    sendParams: SendParamsStruct,
    recipient: string,
    topic: string,
  ): Promise<ViemContractTransaction> {
    return {
      to: toAddress(adapterAddress),
      account: toAddress(sender),
      data: this.encodeFunctionData(
        asAbi(SNOWBRIDGE_L2_ADAPTOR_ABI),
        "sendTokenAndCall",
        [
          normalizeDepositParams(params),
          normalizeSwapParams(swapParams),
          normalizeSendParams(sendParams),
          toAddress(recipient),
          topic as Hex,
        ],
      ),
    };
  }

  async evmParachainTransferAssetsUsingTypeAndThenAddress(
    _provider: PublicClient,
    precompileAddress: string,
    sourceAccount: string,
    destination: [number, string[]],
    assets: [string, bigint][],
    assetsTransferType: number,
    remoteFeesIdIndex: number,
    feesTransferType: number,
    customXcmHex: string,
  ): Promise<ViemContractTransaction> {
    return {
      to: toAddress(precompileAddress),
      account: toAddress(sourceAccount),
      data: this.encodeFunctionData(
        asAbi(MOONBEAM_PALLET_XCM_PRECOMPILE_ABI),
        "transferAssetsUsingTypeAndThenAddress",
        [
          normalizeDestination(destination),
          assets.map(([asset, amount]) => [toAddress(asset), amount]),
          assetsTransferType,
          remoteFeesIdIndex,
          feesTransferType,
          customXcmHex as Hex,
        ],
      ),
    };
  }

  encodeFunctionData(
    abi: Abi | readonly [string, ...string[]],
    method: string,
    args: readonly unknown[],
  ): Hex {
    return encodeFunctionData({
      abi: asAbi(abi),
      functionName: method,
      args: [...args],
    } as never);
  }

  decodeFunctionResult<T = unknown>(
    abi: Abi | readonly [string, ...string[]],
    method: string,
    data: string,
  ): T {
    return decodeFunctionResult({
      abi: asAbi(abi),
      functionName: method,
      data: data as Hex,
    } as never) as T;
  }

  encodeNativeAsset(tokenAddress: string, amount: bigint): Hex {
    return encodeAbiParameters(parseAbiParameters("uint8,address,uint128"), [
      0,
      toAddress(tokenAddress),
      amount,
    ]);
  }

  l1AdapterDepositNativeEther(
    params: L1AdapterDepositParams,
    recipient: string,
    topic: string,
  ): Hex {
    return this.encodeFunctionData(
      asAbi(SNOWBRIDGE_L1_ADAPTOR_ABI),
      "depositNativeEther",
      [normalizeDepositParams(params), toAddress(recipient), topic as Hex],
    );
  }

  l1AdapterDepositToken(
    params: L1AdapterDepositParams,
    recipient: string,
    topic: string,
  ): Hex {
    return this.encodeFunctionData(
      asAbi(SNOWBRIDGE_L1_ADAPTOR_ABI),
      "depositToken",
      [normalizeDepositParams(params), toAddress(recipient), topic as Hex],
    );
  }

  l1SwapRouterExactOutputSingle(
    params: L1SwapRouterExactOutputSingleParams,
  ): Hex {
    return this.encodeFunctionData(
      asAbi(SWAP_ROUTER_ABI),
      "exactOutputSingle",
      [
        {
          ...params,
          tokenIn: toAddress(params.tokenIn),
          tokenOut: toAddress(params.tokenOut),
          recipient: toAddress(params.recipient),
        },
      ],
    );
  }

  l1LegacySwapRouterExactOutputSingle(
    params: L1LegacySwapRouterExactOutputSingleParams,
  ): Hex {
    return this.encodeFunctionData(
      asAbi(SWAP_LEGACY_ROUTER_ABI),
      "exactOutputSingle",
      [
        {
          ...params,
          tokenIn: toAddress(params.tokenIn),
          tokenOut: toAddress(params.tokenOut),
          recipient: toAddress(params.recipient),
        },
      ],
    );
  }

  beneficiaryMultiAddress(beneficiary: string): MultiAddressStruct {
    let kind: number;
    if (/^0x[a-fA-F0-9]{40}$/.test(beneficiary)) {
      kind = 2;
    } else if (/^0x[a-fA-F0-9]{64}$/.test(beneficiary)) {
      kind = 1;
    } else {
      throw new Error("Unknown Beneficiary address format.");
    }
    let data: Hex;
    switch (kind) {
      case 1:
        data = encodeAbiParameters(parseAbiParameters("bytes32"), [
          beneficiary as Hex,
        ]);
        break;
      case 2:
        data = encodeAbiParameters(parseAbiParameters("bytes20"), [
          beneficiary as Hex,
        ]);
        break;
      default:
        throw new Error(`Unknown Beneficiary kind ${kind}.`);
    }
    return { kind, data };
  }

  async estimateGas(
    provider: PublicClient,
    tx: ViemContractTransaction,
  ): Promise<bigint> {
    return await provider.estimateGas(tx);
  }

  async getTransactionCount(
    provider: PublicClient,
    address: string,
    blockTag: "latest" | "pending" = "latest",
  ): Promise<number> {
    return Number(
      await provider.getTransactionCount({
        address: toAddress(address),
        blockTag,
      }),
    );
  }

  async getBalance(provider: PublicClient, address: string): Promise<bigint> {
    return await provider.getBalance({ address: toAddress(address) });
  }

  async getFeeData(provider: PublicClient): Promise<FeeData> {
    const gasPrice = await provider.getGasPrice();
    try {
      const fees = await provider.estimateFeesPerGas();
      return {
        gasPrice: fees.gasPrice ?? gasPrice,
        maxFeePerGas: fees.maxFeePerGas ?? null,
        maxPriorityFeePerGas: fees.maxPriorityFeePerGas ?? null,
      };
    } catch {
      return {
        gasPrice,
        maxFeePerGas: null,
        maxPriorityFeePerGas: null,
      };
    }
  }

  parseUnits(value: string, decimals: number): bigint {
    return parseUnits(value, decimals);
  }

  async gatewayOperatingMode(
    gateway: ViemContract & (IGatewayV1 | IGatewayV2),
  ): Promise<bigint> {
    return BigInt(await gateway.operatingMode());
  }

  async gatewayChannelOperatingModeOf(
    gateway: ViemContract & IGatewayV1,
    channelId: string,
  ): Promise<bigint> {
    return BigInt(await gateway.channelOperatingModeOf(channelId));
  }

  async isContractAddress(
    provider: PublicClient,
    address: string,
  ): Promise<boolean> {
    if (!/^0x[a-fA-F0-9]{40}$/.test(address)) {
      return false;
    }
    try {
      const code = await provider.getCode({ address: toAddress(address) });
      return code !== undefined && code !== "0x";
    } catch {
      return false;
    }
  }

  scanGatewayV1OutboundMessageAccepted(
    receipt: TransactionReceipt,
  ): GatewayV1OutboundMessageAccepted | null {
    for (const log of receipt.logs as Log[]) {
      try {
        const event = decodeEventLog({
          abi: asAbi(IGATEWAY_V1_ABI),
          topics: log.topics,
          data: log.data,
        });
        if (event.eventName === "OutboundMessageAccepted") {
          const args = event.args as unknown as readonly unknown[] | undefined;
          if (!args) {
            continue;
          }
          return {
            channelId: String(args[0]),
            nonce: BigInt(args[1] as bigint),
            messageId: String(args[2]),
            blockHash: receipt.blockHash,
            blockNumber: Number(receipt.blockNumber),
            txHash: receipt.transactionHash,
            txIndex: Number(receipt.transactionIndex),
          };
        }
      } catch {}
    }
    return null;
  }

  scanGatewayV2OutboundMessageAccepted(
    receipt: TransactionReceipt,
  ): GatewayV2OutboundMessageAccepted | null {
    for (const log of receipt.logs as Log[]) {
      try {
        const event = decodeEventLog({
          abi: asAbi(IGATEWAY_V2_ABI),
          topics: log.topics,
          data: log.data,
        });
        if (event.eventName === "OutboundMessageAccepted") {
          const args = event.args as unknown as
            | {
                nonce: bigint;
                payload: {
                  origin: Address;
                  assets: { kind: number; data: Hex }[];
                  xcm: { kind: number; data: Hex };
                  claimer: Hex;
                  value: bigint;
                  executionFee: bigint;
                  relayerFee: bigint;
                };
              }
            | undefined;
          if (!args) {
            continue;
          }
          return {
            nonce: BigInt(args.nonce),
            payload: {
              origin: args.payload.origin,
              assets: args.payload.assets.map((asset) => [
                Number(asset.kind),
                asset.data,
              ]),
              xcm: [Number(args.payload.xcm.kind), args.payload.xcm.data],
              claimer: args.payload.claimer,
              value: BigInt(args.payload.value),
              executionFee: BigInt(args.payload.executionFee),
              relayerFee: BigInt(args.payload.relayerFee),
            },
            blockHash: receipt.blockHash,
            blockNumber: Number(receipt.blockNumber),
            txHash: receipt.transactionHash,
            txIndex: Number(receipt.transactionIndex),
          };
        }
      } catch {}
    }
    return null;
  }
}

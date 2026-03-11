import {
  createPublicClient,
  decodeEventLog,
  decodeFunctionResult,
  encodeAbiParameters,
  encodeFunctionData,
  getAddress,
  http,
  parseAbiParameters,
  parseUnits,
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
  EncodedMultiAddress,
  EthereumProvider,
  EthereumProviderTypes,
  FeeData,
  GatewayV1OutboundMessageAccepted,
  GatewayV2OutboundMessageAccepted,
  IERC20,
  IGatewayV1,
  IGatewayV2,
  ISwapQuoter,
  L1AdapterDepositParams,
  L1LegacySwapRouterExactOutputSingleParams,
  L1SwapRouterExactOutputSingleParams,
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

function resolveBeneficiary(address: string) {
  if (/^0x[a-fA-F0-9]{40}$/.test(address)) {
    return {
      hexAddress: address,
      kind: 2,
    };
  }
  if (/^0x[a-fA-F0-9]{64}$/.test(address)) {
    return {
      hexAddress: address,
      kind: 1,
    };
  }
  throw new Error("Unknown Beneficiary address format.");
}

type ReadonlyFunction = ((...args: unknown[]) => Promise<unknown>) & {
  staticCall: (...args: unknown[]) => Promise<unknown>;
};

export type ViemContractTransaction = TransactionRequest & {
  account?: Address;
  from?: Address;
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

function asAbi(abi: unknown): Abi {
  return abi as unknown as Abi;
}

function asTxRequest(
  tx: ViemContractTransaction,
): TransactionRequest & { account?: Address } {
  const account = tx.account ?? tx.from;
  return account ? { ...tx, account } : tx;
}

function createReadOnlyContract(
  address: Address,
  abi: Abi,
  provider: PublicClient,
): ViemContract {
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
            abi,
            functionName: prop,
            args,
          } as never);
        }) as ReadonlyFunction;

        fn.staticCall = async (...args: unknown[]) => {
          const result = await provider.simulateContract({
            address,
            abi,
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
    return createReadOnlyContract(toAddress(address), abi, provider);
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
    sourceAccount: string,
    tokenAddress: string,
    destinationParaId: number,
    beneficiary: EncodedMultiAddress["address"],
    totalFeeDot: bigint,
    amount: bigint,
    value: bigint,
  ): Promise<ViemContractTransaction> {
    return {
      to: toAddress(gatewayAddress),
      account: toAddress(sourceAccount),
      from: toAddress(sourceAccount),
      value,
      data: this.encodeFunctionData(asAbi(IGATEWAY_V1_ABI), "sendToken", [
        toAddress(tokenAddress),
        BigInt(destinationParaId),
        beneficiary,
        totalFeeDot,
        amount,
      ]),
    };
  }

  async gatewayV2RegisterToken(
    _provider: PublicClient,
    gatewayAddress: string,
    sourceAccount: string,
    tokenAddress: string,
    network: number,
    assetHubExecutionFeeEther: bigint,
    relayerFee: bigint,
    totalValue: bigint,
  ): Promise<ViemContractTransaction> {
    return {
      to: toAddress(gatewayAddress),
      account: toAddress(sourceAccount),
      from: toAddress(sourceAccount),
      value: totalValue,
      data: this.encodeFunctionData(
        asAbi(IGATEWAY_V2_ABI),
        "v2_registerToken",
        [
          toAddress(tokenAddress),
          BigInt(network),
          assetHubExecutionFeeEther,
          relayerFee,
        ],
      ),
    };
  }

  async gatewayV2CreateAgent(
    _provider: PublicClient,
    gatewayAddress: string,
    agentId: string,
  ): Promise<ViemContractTransaction> {
    return {
      to: toAddress(gatewayAddress),
      data: this.encodeFunctionData(asAbi(IGATEWAY_V2_ABI), "v2_createAgent", [
        agentId as Hex,
      ]),
    };
  }

  async gatewayV2SendMessage(
    _provider: PublicClient,
    gatewayAddress: string,
    sourceAccount: string,
    xcm: Uint8Array,
    assets: string[],
    claimer: Uint8Array,
    assetHubExecutionFeeEther: bigint,
    relayerFee: bigint,
    value: bigint,
  ): Promise<ViemContractTransaction> {
    return {
      to: toAddress(gatewayAddress),
      account: toAddress(sourceAccount),
      from: toAddress(sourceAccount),
      value,
      data: this.encodeFunctionData(asAbi(IGATEWAY_V2_ABI), "v2_sendMessage", [
        xcm,
        assets.map(toAddress),
        claimer,
        assetHubExecutionFeeEther,
        relayerFee,
      ]),
    };
  }

  async l2AdapterSendEtherAndCall(
    _provider: PublicClient,
    adapterAddress: string,
    sourceAccount: string,
    depositParams: any,
    sendParams: any,
    refundAddress: string,
    topic: string,
    value?: bigint,
  ): Promise<ViemContractTransaction> {
    return {
      to: toAddress(adapterAddress),
      account: toAddress(sourceAccount),
      from: toAddress(sourceAccount),
      value,
      data: this.encodeFunctionData(
        asAbi(SNOWBRIDGE_L2_ADAPTOR_ABI),
        "sendEtherAndCall",
        [depositParams, sendParams, toAddress(refundAddress), topic as Hex],
      ),
    };
  }

  async l2AdapterSendTokenAndCall(
    _provider: PublicClient,
    adapterAddress: string,
    sourceAccount: string,
    depositParams: any,
    swapParams: any,
    sendParams: any,
    refundAddress: string,
    topic: string,
  ): Promise<ViemContractTransaction> {
    return {
      to: toAddress(adapterAddress),
      account: toAddress(sourceAccount),
      from: toAddress(sourceAccount),
      data: this.encodeFunctionData(
        asAbi(SNOWBRIDGE_L2_ADAPTOR_ABI),
        "sendTokenAndCall",
        [
          depositParams,
          swapParams,
          sendParams,
          toAddress(refundAddress),
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
      from: toAddress(sourceAccount),
      data: this.encodeFunctionData(
        asAbi(MOONBEAM_PALLET_XCM_PRECOMPILE_ABI),
        "transferAssetsUsingTypeAndThenAddress",
        [
          destination,
          assets.map(([asset, amount]) => [toAddress(asset), amount]),
          assetsTransferType,
          remoteFeesIdIndex,
          feesTransferType,
          customXcmHex as Hex,
        ],
      ),
    };
  }

  encodeFunctionData(abi: Abi, method: string, args: readonly unknown[]): Hex {
    return encodeFunctionData({
      abi,
      functionName: method,
      args: [...args],
    } as never);
  }

  decodeFunctionResult<T = unknown>(abi: Abi, method: string, data: string): T {
    return decodeFunctionResult({
      abi,
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
      [params, toAddress(recipient), topic as Hex],
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
      [params, toAddress(recipient), topic as Hex],
    );
  }

  l1SwapRouterExactOutputSingle(
    params: L1SwapRouterExactOutputSingleParams,
  ): Hex {
    return this.encodeFunctionData(
      asAbi(SWAP_ROUTER_ABI),
      "exactOutputSingle",
      [params],
    );
  }

  l1LegacySwapRouterExactOutputSingle(
    params: L1LegacySwapRouterExactOutputSingleParams,
  ): Hex {
    return this.encodeFunctionData(
      asAbi(SWAP_LEGACY_ROUTER_ABI),
      "exactOutputSingle",
      [params],
    );
  }

  beneficiaryMultiAddress(beneficiary: string): EncodedMultiAddress {
    const { hexAddress, kind } = resolveBeneficiary(beneficiary);
    let data: Hex;
    switch (kind) {
      case 1:
        data = encodeAbiParameters(parseAbiParameters("bytes32"), [
          hexAddress as Hex,
        ]);
        break;
      case 2:
        data = encodeAbiParameters(parseAbiParameters("bytes20"), [
          hexAddress as Hex,
        ]);
        break;
      default:
        throw new Error(`Unknown Beneficiary kind ${kind}.`);
    }
    return {
      hexAddress,
      address: {
        kind,
        data,
      },
    };
  }

  async estimateGas(
    provider: PublicClient,
    tx: ViemContractTransaction,
  ): Promise<bigint> {
    return await provider.estimateGas(asTxRequest(tx));
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

  async isContractAddress(
    provider: PublicClient,
    address: string,
  ): Promise<boolean> {
    if (!/^0x[a-fA-F0-9]{40}$/.test(address)) {
      return false;
    }
    try {
      const code = await provider.getBytecode({ address: toAddress(address) });
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
          const args = event.args as readonly unknown[] | undefined;
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
          const args = event.args as readonly unknown[] | undefined;
          if (!args) {
            continue;
          }
          return {
            nonce: BigInt(args[0] as bigint),
            payload: args[1] as Hex,
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

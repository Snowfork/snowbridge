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
} from "ethers";
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

export interface EthersProviderTypes extends EthereumProviderTypes {
  Connection: AbstractProvider;
  Contract: Contract;
  Abi: InterfaceAbi;
  Interface: Interface;
  TransactionReceipt: TransactionReceipt;
  ContractTransaction: ContractTransaction;
}

export class EthersEthereumProvider
  implements EthereumProvider<EthersProviderTypes>
{
  declare readonly providerTypes: EthersProviderTypes;

  static gatewayV1Interface = new Interface(IGATEWAY_V1_ABI);
  static gatewayV2Interface = new Interface(IGATEWAY_V2_ABI);

  createProvider(url: string): AbstractProvider {
    if (url.startsWith("http")) {
      return new JsonRpcProvider(url);
    }
    return new WebSocketProvider(url);
  }

  destroyProvider(provider: AbstractProvider): void {
    provider.destroy();
  }

  async destroyContract(contract: Contract): Promise<void> {
    await contract.removeAllListeners();
  }

  connectContract(
    address: string,
    abi: InterfaceAbi,
    provider: AbstractProvider,
  ): Contract {
    return new Contract(address, abi, provider);
  }

  async erc20Balance(
    provider: AbstractProvider,
    tokenAddress: string,
    owner: string,
    spender: string,
  ): Promise<{ balance: bigint; gatewayAllowance: bigint }> {
    const tokenContract = this.connectContract(
      tokenAddress,
      IERC20_ABI,
      provider,
    ) as Contract & IERC20;
    const [balance, gatewayAllowance] = await Promise.all([
      tokenContract.balanceOf(owner),
      tokenContract.allowance(owner, spender),
    ]);
    return { balance, gatewayAllowance };
  }

  async gatewayV1SendToken(
    provider: AbstractProvider,
    gatewayAddress: string,
    sourceAccount: string,
    tokenAddress: string,
    destinationParaId: number,
    beneficiary: EncodedMultiAddress["address"],
    totalFeeDot: bigint,
    amount: bigint,
    value: bigint,
  ): Promise<ContractTransaction> {
    const gateway = this.connectContract(
      gatewayAddress,
      IGATEWAY_V1_ABI,
      provider,
    );
    return await gateway
      .getFunction("sendToken")
      .populateTransaction(
        tokenAddress,
        destinationParaId,
        beneficiary,
        totalFeeDot,
        amount,
        { value, from: sourceAccount },
      );
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
    const gateway = this.connectContract(
      gatewayAddress,
      IGATEWAY_V2_ABI,
      provider,
    );
    return await gateway
      .getFunction("v2_registerToken")
      .populateTransaction(
        tokenAddress,
        network,
        assetHubExecutionFeeEther,
        relayerFee,
        {
          value: totalValue,
          from: sourceAccount,
        },
      );
  }

  async gatewayV2CreateAgent(
    provider: AbstractProvider,
    gatewayAddress: string,
    agentId: string,
  ): Promise<ContractTransaction> {
    const gateway = this.connectContract(
      gatewayAddress,
      IGATEWAY_V2_ABI,
      provider,
    );
    return await gateway
      .getFunction("v2_createAgent")
      .populateTransaction(agentId);
  }

  async gatewayV2SendMessage(
    provider: AbstractProvider,
    gatewayAddress: string,
    sourceAccount: string,
    xcm: Uint8Array,
    assets: string[],
    claimer: Uint8Array,
    assetHubExecutionFeeEther: bigint,
    relayerFee: bigint,
    value: bigint,
  ): Promise<ContractTransaction> {
    const gateway = this.connectContract(
      gatewayAddress,
      IGATEWAY_V2_ABI,
      provider,
    );
    return await gateway
      .getFunction("v2_sendMessage")
      .populateTransaction(
        xcm,
        assets,
        claimer,
        assetHubExecutionFeeEther,
        relayerFee,
        {
          value,
          from: sourceAccount,
        },
      );
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
    const adapter = this.connectContract(
      adapterAddress,
      SNOWBRIDGE_L2_ADAPTOR_ABI,
      provider,
    );
    const txOptions =
      value === undefined
        ? { from: sourceAccount }
        : { from: sourceAccount, value };
    return await adapter
      .getFunction("sendEtherAndCall")
      .populateTransaction(
        depositParams,
        sendParams,
        refundAddress,
        topic,
        txOptions,
      );
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
    const adapter = this.connectContract(
      adapterAddress,
      SNOWBRIDGE_L2_ADAPTOR_ABI,
      provider,
    );
    return await adapter
      .getFunction("sendTokenAndCall")
      .populateTransaction(
        depositParams,
        swapParams,
        sendParams,
        refundAddress,
        topic,
        {
          from: sourceAccount,
        },
      );
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
    const precompile = this.connectContract(
      precompileAddress,
      MOONBEAM_PALLET_XCM_PRECOMPILE_ABI,
      provider,
    );
    const tx = await precompile
      .getFunction(
        "transferAssetsUsingTypeAndThenAddress((uint8,bytes[]),(address,uint256)[],uint8,uint8,uint8,bytes)",
      )
      .populateTransaction(
        destination,
        assets,
        assetsTransferType,
        remoteFeesIdIndex,
        feesTransferType,
        customXcmHex,
      );
    tx.from = sourceAccount;
    return tx;
  }

  encodeFunctionData(
    abi: InterfaceAbi,
    method: string,
    args: readonly unknown[],
  ): string {
    return new Interface(abi).encodeFunctionData(method, args);
  }

  decodeFunctionResult<T = unknown>(
    abi: InterfaceAbi,
    method: string,
    data: string,
  ): T {
    return new Interface(abi).decodeFunctionResult(method, data) as T;
  }

  encodeNativeAsset(tokenAddress: string, amount: bigint): string {
    return AbiCoder.defaultAbiCoder().encode(
      ["uint8", "address", "uint128"],
      [0, tokenAddress, amount],
    );
  }

  l1AdapterDepositNativeEther(
    params: L1AdapterDepositParams,
    recipient: string,
    topic: string,
  ): string {
    return new Interface(SNOWBRIDGE_L1_ADAPTOR_ABI).encodeFunctionData(
      "depositNativeEther",
      [params, recipient, topic],
    );
  }

  l1AdapterDepositToken(
    params: L1AdapterDepositParams,
    recipient: string,
    topic: string,
  ): string {
    return new Interface(SNOWBRIDGE_L1_ADAPTOR_ABI).encodeFunctionData(
      "depositToken",
      [params, recipient, topic],
    );
  }

  l1SwapRouterExactOutputSingle(
    params: L1SwapRouterExactOutputSingleParams,
  ): string {
    return new Interface(SWAP_ROUTER_ABI).encodeFunctionData(
      "exactOutputSingle",
      [params],
    );
  }

  l1LegacySwapRouterExactOutputSingle(
    params: L1LegacySwapRouterExactOutputSingleParams,
  ): string {
    return new Interface(SWAP_LEGACY_ROUTER_ABI).encodeFunctionData(
      "exactOutputSingle",
      [params],
    );
  }

  beneficiaryMultiAddress(beneficiary: string): EncodedMultiAddress {
    const abi = AbiCoder.defaultAbiCoder();
    const { hexAddress, kind } = resolveBeneficiary(beneficiary);
    let data: string;
    switch (kind) {
      case 1:
        data = abi.encode(["bytes32"], [hexAddress]);
        break;
      case 2:
        data = abi.encode(["bytes20"], [hexAddress]);
        break;
      default:
        throw new Error(`Unknown Beneficiary kind {kind}.`);
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
    provider: AbstractProvider,
    tx: ContractTransaction,
  ): Promise<bigint> {
    return await provider.estimateGas(tx);
  }

  async getTransactionCount(
    provider: AbstractProvider,
    address: string,
    blockTag: "latest" | "pending" = "latest",
  ): Promise<number> {
    return await provider.getTransactionCount(address, blockTag);
  }

  async getBalance(
    provider: AbstractProvider,
    address: string,
  ): Promise<bigint> {
    return await provider.getBalance(address);
  }

  async getFeeData(provider: AbstractProvider): Promise<FeeData> {
    const feeData: EthersFeeData = await provider.getFeeData();
    return {
      gasPrice: feeData.gasPrice ?? null,
      maxFeePerGas: feeData.maxFeePerGas ?? null,
      maxPriorityFeePerGas: feeData.maxPriorityFeePerGas ?? null,
    };
  }

  parseUnits(value: string, decimals: number): bigint {
    return parseUnits(value, decimals);
  }

  async isContractAddress(
    provider: AbstractProvider,
    address: string,
  ): Promise<boolean> {
    if (!/^0x[a-fA-F0-9]{40}$/.test(address)) {
      return false;
    }
    try {
      const code = await provider.getCode(address);
      return code !== "0x";
    } catch {
      return false;
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
        });
        if (event && event.name === "OutboundMessageAccepted") {
          return {
            channelId: String(event.args[0]),
            nonce: BigInt(event.args[1]),
            messageId: String(event.args[2]),
            blockHash: receipt.blockHash,
            blockNumber: receipt.blockNumber,
            txHash: receipt.hash,
            txIndex: receipt.index,
          };
        }
      } catch {}
    }
    return null;
  }

  scanGatewayV2OutboundMessageAccepted(
    receipt: TransactionReceipt,
  ): GatewayV2OutboundMessageAccepted | null {
    for (const log of receipt.logs) {
      try {
        const event = EthersEthereumProvider.gatewayV2Interface.parseLog({
          topics: [...log.topics],
          data: log.data,
        });
        if (event && event.name === "OutboundMessageAccepted") {
          return {
            nonce: BigInt(event.args[0]),
            payload: event.args[1],
            blockHash: receipt.blockHash,
            blockNumber: receipt.blockNumber,
            txHash: receipt.hash,
            txIndex: receipt.index,
          };
        }
      } catch {}
    }
    return null;
  }
}

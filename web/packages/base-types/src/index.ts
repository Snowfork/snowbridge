export type BridgeInfo = {
  environment: Environment;
  routes: readonly TransferRoute[];
  registry: AssetRegistry;
};

export type AccountType = "AccountId20" | "AccountId32";

export type XcmVersion = "v4" | "v5";

export type EthereumKind = "ethereum" | "ethereum_l2";
export type ParachainKind = "polkadot" | "kusama";
export type ChainKind = EthereumKind | ParachainKind;

export type XC20TokenMap = Record<string, string>;

export type ERC20Metadata = {
  token: string;
  name: string;
  symbol: string;
  decimals: number;
  foreignId?: string;
  // The gas cost of a local transfer, which involves unlocking for ENA and minting for PNA.
  deliveryGas?: bigint;
  // For ERC-20 tokens on L2 chains that have a corresponding mapped L1 token address.
  swapTokenAddress?: string;
  // fee tier for uniswap call in basis points (e.g., 500 = 0.05%)
  swapFee?: number;
};

export type ERC20MetadataMap = Record<string, ERC20Metadata>;

export type EthereumChain = ChainId & {
  kind: EthereumKind;
  key: `${EthereumKind}_${number}`;
  name?: string;
  evmParachainId?: number;
  assets: ERC20MetadataMap;
  precompile?: `0x${string}`;
  xcDOT?: string;
  xcTokenMap?: XC20TokenMap;
  // The gas cost of v2_submit excludes command execution, mainly covers the verification.
  baseDeliveryGas?: bigint;
};

export type ChainProperties = {
  tokenSymbols: string;
  tokenDecimals: number;
  ss58Format: number;
  isEthereum: boolean;
  accountType: AccountType;
  evmChainId?: number;
  name: string;
  specName: string;
  specVersion: number;
};

export type Asset = {
  token: string;
  name: string;
  minimumBalance: bigint;
  symbol: string;
  decimals: number;
  isSufficient: boolean;
  xc20?: string;
  // Location on source Parachain
  location?: any;
  // Location reanchored on AH
  locationOnAH?: any;
  // Location reanchored on Ethereum
  locationOnEthereum?: any;
  // For chains that use `Assets` pallet to manage local assets
  // the asset_id is normally represented as u32, but on Moonbeam,
  // it is u128, so use string here to avoid overflow
  assetId?: string;
  // Identifier of the PNA
  foreignId?: string;
};

export type AssetMap = Record<string, Asset>;

export type Parachain = ChainId & {
  kind: ParachainKind;
  key: `${ParachainKind}_${number}`;
  info: ChainProperties;
  features: {
    hasPalletXcm: boolean;
    hasDryRunApi: boolean;
    hasTxPaymentApi: boolean;
    hasDryRunRpc: boolean;
    hasDotBalance: boolean;
    hasEthBalance: boolean;
    hasXcmPaymentApi: boolean;
    supportsAliasOrigin: boolean;
    xcmVersion: XcmVersion;
    /** @deprecated Remove once V2 is fully rolled out to all parachains */
    supportsV2: boolean;
  };
  assets: AssetMap;
  estimatedExecutionFeeDOT: bigint;
  estimatedDeliveryFeeDOT: bigint;
  xcDOT?: string;
};

export type EthereumChainMap = Record<ChainKey<EthereumKind>, EthereumChain>;
export type ParachainMap = Record<ChainKey<ParachainKind>, Parachain>;

export type KusamaConfig = {
  assetHubParaId: number;
  bridgeHubParaId: number;
  parachains: ParachainMap;
};

export type Environment = {
  name: string;
  // Ethereum
  ethChainId: number;
  gatewayContract: string;
  beefyContract: string;
  beaconApiUrl: string;
  ethereumChains: Record<string, string>;
  // Substrate
  assetHubParaId: number;
  bridgeHubParaId: number;
  /** @deprecated Remove once V2 is fully rolled out to all parachains */
  v2_parachains?: readonly number[];
  relaychainUrl: string;
  parachains: {
    [paraId: string]: string;
  };
  // Indexer
  indexerGraphQlUrl: string;
  kusama?: {
    assetHubParaId: number;
    bridgeHubParaId: number;
    parachains: Record<string, string>;
  };
  // Assets
  assetOverrides?: AssetOverrideMap;
  precompiles?: PrecompileMap;
  metadataOverrides?: ERC20MetadataOverrideMap;
  // L2 Forwarding
  l2Bridge?: {
    acrossAPIUrl: string;
    l1AdapterAddress: string;
    l1HandlerAddress: string;
    l1FeeTokenAddress: string;
    l1SwapRouterAddress: string;
    l1SwapQuoterAddress: string;
    l2Chains: Record<number, L2ForwardMetadata>;
  };
};

export type ChainId = {
  kind: ChainKind;
  /** Ethereum chain id or polkadot parachain id.
   */
  id: number;
};

export type TransferRoute = {
  from: ChainId;
  to: ChainId;
  assets: readonly string[];
};

export type ChainKey<T extends string> = `${T}_${number}`;

export type Source = ChainId & {
  key: ChainKey<ChainKind>;
  destinations: Record<
    ChainKey<ChainKind>,
    ChainId & {
      key: ChainKey<ChainKind>;
      assets: string[];
    }
  >;
};

export type TransferLocation = ParachainLocation | EthereumLocation;

export type ParachainLocation = ChainId & {
  kind: ParachainKind;
  key: ChainKey<ParachainKind>;
  parachain: Parachain;
};

export type EthereumLocation = ChainId & {
  kind: EthereumKind;
  key: ChainKey<EthereumKind>;
  ethChain: EthereumChain;
  parachain?: Parachain;
};

export type AssetOverrideMap = Record<string, Asset[]>;

export type ERC20MetadataOverrideMap = Record<
  string,
  {
    name?: string;
    symbol?: string;
    decimals?: number;
  }
>;

export type PrecompileMap = Record<string, `0x${string}`>;

export type AssetRegistry = {
  timestamp: string;
  environment: string;
  gatewayAddress: string;
  ethChainId: number;
  assetHubParaId: number;
  bridgeHubParaId: number;
  relaychain: ChainProperties;
  bridgeHub: ChainProperties;
  ethereumChains: EthereumChainMap;
  parachains: ParachainMap;
  kusama?: KusamaConfig;
};

export type ContractCall = {
  target: string;
  calldata: string;
  value: bigint;
  gas: bigint;
};

export type SubstrateAccount = {
  nonce: bigint;
  consumers: bigint;
  providers: bigint;
  sufficients: bigint;
  data: {
    free: bigint;
    reserved: bigint;
    frozen: bigint;
  };
};

export type PNAMap = Record<
  string,
  {
    token: string;
    foreignId: string;
    ethereumlocation: any;
  }
>;

export type AssetSwapRoute = {
  inputToken: string;
  outputToken: string;
  swapFee: number; // fee tier for uniswap call in basis points (e.g., 500 = 0.05%)
};

export type L2ForwardMetadata = {
  adapterAddress: string;
  feeTokenAddress: string;
  swapRoutes: readonly AssetSwapRoute[];
};

export type FeeEstimateErrorDetails = {
  type: string;
  code: string;
  status: number;
  message: string;
  id: string;
};
export class FeeEstimateError extends Error {
  readonly details: FeeEstimateErrorDetails;
  constructor(details: FeeEstimateErrorDetails) {
    super(details.message);
    this.details = details;
  }
}

export type AccountType = "AccountId20" | "AccountId32";

export type XcmVersion = "v4" | "v5";

export interface XC20TokenMap {
  [xc20: string]: string;
}

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

export interface ERC20MetadataMap {
  [token: string]: ERC20Metadata;
}

export type EthereumChain = {
  chainId: number;
  id: string;
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

export interface AssetMap {
  [token: string]: Asset;
}

export type Parachain = {
  parachainId: number;
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

export interface ParachainMap {
  [paraId: string]: Parachain;
}

export function supportsEthereumToPolkadotV2(parachain: Parachain): boolean {
  return (
    parachain.features.hasXcmPaymentApi &&
    parachain.features.xcmVersion === "v5" &&
    parachain.features.supportsV2
  );
}

export function supportsPolkadotToEthereumV2(parachain: Parachain): boolean {
  return (
    parachain.features.hasEthBalance &&
    parachain.features.hasXcmPaymentApi &&
    parachain.features.supportsAliasOrigin &&
    parachain.features.xcmVersion === "v5" &&
    parachain.features.supportsV2
  );
}

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
  ethereumChains: {
    [chainId: string]: string;
  };
  // Substrate
  assetHubParaId: number;
  bridgeHubParaId: number;
  /** @deprecated Remove once V2 is fully rolled out to all parachains */
  v2_parachains?: number[];
  relaychainUrl: string;
  parachains: {
    [paraId: string]: string;
  };
  // Indexer
  indexerGraphQlUrl: string;
  kusama?: {
    assetHubParaId: number;
    bridgeHubParaId: number;
    parachains: { [paraId: string]: string };
  };
  // Assets
  assetOverrides?: AssetOverrideMap;
  precompiles?: PrecompileMap;
  metadataOverrides?: ERC20MetadataOverrideMap;
  // L2 Forwarding
  l2Bridge?: {
    acrossAPIUrl: string;
    l1AdapterAddress: string;
    l1FeeTokenAddress: string;
    l1SwapQuoterAddress: string;
    l2Chains: { [l2ChainId: number]: L2ForwardMetadata };
  };
};

export type SourceType = "substrate" | "ethereum";

export type Path = {
  type: SourceType;
  id: string;
  source: number;
  destinationType: SourceType;
  destination: number;
  asset: string;
};

export type Source = {
  type: SourceType;
  id: string;
  key: string;
  destinations: {
    [destination: string]: { type: SourceType; assets: string[] };
  };
};

export type TransferLocation = {
  id: string;
  name: string;
  key: string;
  type: SourceType;
  parachain?: Parachain;
  ethChain?: EthereumChain;
};
export interface AssetOverrideMap {
  [paraId: string]: Asset[];
}

export interface ERC20MetadataOverrideMap {
  [token: string]: {
    name?: string;
    symbol?: string;
    decimals?: number;
  };
}

export interface PrecompileMap {
  [chainId: string]: `0x${string}`;
}

export type AssetRegistry = {
  timestamp: string;
  environment: string;
  gatewayAddress: string;
  ethChainId: number;
  assetHubParaId: number;
  bridgeHubParaId: number;
  relaychain: ChainProperties;
  bridgeHub: ChainProperties;
  ethereumChains: {
    [chainId: string]: EthereumChain;
  };
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

export interface PNAMap {
  [token: string]: {
    token: string;
    foreignId: string;
    ethereumlocation: any;
  };
}

export type AssetSwapRoute = {
  inputToken: string;
  outputToken: string;
  swapFee: number; // fee tier for uniswap call in basis points (e.g., 500 = 0.05%)
};

export type L2ForwardMetadata = {
  adapterAddress: string;
  feeTokenAddress: string;
  swapRoutes: AssetSwapRoute[];
};

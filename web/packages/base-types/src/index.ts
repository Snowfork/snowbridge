export type AccountType = "AccountId20" | "AccountId32";

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
  };
  assets: AssetMap;
  estimatedExecutionFeeDOT: bigint;
  estimatedDeliveryFeeDOT: bigint;
  xcDOT?: string;
};

export interface ParachainMap {
  [paraId: string]: Parachain;
}

export type KusamaConfig = {
  assetHubParaId: number;
  bridgeHubParaId: number;
  parachains: ParachainMap;
};

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
  kusama: KusamaConfig | undefined;
};

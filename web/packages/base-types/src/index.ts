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
    parachain.features.xcmVersion === "v5"
  );
}

export function supportsPolkadotToEthereumV2(parachain: Parachain): boolean {
  return (
    parachain.features.hasEthBalance &&
    parachain.features.hasXcmPaymentApi &&
    parachain.features.supportsAliasOrigin &&
    parachain.features.xcmVersion === "v5"
  );
}

export type KusamaConfig = {
  assetHubParaId: number;
  bridgeHubParaId: number;
  parachains: ParachainMap;
};

export type Environment = {
    name: string;
    ethChainId: number;
    beaconApiUrl: string;
    ethereumChains: {
        [chainId: string]: string;
    };
    relaychainUrl: string;
    parachains: {
        [paraId: string]: string;
    };
    gatewayContract: string;
    beefyContract: string;
    assetHubParaId: number;
    bridgeHubParaId: number;
    indexerGraphQlUrl: string;
    kusama?: {
        assetHubParaId: number
        bridgeHubParaId: number
        parachains: { [paraId: string]: string }
    }
    assetOverrides?: AssetOverrideMap,
    precompiles?: PrecompileMap,
    metadataOverrides? : ERC20MetadataOverrideMap,
};

export type SourceType = "substrate" | "ethereum"

export type Path = {
    type: SourceType
    id: string
    source: number
    destinationType: SourceType
    destination: number
    asset: string
}

export type Source = {
    type: SourceType
    id: string
    key: string
    destinations: { [destination: string]: { type: SourceType; assets: string[] } }
}

export type TransferLocation = {
    id: string
    name: string
    key: string
    type: SourceType
    parachain?: Parachain
    ethChain?: EthereumChain
}
export interface AssetOverrideMap {
    [paraId: string]: Asset[]
}

export interface ERC20MetadataOverrideMap {
  [token: string]: {
    name?: string
    symbol?: string
    decimals?: number
  }
}

export interface PrecompileMap {
    [chainId: string]: `0x${string}`
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

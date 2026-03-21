export type MultiAddressStruct = {
  kind: number;
  data: string;
};

export type QuoteExactOutputSingleParamsStruct = {
  tokenIn: string;
  tokenOut: string;
  amount: bigint;
  fee: number;
  sqrtPriceLimitX96: bigint | number;
};

export type DepositParamsStruct = {
  inputToken: string;
  outputToken: string;
  inputAmount: bigint;
  outputAmount: bigint;
  destinationChainId: bigint;
  fillDeadlineBuffer: bigint;
};

export type SendParamsStruct = {
  xcm: Uint8Array;
  assets: any[];
  claimer: Uint8Array;
  executionFee: bigint;
  relayerFee: bigint;
};

export type SwapParamsStruct = {
  inputAmount: bigint;
  router: string;
  callData: string;
};

export type IGatewayV1 = {
  quoteSendTokenFee(
    token: string,
    destinationChain: number,
    destinationFee: bigint,
  ): Promise<bigint>;
  operatingMode(): Promise<bigint>;
  channelNoncesOf(channelId: string): Promise<[bigint, bigint]>;
  channelOperatingModeOf(channelId: string): Promise<bigint>;
  agentOf(agentID: string): Promise<string>;
};

export type IGatewayV2 = {
  operatingMode(): Promise<bigint>;
  v2_outboundNonce(): Promise<bigint>;
  isTokenRegistered(token: string): Promise<boolean>;
  agentOf(agentID: string): Promise<string>;
};

export type BeefyClient = {
  latestBeefyBlock(): Promise<bigint>;
};

export type ISwapQuoter = {
  quoteExactOutputSingle: {
    staticCall(
      params: QuoteExactOutputSingleParamsStruct,
    ): Promise<readonly [bigint, ...any[]]>;
  };
};

export type IERC20 = {
  balanceOf(account: string): Promise<bigint>;
  allowance(owner: string, spender: string): Promise<bigint>;
};

export const IGATEWAY_V1_ABI = [
  {
    type: "function",
    name: "agentOf",
    inputs: [{ name: "agentID", type: "bytes32", internalType: "bytes32" }],
    outputs: [{ name: "", type: "address", internalType: "address" }],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "channelNoncesOf",
    inputs: [{ name: "channelID", type: "bytes32", internalType: "ChannelID" }],
    outputs: [
      { name: "", type: "uint64", internalType: "uint64" },
      { name: "", type: "uint64", internalType: "uint64" },
    ],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "channelOperatingModeOf",
    inputs: [{ name: "channelID", type: "bytes32", internalType: "ChannelID" }],
    outputs: [{ name: "", type: "uint8", internalType: "enum OperatingMode" }],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "operatingMode",
    inputs: [],
    outputs: [{ name: "", type: "uint8", internalType: "enum OperatingMode" }],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "quoteSendTokenFee",
    inputs: [
      { name: "token", type: "address", internalType: "address" },
      {
        name: "destinationChain",
        type: "uint32",
        internalType: "ParaID",
      },
      { name: "destinationFee", type: "uint128", internalType: "uint128" },
    ],
    outputs: [{ name: "", type: "uint256", internalType: "uint256" }],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "sendToken",
    inputs: [
      { name: "token", type: "address", internalType: "address" },
      {
        name: "destinationChain",
        type: "uint32",
        internalType: "ParaID",
      },
      {
        name: "destinationAddress",
        type: "tuple",
        internalType: "struct MultiAddress",
        components: [
          { name: "kind", type: "uint8", internalType: "enum Kind" },
          { name: "data", type: "bytes", internalType: "bytes" },
        ],
      },
      { name: "destinationFee", type: "uint128", internalType: "uint128" },
      { name: "amount", type: "uint128", internalType: "uint128" },
    ],
    outputs: [],
    stateMutability: "payable",
  },
  {
    type: "event",
    name: "OutboundMessageAccepted",
    inputs: [
      {
        name: "channelID",
        type: "bytes32",
        indexed: true,
        internalType: "ChannelID",
      },
      {
        name: "nonce",
        type: "uint64",
        indexed: false,
        internalType: "uint64",
      },
      {
        name: "messageID",
        type: "bytes32",
        indexed: true,
        internalType: "bytes32",
      },
      {
        name: "payload",
        type: "bytes",
        indexed: false,
        internalType: "bytes",
      },
    ],
    anonymous: false,
  },
] as const;

export const IGATEWAY_V2_ABI = [
  {
    type: "function",
    name: "agentOf",
    inputs: [{ name: "agentID", type: "bytes32", internalType: "bytes32" }],
    outputs: [{ name: "", type: "address", internalType: "address" }],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "isTokenRegistered",
    inputs: [{ name: "token", type: "address", internalType: "address" }],
    outputs: [{ name: "", type: "bool", internalType: "bool" }],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "operatingMode",
    inputs: [],
    outputs: [{ name: "", type: "uint8", internalType: "enum OperatingMode" }],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "v2_createAgent",
    inputs: [{ name: "id", type: "bytes32", internalType: "bytes32" }],
    outputs: [],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "v2_outboundNonce",
    inputs: [],
    outputs: [{ name: "", type: "uint64", internalType: "uint64" }],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "v2_registerToken",
    inputs: [
      { name: "token", type: "address", internalType: "address" },
      { name: "network", type: "uint8", internalType: "uint8" },
      { name: "executionFee", type: "uint128", internalType: "uint128" },
      { name: "relayerFee", type: "uint128", internalType: "uint128" },
    ],
    outputs: [],
    stateMutability: "payable",
  },
  {
    type: "function",
    name: "v2_sendMessage",
    inputs: [
      { name: "xcm", type: "bytes", internalType: "bytes" },
      { name: "assets", type: "bytes[]", internalType: "bytes[]" },
      { name: "claimer", type: "bytes", internalType: "bytes" },
      { name: "executionFee", type: "uint128", internalType: "uint128" },
      { name: "relayerFee", type: "uint128", internalType: "uint128" },
    ],
    outputs: [],
    stateMutability: "payable",
  },
  {
    type: "event",
    name: "OutboundMessageAccepted",
    inputs: [
      {
        name: "nonce",
        type: "uint64",
        indexed: false,
        internalType: "uint64",
      },
      {
        name: "payload",
        type: "tuple",
        indexed: false,
        internalType: "struct Payload",
        components: [
          { name: "origin", type: "address", internalType: "address" },
          {
            name: "assets",
            type: "tuple[]",
            internalType: "struct Asset[]",
            components: [
              { name: "kind", type: "uint8", internalType: "uint8" },
              { name: "data", type: "bytes", internalType: "bytes" },
            ],
          },
          {
            name: "xcm",
            type: "tuple",
            internalType: "struct Xcm",
            components: [
              { name: "kind", type: "uint8", internalType: "uint8" },
              { name: "data", type: "bytes", internalType: "bytes" },
            ],
          },
          { name: "claimer", type: "bytes", internalType: "bytes" },
          { name: "value", type: "uint128", internalType: "uint128" },
          {
            name: "executionFee",
            type: "uint128",
            internalType: "uint128",
          },
          { name: "relayerFee", type: "uint128", internalType: "uint128" },
        ],
      },
    ],
    anonymous: false,
  },
] as const;

export const IERC20_ABI = [
  {
    type: "function",
    name: "allowance",
    inputs: [
      { name: "owner", type: "address", internalType: "address" },
      { name: "spender", type: "address", internalType: "address" },
    ],
    outputs: [{ name: "", type: "uint256", internalType: "uint256" }],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "approve",
    inputs: [
      { name: "spender", type: "address", internalType: "address" },
      { name: "amount", type: "uint256", internalType: "uint256" },
    ],
    outputs: [{ name: "", type: "bool", internalType: "bool" }],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "balanceOf",
    inputs: [{ name: "account", type: "address", internalType: "address" }],
    outputs: [{ name: "", type: "uint256", internalType: "uint256" }],
    stateMutability: "view",
  },
] as const;

export const BEEFY_CLIENT_ABI = [
  {
    type: "function",
    name: "latestBeefyBlock",
    inputs: [],
    outputs: [{ name: "", type: "uint64", internalType: "uint64" }],
    stateMutability: "view",
  },
] as const;

export const SWAP_QUOTER_ABI = [
  {
    type: "function",
    name: "quoteExactOutputSingle",
    inputs: [
      {
        name: "params",
        type: "tuple",
        internalType: "struct ISwapQuoter.QuoteExactOutputSingleParams",
        components: [
          { name: "tokenIn", type: "address", internalType: "address" },
          { name: "tokenOut", type: "address", internalType: "address" },
          { name: "amount", type: "uint256", internalType: "uint256" },
          { name: "fee", type: "uint24", internalType: "uint24" },
          {
            name: "sqrtPriceLimitX96",
            type: "uint160",
            internalType: "uint160",
          },
        ],
      },
    ],
    outputs: [
      { name: "amountIn", type: "uint256", internalType: "uint256" },
      {
        name: "sqrtPriceX96After",
        type: "uint160",
        internalType: "uint160",
      },
      {
        name: "initializedTicksCrossed",
        type: "uint32",
        internalType: "uint32",
      },
      { name: "gasEstimate", type: "uint256", internalType: "uint256" },
    ],
    stateMutability: "nonpayable",
  },
] as const;

export const SWAP_ROUTER_ABI = [
  {
    type: "function",
    name: "exactOutputSingle",
    inputs: [
      {
        name: "params",
        type: "tuple",
        internalType: "struct ISwapRouter.ExactOutputSingleParams",
        components: [
          { name: "tokenIn", type: "address", internalType: "address" },
          { name: "tokenOut", type: "address", internalType: "address" },
          { name: "fee", type: "uint24", internalType: "uint24" },
          { name: "recipient", type: "address", internalType: "address" },
          { name: "deadline", type: "uint256", internalType: "uint256" },
          { name: "amountOut", type: "uint256", internalType: "uint256" },
          {
            name: "amountInMaximum",
            type: "uint256",
            internalType: "uint256",
          },
          {
            name: "sqrtPriceLimitX96",
            type: "uint160",
            internalType: "uint160",
          },
        ],
      },
    ],
    outputs: [{ name: "amountIn", type: "uint256", internalType: "uint256" }],
    stateMutability: "nonpayable",
  },
] as const;

export const SWAP_LEGACY_ROUTER_ABI = [
  {
    type: "function",
    name: "exactOutputSingle",
    inputs: [
      {
        name: "params",
        type: "tuple",
        internalType: "struct ISwapLegacyRouter.ExactOutputSingleParams",
        components: [
          { name: "tokenIn", type: "address", internalType: "address" },
          { name: "tokenOut", type: "address", internalType: "address" },
          { name: "fee", type: "uint24", internalType: "uint24" },
          { name: "recipient", type: "address", internalType: "address" },
          { name: "amountOut", type: "uint256", internalType: "uint256" },
          {
            name: "amountInMaximum",
            type: "uint256",
            internalType: "uint256",
          },
          {
            name: "sqrtPriceLimitX96",
            type: "uint160",
            internalType: "uint160",
          },
        ],
      },
    ],
    outputs: [{ name: "amountIn", type: "uint256", internalType: "uint256" }],
    stateMutability: "nonpayable",
  },
] as const;

export const SNOWBRIDGE_L1_ADAPTOR_ABI = [
  {
    type: "function",
    name: "depositNativeEther",
    inputs: [
      {
        name: "params",
        type: "tuple",
        internalType: "struct DepositParams",
        components: [
          { name: "inputToken", type: "address", internalType: "address" },
          { name: "outputToken", type: "address", internalType: "address" },
          { name: "inputAmount", type: "uint256", internalType: "uint256" },
          { name: "outputAmount", type: "uint256", internalType: "uint256" },
          {
            name: "destinationChainId",
            type: "uint256",
            internalType: "uint256",
          },
          {
            name: "fillDeadlineBuffer",
            type: "uint32",
            internalType: "uint32",
          },
        ],
      },
      { name: "recipient", type: "address", internalType: "address" },
      { name: "topic", type: "bytes32", internalType: "bytes32" },
    ],
    outputs: [],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "depositToken",
    inputs: [
      {
        name: "params",
        type: "tuple",
        internalType: "struct DepositParams",
        components: [
          { name: "inputToken", type: "address", internalType: "address" },
          { name: "outputToken", type: "address", internalType: "address" },
          { name: "inputAmount", type: "uint256", internalType: "uint256" },
          { name: "outputAmount", type: "uint256", internalType: "uint256" },
          {
            name: "destinationChainId",
            type: "uint256",
            internalType: "uint256",
          },
          {
            name: "fillDeadlineBuffer",
            type: "uint32",
            internalType: "uint32",
          },
        ],
      },
      { name: "recipient", type: "address", internalType: "address" },
      { name: "topic", type: "bytes32", internalType: "bytes32" },
    ],
    outputs: [],
    stateMutability: "nonpayable",
  },
] as const;

export const SNOWBRIDGE_L2_ADAPTOR_ABI = [
  {
    type: "function",
    name: "sendEtherAndCall",
    inputs: [
      {
        name: "params",
        type: "tuple",
        internalType: "struct DepositParams",
        components: [
          { name: "inputToken", type: "address", internalType: "address" },
          { name: "outputToken", type: "address", internalType: "address" },
          { name: "inputAmount", type: "uint256", internalType: "uint256" },
          { name: "outputAmount", type: "uint256", internalType: "uint256" },
          {
            name: "destinationChainId",
            type: "uint256",
            internalType: "uint256",
          },
          {
            name: "fillDeadlineBuffer",
            type: "uint32",
            internalType: "uint32",
          },
        ],
      },
      {
        name: "sendParams",
        type: "tuple",
        internalType: "struct SendParams",
        components: [
          { name: "xcm", type: "bytes", internalType: "bytes" },
          { name: "assets", type: "bytes[]", internalType: "bytes[]" },
          { name: "claimer", type: "bytes", internalType: "bytes" },
          { name: "executionFee", type: "uint128", internalType: "uint128" },
          { name: "relayerFee", type: "uint128", internalType: "uint128" },
        ],
      },
      { name: "recipient", type: "address", internalType: "address" },
      { name: "topic", type: "bytes32", internalType: "bytes32" },
    ],
    outputs: [],
    stateMutability: "payable",
  },
  {
    type: "function",
    name: "sendTokenAndCall",
    inputs: [
      {
        name: "params",
        type: "tuple",
        internalType: "struct DepositParams",
        components: [
          { name: "inputToken", type: "address", internalType: "address" },
          { name: "outputToken", type: "address", internalType: "address" },
          { name: "inputAmount", type: "uint256", internalType: "uint256" },
          { name: "outputAmount", type: "uint256", internalType: "uint256" },
          {
            name: "destinationChainId",
            type: "uint256",
            internalType: "uint256",
          },
          {
            name: "fillDeadlineBuffer",
            type: "uint32",
            internalType: "uint32",
          },
        ],
      },
      {
        name: "swapParams",
        type: "tuple",
        internalType: "struct SwapParams",
        components: [
          { name: "inputAmount", type: "uint256", internalType: "uint256" },
          { name: "router", type: "address", internalType: "address" },
          { name: "callData", type: "bytes", internalType: "bytes" },
        ],
      },
      {
        name: "sendParams",
        type: "tuple",
        internalType: "struct SendParams",
        components: [
          { name: "xcm", type: "bytes", internalType: "bytes" },
          { name: "assets", type: "bytes[]", internalType: "bytes[]" },
          { name: "claimer", type: "bytes", internalType: "bytes" },
          { name: "executionFee", type: "uint128", internalType: "uint128" },
          { name: "relayerFee", type: "uint128", internalType: "uint128" },
        ],
      },
      { name: "recipient", type: "address", internalType: "address" },
      { name: "topic", type: "bytes32", internalType: "bytes32" },
    ],
    outputs: [],
    stateMutability: "nonpayable",
  },
] as const;

export const MOONBEAM_PALLET_XCM_PRECOMPILE_ABI = [
  {
    inputs: [
      {
        components: [
          { internalType: "uint8", name: "parents", type: "uint8" },
          { internalType: "bytes[]", name: "interior", type: "bytes[]" },
        ],
        internalType: "struct XCM.Location",
        name: "dest",
        type: "tuple",
      },
      {
        components: [
          { internalType: "address", name: "asset", type: "address" },
          { internalType: "uint256", name: "amount", type: "uint256" },
        ],
        internalType: "struct XCM.AssetAddressInfo[]",
        name: "assets",
        type: "tuple[]",
      },
      {
        internalType: "enum XCM.TransferType",
        name: "assetsTransferType",
        type: "uint8",
      },
      { internalType: "uint8", name: "remoteFeesIdIndex", type: "uint8" },
      {
        internalType: "enum XCM.TransferType",
        name: "feesTransferType",
        type: "uint8",
      },
      { internalType: "bytes", name: "customXcmOnDest", type: "bytes" },
    ],
    name: "transferAssetsUsingTypeAndThenAddress",
    outputs: [],
    stateMutability: "nonpayable",
    type: "function",
  },
] as const;

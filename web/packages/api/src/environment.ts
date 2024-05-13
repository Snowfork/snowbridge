
export type Config = {
  BEACON_HTTP_API: string
  ETHEREUM_WS_API: (secret: string) => string
  RELAY_CHAIN_WS_URL: string
  ASSET_HUB_WS_URL: string
  BRIDGE_HUB_WS_URL: string
  GATEWAY_CONTRACT: string
  BEEFY_CONTRACT: string
  ASSET_HUB_PARAID: number
  BRIDGE_HUB_PARAID: number
  PRIMARY_GOVERNANCE_CHANNEL_ID: string
  SECONDARY_GOVERNANCE_CHANNEL_ID: string
  RELAYERS: Relayer[]
  PARACHAINS: string[]
}

export type SourceType = 'substrate' | 'ethereum'
export type Relayer = { name: string, account: string, type: SourceType }
export type ParachainInfo = { paraId: number, destinationFeeDOT: bigint, has20ByteAccounts: boolean, decimals: number }
export type TransferLocation = {
  id: string
  name: string
  type: SourceType
  destinationIds: string[]
  paraInfo?: ParachainInfo
  erc20tokensReceivable: { [name: string]: string }
}

export type SnowbridgeEnvironment = {
  config: Config
  name: string
  locations: TransferLocation[]
}

export const SNOWBRIDGE_ENV: { [id: string]: SnowbridgeEnvironment } = {
  'local_e2e': {
    name: 'local_e2e',
    locations: [{
      id: 'ethereum',
      name: 'Ethereum',
      type: 'ethereum',
      destinationIds: ['assethub', 'penpal'],
      erc20tokensReceivable: {
        "WETH": '0x87d1f7fdfEe7f651FaBc8bFCB6E086C278b77A7d',
      },
    },
    {
      id: 'assethub',
      name: 'Asset Hub',
      type: 'substrate',
      destinationIds: ['ethereum'],
      paraInfo: {
        paraId: 1000,
        destinationFeeDOT: 0n,
        has20ByteAccounts: false,
        decimals: 12,
      },
      erc20tokensReceivable: {
        "WETH": '0x87d1f7fdfEe7f651FaBc8bFCB6E086C278b77A7d',
      },
    },
    {
      id: 'penpal',
      name: 'Penpal',
      type: 'substrate',
      destinationIds: ["ethereum"],
      paraInfo: {
        paraId: 2000,
        destinationFeeDOT: 4_000_000_000n,
        has20ByteAccounts: false,
        decimals: 12,
      },
      erc20tokensReceivable: {
        "WETH": '0x87d1f7fdfEe7f651FaBc8bFCB6E086C278b77A7d',
      },
    }],
    config: {
      BEACON_HTTP_API: 'http://127.0.0.1:9596',
      ETHEREUM_WS_API: () => 'ws://127.0.0.1:8546',
      RELAY_CHAIN_WS_URL: 'ws://127.0.0.1:9944',
      ASSET_HUB_WS_URL: 'ws://127.0.0.1:12144',
      BRIDGE_HUB_WS_URL: 'ws://127.0.0.1:11144',
      PARACHAINS: ['ws://127.0.0.1:13144'],
      GATEWAY_CONTRACT: '0xEDa338E4dC46038493b885327842fD3E301CaB39',
      BEEFY_CONTRACT: '0x992B9df075935E522EC7950F37eC8557e86f6fdb',
      ASSET_HUB_PARAID: 1000,
      BRIDGE_HUB_PARAID: 1013,
      PRIMARY_GOVERNANCE_CHANNEL_ID: '0x0000000000000000000000000000000000000000000000000000000000000001',
      SECONDARY_GOVERNANCE_CHANNEL_ID: '0x0000000000000000000000000000000000000000000000000000000000000002',
      RELAYERS: [
        { name: 'beacon', account: '5GWFwdZb6JyU46e6ZiLxjGxogAHe8SenX76btfq8vGNAaq8c', type: 'substrate' },
        { name: 'beefy', account: '0x87D987206180B8f3807Dd90455606eEa85cdB87a', type: 'ethereum' },
        { name: 'parachain-primary-gov', account: '0xeEBFA6B9242A19f91a0463291A937a20e3355681', type: 'ethereum' },
        { name: 'parachain-secondary-gov', account: '0x13e16C4e5787f878f98a610EB321170512b134D4', type: 'ethereum' },
        { name: 'execution-assethub', account: '5DF6KbMTBPGQN6ScjqXzdB2ngk5wi3wXvubpQVUZezNfM6aV', type: 'substrate' },
        { name: 'parachain-assethub', account: '0x8b66D5499F52D6F1857084A61743dFCB9a712859', type: 'ethereum' },
        { name: 'execution-penpal', account: '5HgmfVcc8xBUcReNJsUaJMhFWGkdYpEw4RiCX4SeZPdKXR6H', type: 'substrate' },
        { name: 'parachain-penpal', account: '0x01F6749035e02205768f97e6f1d394Fb6769EC20', type: 'ethereum' },
      ],
    },
  },
  'rococo_sepolia': {
    name: 'rococo_sepolia',
    locations: [{
      id: 'ethereum',
      name: 'Ethereum',
      type: 'ethereum',
      destinationIds: ['assethub', 'muse'],
      erc20tokensReceivable: {
        "WETH": '0xfff9976782d46cc05630d1f6ebab18b2324d6b14',
        "vETH": '0xc3d088842dcf02c13699f936bb83dfbbc6f721ab',
        "MUSE1": '0xb34a6924a02100ba6ef12af1c798285e8f7a16ee',
        "MUSE2": '0xc9f05326311bc2a55426761bec20057685fb80f7',
      },
    },
    {
      id: 'assethub',
      name: 'Asset Hub',
      type: 'substrate',
      destinationIds: ['ethereum'],
      paraInfo: {
        paraId: 1000,
        destinationFeeDOT: 0n,
        has20ByteAccounts: false,
        decimals: 12,
      },
      erc20tokensReceivable: {
        "WETH": '0xfff9976782d46cc05630d1f6ebab18b2324d6b14',
        "vETH": '0xc3d088842dcf02c13699f936bb83dfbbc6f721ab',
        "MUSE1": '0xb34a6924a02100ba6ef12af1c798285e8f7a16ee',
        "MUSE2": '0xc9f05326311bc2a55426761bec20057685fb80f7',
      },
    },
    {
      id: 'muse',
      name: 'Muse',
      type: 'substrate',
      destinationIds: [],
      paraInfo: {
        paraId: 3369,
        destinationFeeDOT: 4_000_000_000_000n,
        has20ByteAccounts: true,
        decimals: 12,
      },
      erc20tokensReceivable: {
        "MUSE1": '0xb34a6924a02100ba6ef12af1c798285e8f7a16ee',
        "MUSE2": '0xc9f05326311bc2a55426761bec20057685fb80f7',
      },
    }],
    config: {
      BEACON_HTTP_API: 'https://lodestar-sepolia.chainsafe.io',
      ETHEREUM_WS_API: (key) => `https://eth-sepolia.g.alchemy.com/v2/${key}`,
      RELAY_CHAIN_WS_URL: 'wss://rococo-rpc.polkadot.io',
      ASSET_HUB_WS_URL: 'wss://rococo-asset-hub-rpc.polkadot.io',
      BRIDGE_HUB_WS_URL: 'wss://rococo-bridge-hub-rpc.polkadot.io',
      PARACHAINS: ['wss://rococo-muse-rpc.polkadot.io'],
      GATEWAY_CONTRACT: '0x5b4909ce6ca82d2ce23bd46738953c7959e710cd',
      BEEFY_CONTRACT: '0x27e5e17ac995d3d720c311e1e9560e28f5855fb1',
      ASSET_HUB_PARAID: 1000,
      BRIDGE_HUB_PARAID: 1013,
      PRIMARY_GOVERNANCE_CHANNEL_ID: '0x0000000000000000000000000000000000000000000000000000000000000001',
      SECONDARY_GOVERNANCE_CHANNEL_ID: '0x0000000000000000000000000000000000000000000000000000000000000002',
      RELAYERS: [
        { name: 'beacon', account: '5FyC9GkHhiAYjMtddwVNc2gx8wBjH9gpMKWbQ1QVXmmJtr8M', type: 'substrate' },
        { name: 'beefy', account: '0xF061685F2B729b89a7A5966B3ab9aee15269e8FE', type: 'ethereum' },
        { name: 'parachain-primary-gov', account: '0xE3f4e40E0dB2F828e248dB2790958035BaB1a4B5', type: 'ethereum' },
        { name: 'parachain-secondary-gov', account: '0x71429d3B9d68557C2F49e42e12B9cf7edAADCe3D', type: 'ethereum' },
        { name: 'execution-assethub', account: '5GeLu5rPcaoZ2RVDbhX8DKJt8NxnKn6DvvjfuhnwTZyYRY59', type: 'substrate' },
        { name: 'parachain-assethub', account: '0x0b65d43d159f1C40Bad7768fd59667E3104a2ECE', type: 'ethereum' },
      ],
    },
  }
};

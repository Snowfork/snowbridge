module.exports = {
  apps: [
    {
      name: "westend-monitor",
      interpreter: "node",
      node_args: "--require=ts-node/register --require=dotenv/config",
      script: "./src/main.ts",
      args: "cron",
    },
    {
      name: "EtherToPolkadot",
      interpreter: "node",
      node_args: "--require=ts-node/register --require=dotenv/config",
      script: "./src/transfer_native_ether_to_polkadot.ts",
      args: "cron",
    },
    {
      name: "EtherToEthereum",
      interpreter: "node",
      node_args: "--require=ts-node/register --require=dotenv/config",
      script: "./src/transfer_native_ether_to_ethereum.ts",
      args: "cron",
    },
    {
      name: "WndToEthereum",
      interpreter: "node",
      node_args: "--require=ts-node/register --require=dotenv/config",
      script: "./src/transfer_relay_token_to_ethereum.ts",
      args: "cron",
    },
    {
      name: "WndToPolkadot",
      interpreter: "node",
      node_args: "--require=ts-node/register --require=dotenv/config",
      script: "./src/transfer_relay_token_to_polkadot.ts",
      args: "cron",
    },
  ],
};

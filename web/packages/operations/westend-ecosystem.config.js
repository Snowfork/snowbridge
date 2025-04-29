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
      name: "EtherToAssetHub",
      interpreter: "node",
      node_args: "--require=ts-node/register --require=dotenv/config",
      script: "./src/transfer_ether_to_ah.ts",
      args: "cron",
    },
    {
      name: "EtherFromAssetHub",
      interpreter: "node",
      node_args: "--require=ts-node/register --require=dotenv/config",
      script: "./src/transfer_ether_from_ah.ts",
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
      name: "WndToAssetHub",
      interpreter: "node",
      node_args: "--require=ts-node/register --require=dotenv/config",
      script: "./src/transfer_relay_token_to_ah.ts",
      args: "cron",
    },
  ],
};

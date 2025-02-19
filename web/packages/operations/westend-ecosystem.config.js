module.exports = {
  apps: [
    {
      name: "westend-monitor",
      node_args: "--require=dotenv/config",
      script: "./dist/src/main.js",
      args: "cron",
    },
    {
      name: "westend-transferToPolkadot",
      node_args: "--require=dotenv/config",
      script: "./dist/src/transfer_to_polkadot.js",
      args: "cron",
    },
    {
      name: "westend-transferToEthereum",
      node_args: "--require=dotenv/config",
      script: "./dist/src/transfer_to_ethereum.js",
      args: "cron",
    },
  ],
};

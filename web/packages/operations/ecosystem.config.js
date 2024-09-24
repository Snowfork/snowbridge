module.exports = {
  apps: [
    {
      name: "monitor",
      node_args: "--require=dotenv/config",
      script: "./dist/src/main.js",
      args: "cron"
    },
    {
      name: "transferTest",
      node_args: "--require=dotenv/config",
      script: "./dist/src/transfer_token.js",
      args: "cron"
    },
  ],
};

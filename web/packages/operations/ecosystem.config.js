module.exports = {
  apps: [
    {
      name: "monitor",
      node_args: "--require=dotenv/config",
      script: "./dist/src/main.js",
      args: "cron",
    },
  ],
};

// Truffle is only used for its ability to debug transactions
module.exports = {
  networks: {
    localhost: {
      host: "127.0.0.1",
      port: 8545,
      network_id: "15"
    },
  },
  compilers: {
    solc: {
      version: "0.8.9",
      optimizer: {
        enabled: false,
        runs: 200
      }
    }
  }
};

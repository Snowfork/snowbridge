module.exports = {
  networks: {
    develop: {
      host: "localhost",
      port: 7545, // Match default network 'ganache'
      network_id: 5777,
      gas: 6721975, // Truffle default development block gas limit
      gasPrice: 200000000000,
      solc: {
        version: "0.6.2",
        optimizer: {
          enabled: true,
          runs: 200
        }
      }
    },
  },
  mocha: {
    useColors: true
  },
  // Configure your compilers
  compilers: {
    solc: {
      version: "0.6.2",
    }
  }
};

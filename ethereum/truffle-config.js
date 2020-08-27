module.exports = {
  networks: {
    vincent_develop: {
      host: "172.29.96.1",
      port: 7545,
      network_id: "5777"
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

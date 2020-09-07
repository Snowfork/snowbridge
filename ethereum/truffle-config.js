module.exports = {
  networks: {
    development: {
      host: "127.0.0.1",
      port: 7545,
      network_id: "*"
    }
  },
  mocha: {
    useColors: true
  },
  compilers: {
    solc: {
      version: "0.6.2",
    }
  }
};

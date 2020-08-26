const EthereumApp = artifacts.require("EthereumApp");
const ERC20App = artifacts.require("ERC20App");
const Decoder = artifacts.require("Decoder");
const TestToken = artifacts.require("TestToken");
const Verifier = artifacts.require("Verifier");

module.exports = function(deployer, network, accounts) {
  deployer.then(async () => {
    await deployer.deploy(EthereumApp);
    await deployer.deploy(ERC20App);

    await deployer.deploy(TestToken, 100000000, "Test Token", "TEST");

    await deployer.deploy(Verifier, accounts[0]);
  })
};

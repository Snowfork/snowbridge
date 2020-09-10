const ETHApp = artifacts.require("ETHApp");
const ERC20App = artifacts.require("ERC20App");
const TestToken = artifacts.require("TestToken");
const Verifier = artifacts.require("Verifier");

module.exports = function(deployer, network, accounts) {
  deployer.then(async () => {
    await deployer.deploy(ETHApp);
    await deployer.deploy(ERC20App);

    await deployer.deploy(TestToken, 100000000, "Test Token", "TEST");

    await deployer.deploy(Verifier, accounts[0]);
    await deployer.deploy(Broker, Verifier, [ETHApp, ERC20App]);
  })
};

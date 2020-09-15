const Verifier = artifacts.require("Verifier");
const ETHApp = artifacts.require("ETHApp");
const ERC20App = artifacts.require("ERC20App");
const Bridge = artifacts.require("Bridge");
const TestToken = artifacts.require("TestToken");

module.exports = function(deployer, network, accounts) {
  deployer.then(async () => {
    // Deploy Verifier
    await deployer.deploy(Verifier);

    // Deploy Applications
    await deployer.deploy(ETHApp);
    await deployer.deploy(ERC20App);

    // Deploy Bridge
    await deployer.deploy(Bridge, Verifier, [ETHApp, ERC20App]);

    // Deploy TEST ERC20 token for testing
    await deployer.deploy(TestToken, 100000000, "Test Token", "TEST");
  })
};

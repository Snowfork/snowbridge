const Bank = artifacts.require("Bank");
const TestToken = artifacts.require("TestToken");
const Verifier = artifacts.require("Verifier");

module.exports = function(deployer, network, accounts) {
  deployer.then(async () => {
    await deployer.deploy(Bank);
    await deployer.deploy(TestToken, 100000000, "Test Token", "TEST");

    await deployer.deploy(Verifier, accounts[0]);
  })
};

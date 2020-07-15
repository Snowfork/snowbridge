const Bank = artifacts.require("Bank");
const TestToken = artifacts.require("TestToken");

module.exports = function(deployer) {
  deployer.then(async () => {
    await deployer.deploy(Bank);
    await deployer.deploy(TestToken, 100000000, "Test Token", "TEST");
  })
};

const Verifier = artifacts.require("Verifier");
const BasicOutChannel = artifacts.require("BasicOutChannel");
const IncentivizedOutChannel = artifacts.require("IncentivizedOutChannel");
const Decoder = artifacts.require("Decoder");
const ETHApp = artifacts.require("ETHApp");
const ERC20App = artifacts.require("ERC20App");
const Bridge = artifacts.require("Bridge");
const TestToken = artifacts.require("TestToken");

module.exports = function(deployer, network, accounts) {
  deployer.then(async () => {
    // Deploy Verifier and get deployed address
    const verifier = await deployer.deploy(Verifier, accounts[0]);

    // Deploy OutChannels and get deployed addresses
    const basicOutChannel = await deployer.deploy(BasicOutChannel);
    const incentivizedOutChannel = await deployer.deploy(IncentivizedOutChannel);

    // Link libraries to applications
    await deployer.deploy(Decoder);
    deployer.link(Decoder, [ETHApp, ERC20App]);

    // Deploy applications
    const ethApp = await deployer.deploy(ETHApp, basicOutChannel.address, incentivizedOutChannel.address);
    const erc20App = await deployer.deploy(ERC20App);

    // Deploy Bridge
    const bridge = await deployer.deploy(Bridge, verifier.address, [ethApp.address, erc20App.address]);

    // Deploy TEST ERC20 token for testing
    await deployer.deploy(TestToken, 100000000, "Test Token", "TEST");

  })
};

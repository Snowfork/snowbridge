const Verifier = artifacts.require("Verifier");
const BasicSendChannel = artifacts.require("BasicSendChannel");
const IncentivizedSendChannel = artifacts.require("IncentivizedSendChannel");
const Decoder = artifacts.require("Decoder");
const ETHApp = artifacts.require("ETHApp");
const ERC20App = artifacts.require("ERC20App");
const Bridge = artifacts.require("Bridge");
const TestToken = artifacts.require("TestToken");

module.exports = function(deployer, network, accounts) {
  deployer.then(async () => {
    // Deploy Verifier and get deployed address
    const verifier = await deployer.deploy(Verifier, accounts[0]);

    // Deploy SendChannels and get deployed addresses
    const basicSendChannel = await deployer.deploy(BasicSendChannel);
    const incentivizedSendChannel = await deployer.deploy(IncentivizedSendChannel);

    // Link libraries to applications
    await deployer.deploy(Decoder);
    deployer.link(Decoder, [ETHApp, ERC20App]);

    // Deploy applications
    const ethApp = await deployer.deploy(ETHApp, basicSendChannel.address, incentivizedSendChannel.address);
    const erc20App = await deployer.deploy(ERC20App, basicSendChannel.address, incentivizedSendChannel.address);

    // Deploy Bridge
    const bridge = await deployer.deploy(Bridge, verifier.address, [ethApp.address, erc20App.address]);

    // Deploy TEST ERC20 token for testing
    await deployer.deploy(TestToken, 100000000, "Test Token", "TEST");

  })
};

const Verifier = artifacts.require("Verifier");

const Decoder = artifacts.require("Decoder");
const ETHApp = artifacts.require("ETHApp");
const ERC20App = artifacts.require("ERC20App");
const Bridge = artifacts.require("Bridge");
const TestToken = artifacts.require("TestToken");

const channelContracts = {
  basic: {
    inbound: artifacts.require("BasicInboundChannel"),
    outbound: artifacts.require("BasicOutboundChannel")
  },
  incentivized: {
    inbound: artifacts.require("IncentivizedInboundChannel"),
    outbound: artifacts.require("IncentivizedOutboundChannel")
  }, 
}

const channels = {
  basic: {
    inbound: null,
    outbound: null
  },
  incentivized: {
    inbound: null,
    outbound: null
  }, 
} 

module.exports = function(deployer, network, accounts) {
  deployer.then(async () => {

    channels.basic.inbound = await deployer.deploy(channelContracts.basic.inbound)
    channels.basic.outbound = await deployer.deploy(channelContracts.basic.inbound)
    channels.incentivized.inbound = await deployer.deploy(channelContracts.incentivized.inbound)
    channels.incentivized.outbound = await deployer.deploy(channelContracts.incentivized.outbound)

    // Link libraries to applications
    await deployer.deploy(Decoder);
    deployer.link(Decoder, [ETHApp, ERC20App]);

    // Deploy applications
    const ethApp = await deployer.deploy(
      ETHApp,
      channels.basic.outbound.address,
      channels.incentivized.outbound.address
    );

    const erc20App = await deployer.deploy(
      ERC20App,
      channels.basic.outbound.address,
      channels.incentivized.outbound.address
    );

    // Deploy TEST ERC20 token for testing
    await deployer.deploy(TestToken, 100000000, "Test Token", "TEST");
  })
};

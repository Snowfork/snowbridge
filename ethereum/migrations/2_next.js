const Decoder = artifacts.require("Decoder");
const ETHApp = artifacts.require("ETHApp");
const ERC20App = artifacts.require("ERC20App");
const TestToken = artifacts.require("TestToken");

const channels = {
  basic: {
    inbound: {
      contract: artifacts.require("BasicInboundChannel"),
      instance: null
    },
    outbound: {
      contract: artifacts.require("BasicOutboundChannel"),
      instance: null,
    }
  },
  incentivized: {
    inbound: {
      contract: artifacts.require("IncentivizedInboundChannel"),
      instance: null 
    },
    outbound: {
      contract: artifacts.require("IncentivizedOutboundChannel"),
      instance: null 
    }
  },
}

module.exports = function(deployer, network, accounts) {
  deployer.then(async () => {

    channels.basic.inbound.instance = await deployer.deploy(channels.basic.inbound.contract)
    channels.basic.outbound.instance = await deployer.deploy(channels.basic.outbound.contract)
    channels.incentivized.inbound.instance = await deployer.deploy(channels.incentivized.inbound.contract)
    channels.incentivized.outbound.instance = await deployer.deploy(channels.incentivized.outbound.contract)


    // Link libraries to applications
    await deployer.deploy(Decoder);
    deployer.link(Decoder, [ETHApp, ERC20App]);

    // Deploy applications
    const ethApp = await deployer.deploy(
      ETHApp,
      {
        inbound: channels.basic.inbound.instance.address,
        outbound: channels.basic.outbound.instance.address,
      },
      {
        inbound: channels.incentivized.inbound.instance.address,
        outbound: channels.incentivized.outbound.instance.address,
      },
    );

    // const erc20App = await deployer.deploy(
    //   ERC20App,
    //   channels.basic.outbound.address,
    //   channels.incentivized.outbound.address
    // );

    // Deploy TEST ERC20 token for testing
    await deployer.deploy(TestToken, 100000000, "Test Token", "TEST");
  })
};

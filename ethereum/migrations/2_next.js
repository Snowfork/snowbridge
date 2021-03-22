const ScaleCodec = artifacts.require("ScaleCodec");
const ETHApp = artifacts.require("ETHApp");
const ERC20App = artifacts.require("ERC20App");
const TestToken = artifacts.require("TestToken");
const ValidatorRegistry = artifacts.require("ValidatorRegistry");
const Web3Utils = require("web3-utils");

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

const contracts = {
  lightclientbridge: {
    contract: artifacts.require("LightClientBridge"),
    instance: null
  }
}

module.exports = function (deployer, network, accounts) {
  deployer.then(async () => {
    channels.basic.inbound.instance = await deployer.deploy(channels.basic.inbound.contract)
    channels.basic.outbound.instance = await deployer.deploy(channels.basic.outbound.contract)
    channels.incentivized.inbound.instance = await deployer.deploy(channels.incentivized.inbound.contract)
    channels.incentivized.outbound.instance = await deployer.deploy(channels.incentivized.outbound.contract)

    // Link libraries to applications
    await deployer.deploy(ScaleCodec);
    deployer.link(ScaleCodec, [ETHApp, ERC20App]);

    // Deploy applications
    await deployer.deploy(
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

    await deployer.deploy(
      ERC20App,
      {
        inbound: channels.basic.inbound.instance.address,
        outbound: channels.basic.outbound.instance.address,
      },
      {
        inbound: channels.incentivized.inbound.instance.address,
        outbound: channels.incentivized.outbound.instance.address,
      },
    );

    await deployer.deploy(TestToken, 100000000, "Test Token", "TEST");

    const valRegistry = await deployer.deploy(ValidatorRegistry, "0x383e21111d06dbc3ec0e3030b530ca766d6a821cc1203ce4d253321b5e88b045");
    contracts.lightclientbridge.instance = await deployer.deploy(contracts.lightclientbridge.contract, valRegistry.address)
  })
};

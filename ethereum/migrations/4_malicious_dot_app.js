require("dotenv").config();

const MaliciousDOTApp = artifacts.require("MaliciousDOTApp");
const ScaleCodec = artifacts.require("ScaleCodec");
const BasicInboundChannel = artifacts.require("BasicInboundChannel");
const BasicOutboundChannel = artifacts.require("BasicOutboundChannel");
const IncentivizedInboundChannel = artifacts.require("IncentivizedInboundChannel");
const IncentivizedOutboundChannel = artifacts.require("IncentivizedOutboundChannel");

module.exports = function (deployer, network) {

  deployer.then(async () => {

    if (network === 'ropsten' || network === 'e2e_test') {
      deployer.link(ScaleCodec, [MaliciousDOTApp]);

      const basicInboundChannelInstance = await BasicInboundChannel.deployed()
      const basicOutboundChannelInstance = await BasicOutboundChannel.deployed()
      const incentivizedInboundChannelInstance = await IncentivizedInboundChannel.deployed()
      const incentivizedOutboundChannelInstance = await IncentivizedOutboundChannel.deployed()

      await deployer.deploy(
        MaliciousDOTApp,
        "Snowfork DOT",
        "SnowDOT",
        incentivizedOutboundChannelInstance.address,
        {
          inbound: basicInboundChannelInstance.address,
          outbound: basicOutboundChannelInstance.address,
        },
        {
          inbound: incentivizedInboundChannelInstance.address,
          outbound: incentivizedOutboundChannelInstance.address,
        },
      );
    }

  })
};

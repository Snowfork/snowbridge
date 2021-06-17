require("dotenv").config();

const MaliciousERC20App = artifacts.require("MaliciousERC20App");
const ScaleCodec = artifacts.require("ScaleCodec");
const BasicInboundChannel = artifacts.require("BasicInboundChannel");
const BasicOutboundChannel = artifacts.require("BasicOutboundChannel");
const IncentivizedInboundChannel = artifacts.require("IncentivizedInboundChannel");
const IncentivizedOutboundChannel = artifacts.require("IncentivizedOutboundChannel");

module.exports = function (deployer, network, accounts) {
  const administrator = accounts[0];

  deployer.then(async () => {

    if (network === 'development') {
      return
    }
    deployer.link(ScaleCodec, [MaliciousERC20App]);

    const basicInboundChannelInstance = await BasicInboundChannel.deployed()
    const basicOutboundChannelInstance = await BasicOutboundChannel.deployed()
    const incentivizedInboundChannelInstance = await IncentivizedInboundChannel.deployed()
    const incentivizedOutboundChannelInstance = await IncentivizedOutboundChannel.deployed()

    const maliciousERC20App = await deployer.deploy(
      MaliciousERC20App,
      {
        inbound: basicInboundChannelInstance.address,
        outbound: basicOutboundChannelInstance.address,
      },
      {
        inbound: incentivizedInboundChannelInstance.address,
        outbound: incentivizedOutboundChannelInstance.address,
      },
    );

    await incentivizedOutboundChannelInstance.authorizeDefaultOperator(
      maliciousERC20App.address,
      { from: administrator }
    );

  })
};

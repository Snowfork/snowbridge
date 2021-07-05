require("dotenv").config();

const BasicInboundChannel = artifacts.require("BasicInboundChannel");
const TestBasicInboundChannelProxy = artifacts.require("TestBasicInboundChannelProxy");

module.exports = function (deployer, network, accounts) {
  deployer.then(async () => {

    if (network === 'ropsten' || network === 'e2e_test') {
      const basicChannelInstance = await BasicInboundChannel.deployed();
      const channelProxy = await deployer.deploy(TestBasicInboundChannelProxy, basicChannelInstance.address)
      await basicChannelInstance.transferOwnership(channelProxy.address)
    }

  });
}

require("dotenv").config();

const BasicInboundChannel = artifacts.require("BasicInboundChannel");
const BasicInboundChannelProxy = artifacts.require("BasicInboundChannelProxy");

module.exports = function (deployer, network, accounts) {
  deployer.then(async () => {

    if (network === 'ropsten' || network === 'e2e_test') {
      const basicChannelInstance = await BasicInboundChannel.deployed();
      const channelProxy = await deployer.deploy(BasicInboundChannelProxy, basicChannelInstance.address)
      await basicChannelInstance.transferOwnership(channelProxy.address)
    }

  });
}

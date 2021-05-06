const BigNumber = require('bignumber.js');
const rlp = require("rlp");
const { ethers } = require("ethers");

const assert = require('chai').assert;

const MockFeeSource = artifacts.require("MockFeeSource");
const MockRewardSource = artifacts.require("MockRewardSource");

const channelContracts = {
  basic: {
    inbound: artifacts.require("BasicInboundChannel"),
    outbound: artifacts.require("BasicOutboundChannel"),
  },
  incentivized: {
    inbound: artifacts.require("IncentivizedInboundChannel"),
    outbound: artifacts.require("IncentivizedOutboundChannel"),
  },
};

const confirmBasicChannelSend = (channelEvent, channelAddress, sendingAppAddress, expectedNonce = 0, expectedPayload) => {
  var abi = ["event Message(address source, uint64 nonce, bytes payload)"];
  var iface = new ethers.utils.Interface(abi);
  let decodedEvent = iface.decodeEventLog('Message(address,uint64,bytes)', channelEvent.data, channelEvent.topics);

  channelEvent.address.should.be.equal(channelAddress);
  decodedEvent.source.should.be.equal(sendingAppAddress);

  assert(decodedEvent.nonce.eq(ethers.BigNumber.from(expectedNonce)));
  if (expectedPayload) {
    decodedEvent.payload.should.be.equal(expectedPayload);
  }
};

const deployAppWithMockChannels = async (deployer, channels, appContract, ...appContractArgs) => {
  const app = await appContract.new(
    ...appContractArgs,
    {
      inbound: channels[0],
      outbound: channels[1],
    },
    {
      inbound: channels[0],
      outbound: channels[1],
    },
    {
      from: deployer,
    }
  );

  return app;
}

const addressBytes = (address) => Buffer.from(address.replace(/^0x/, ""), "hex");

const BASIC_CHANNEL = 0;
const INCENTIVIZED_CHANNEL = 1;

const ChannelId = {
  Basic: 0,
  Incentivized: 1,
}

const encodeLog = (log) => {
  return rlp.encode([log.address, log.topics, log.data]).toString("hex")
}

module.exports = {
  deployAppWithMockChannels,
  addressBytes,
  ChannelId,
  encodeLog,
};

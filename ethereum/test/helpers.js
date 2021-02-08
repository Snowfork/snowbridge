const ethers = require("ethers");
const BigNumber = require('bignumber.js');
const rlp = require("rlp");

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

const confirmChannelSend = (channelEvent, channelAddress, sendingAppAddress, expectedNonce = 0, expectedPayload) => {
  outChannelLogFields = [
    {
      type: 'address',
      name: 'source'
    },
    {
      type: 'uint64',
      name: 'nonce'
    },
    {
      type: 'bytes',
      name: 'payload',
    }
  ];

  const decodedEvent = web3.eth.abi.decodeLog(outChannelLogFields, channelEvent.data, channelEvent.topics);

  channelEvent.address.should.be.equal(channelAddress);
  decodedEvent.source.should.be.equal(sendingAppAddress);
  decodedEvent.nonce.should.be.equal('' + expectedNonce);
  if (expectedPayload) {
    decodedEvent.payload.should.be.equal(expectedPayload);
  }
};

const confirmUnlock = (rawEvent, ethAppAddress, expectedRecipient, expectedAmount) => {
  unlockLogFields = [
    {
      type: 'bytes32',
      name: 'sender'
    },
    {
      type: 'address',
      name: 'recipient'
    },
    {
      type: 'uint256',
      name: 'amount'
    }
  ];

  const decodedEvent = web3.eth.abi.decodeLog(unlockLogFields, rawEvent.data, rawEvent.topics);

  rawEvent.address.should.be.equal(ethAppAddress);
  decodedEvent.recipient.should.be.equal(expectedRecipient);
  decodedEvent.amount.should.be.bignumber.equal(expectedAmount);
};

const confirmUnlockTokens = (rawEvent, erc20AppAddress, expectedRecipient, expectedAmount) => {
  unlockLogFields = [
    {
      type: 'address',
      name: 'token'
    },
    {
      type: 'bytes32',
      name: 'sender'
    },
    {
      type: 'address',
      name: 'recipient'
    },
    {
      type: 'uint256',
      name: 'amount'
    }
  ];

  const decodedEvent = web3.eth.abi.decodeLog(unlockLogFields, rawEvent.data, rawEvent.topics);

  rawEvent.address.should.be.equal(erc20AppAddress);
  decodedEvent.recipient.should.be.equal(expectedRecipient);
  decodedEvent.amount.should.be.bignumber.equal(expectedAmount);
};

const confirmMessageDispatched = (rawEvent, expectedNonce, expectedResult) => {
  messageDispatchedLogFields = [{
    type: 'uint64',
    name: 'nonce'
  }, {
    type: 'bool',
    name: 'result'
  }];

  const decodedEvent = web3.eth.abi.decodeLog(messageDispatchedLogFields, rawEvent.data, rawEvent.topics);

  parseFloat(decodedEvent.nonce).should.be.equal(expectedNonce);
  decodedEvent.result.should.be.equal(expectedResult);
};


const hashMessage = (message) => {
  return ethers.utils.solidityKeccak256(
    ['address', 'uint256', 'bytes'],
    [message.target, message.nonce, message.payload]
  );
}

const deployAppContractWithChannels = async (appContract) => {
  const channels = {
    basic: {
      inbound: await channelContracts.basic.inbound.new(),
      outbound: await channelContracts.basic.outbound.new(),
    },
    incentivized: {
      inbound: await channelContracts.incentivized.inbound.new(),
      outbound: await channelContracts.incentivized.outbound.new(),
    },
  };

  const app = await appContract.new(
    {
      inbound: channels.basic.inbound.address,
      outbound: channels.basic.outbound.address,
    },
    {
      inbound: channels.incentivized.inbound.address,
      outbound: channels.incentivized.outbound.address,
    },
  );

  return [channels, app]
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
  confirmChannelSend,
  confirmUnlock,
  confirmUnlockTokens,
  confirmMessageDispatched,
  hashMessage,
  deployAppContractWithChannels,
  addressBytes,
  ChannelId,
  encodeLog,
};

const ethers = require("ethers");
const BigNumber = require('bignumber.js');

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

const confirmMessageDelivered = (rawEvent, expectedNonce, expectedResult) => {
  messageDeliveredLogFields = [{
    type: 'uint64',
    name: 'nonce'
  }, {
    type: 'bool',
    name: 'result'
  }];

  const decodedEvent = web3.eth.abi.decodeLog(messageDeliveredLogFields, rawEvent.data, rawEvent.topics);

  parseFloat(decodedEvent._nonce).should.be.equal(expectedNonce);
  decodedEvent._result.should.be.equal(expectedResult);
};


const hashMessage = (message) => {
  return ethers.utils.solidityKeccak256(
    ['uint256', 'string', 'address', 'bytes'],
    [message.nonce, message.senderApplicationId, message.targetApplicationAddress, message.payload]
  );
}

const deployContracts = async (channelContracts, appContract) => {
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

module.exports = {
  confirmChannelSend,
  confirmUnlock,
  confirmUnlockTokens,
  confirmMessageDelivered,
  hashMessage,
  deployContracts,
  addressBytes,
  ChannelId,
};

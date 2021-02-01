const ethers = require("ethers");
const BigNumber = require('bignumber.js');

const confirmChannelSend = (channelEvent, channelAddress, sendingAppAddress, expectedNonce = 0) => {
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

const confirmMessageDelivered = (rawEvent, expectedNonce, expectedResult) => {
    messageDeliveredLogFields = [{
        type: 'uint64',
        name: '_nonce'
    }, {
        type: 'bool',
        name: '_result'
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

module.exports = { confirmChannelSend, confirmUnlock, confirmMessageDelivered, hashMessage };

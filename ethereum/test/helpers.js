const confirmChannelSend = (channelEvent, channelAddress, sendingAppAddress, expectedTargetApplicationId, expectedPayload, expectedNonce = 0) => {
    outChannelLogFields = [{
        type: 'uint256',
        name: 'nonce'
    }, {
        type: 'address',
        name: 'senderAddress'
    }, {
        type: 'string',
        name: 'targetApplicationId',
    }, {
        type: 'bytes',
        name: 'payload',
    }];

    const decodedEvent = web3.eth.abi.decodeLog(outChannelLogFields, channelEvent.data, channelEvent.topics);

    channelEvent.address.should.be.equal(channelAddress);
    decodedEvent.nonce.should.be.equal('' + expectedNonce);
    decodedEvent.senderAddress.should.be.equal(sendingAppAddress);
    decodedEvent.targetApplicationId.should.be.equal(expectedTargetApplicationId);
    decodedEvent.payload.should.be.equal(expectedPayload);
};

const confirmUnlock = (rawEvent, ethAppAddress, expectedRecipient, expectedAmount) => {
    unlockLogFields = [{
        type: 'address',
        name: '_recipient'
    }, {
        type: 'uint256',
        name: '_amount'
    }];

    const decodedEvent = web3.eth.abi.decodeLog(unlockLogFields, rawEvent.data, rawEvent.topics);

    rawEvent.address.should.be.equal(ethAppAddress);
    decodedEvent._recipient.should.be.equal(expectedRecipient);
    parseFloat(decodedEvent._amount).should.be.equal(expectedAmount);
};

module.exports = { confirmChannelSend, confirmUnlock };

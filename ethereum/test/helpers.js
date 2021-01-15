const ethers = require("ethers");

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

    const decodedEvent2 = ethers.utils.defaultAbiCoder.decode(
        ['uint256', 'address', 'string', 'bytes'],
        channelEvent.data
    );
    const decodedEvent = web3.eth.abi.decodeLog(outChannelLogFields, channelEvent.data, channelEvent.topics);

    channelEvent.address.should.be.equal(channelAddress);
    decodedEvent.nonce.should.be.equal('' + expectedNonce);
    decodedEvent.senderAddress.should.be.equal(sendingAppAddress);
    decodedEvent.targetApplicationId.should.be.equal(expectedTargetApplicationId);
    decodedEvent.payload.should.be.equal(expectedPayload);
};

module.exports = { confirmChannelSend };
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

    const decodedEvent = web3.eth.abi.decodeLog(outChannelLogFields, channelEvent.data, channelEvent.topics);

    channelEvent.address.should.be.equal(channelAddress);
    decodedEvent.nonce.should.be.equal('' + expectedNonce);
    decodedEvent.senderAddress.should.be.equal(sendingAppAddress);
    decodedEvent.targetApplicationId.should.be.equal(expectedTargetApplicationId);
    decodedEvent.payload.should.be.equal(expectedPayload);
};

const confirmUnlock = (rawEvent, polkadotSender, ethAppAddress, expectedRecipient, expectedAmount) => {
    unlockLogFields = [{
        type: 'bytes32',
        name: '_polkadotSender'
    }, {
        type: 'address',
        name: '_recipient'
    }, {
        type: 'uint256',
        name: '_amount'
    }];

    const decodedEvent = web3.eth.abi.decodeLog(unlockLogFields, rawEvent.data, rawEvent.topics);

    rawEvent.address.should.be.equal(ethAppAddress);
    decodedEvent._polkadotSender.should.be.equal(polkadotSender);
    decodedEvent._recipient.should.be.equal(expectedRecipient);
    parseFloat(decodedEvent._amount).should.be.equal(expectedAmount);
};

const confirmMessageDelivered = (rawEvent, expectedNonce, expectedResult) => {
    messageDeliveredLogFields = [{
        type: 'uint256',
        name: '_nonce'
    }, {
        type: 'bool',
        name: '_result'
    }];

    const decodedEvent = web3.eth.abi.decodeLog(messageDeliveredLogFields, rawEvent.data, rawEvent.topics);

    parseFloat(decodedEvent._nonce).should.be.equal(expectedNonce);
    decodedEvent._result.should.be.equal(expectedResult);
};

const buildPackedCommitment = (messages) => {
    let messagesBytes;
    for(var i = 0; i < messages.length; i++) {
        // Pack the message contents into a bytes field
        const message = ethers.utils.solidityPack(
            [ 'uint256', 'string', 'address', 'bytes' ],
            [ messages[i].nonce, messages[i].senderApplicationId, messages[i].targetApplicationAddress, messages[i].payload ]
        );

        // Append the message's bytes to any previous message bytes
        if(i == 0) {
            messagesBytes = message;
        } else {
            messagesBytes = ethers.utils.solidityPack(
                [ 'bytes', 'bytes' ],
                [ messagesBytes, message ]
            );
        }
    }

    // Hash the messages' bytes to convert from 'bytes memory' to 'bytes32' for saving gas on function arguments
    return ethers.utils.solidityKeccak256(
        [ 'bytes'],
        [ messagesBytes ]
    );
}

const buildCommitment = (messages) => {
    let dataStartOffset = 32;
    const prefix = ethers.utils.defaultAbiCoder.encode(
        [ 'uint256', 'uint256', 'uint256'],
        [ 2, 1, dataStartOffset ],
    )

    const messagesBytes = ethers.utils.defaultAbiCoder.encode(
        [ 'uint256', 'string', 'address', 'bytes' ],
        [ messages[0].nonce, messages[0].senderApplicationId, messages[0].targetApplicationAddress, messages[0].payload ]
    );

    let prefixedBytes = prefix + messagesBytes.slice(2, messagesBytes.length)
    console.log("loc:", prefixedBytes)

    // Hash the messages' bytes to convert from 'bytes memory' to 'bytes32' for saving gas on function arguments
    return ethers.utils.solidityKeccak256(
        [ 'bytes'],
        [ messagesBytes ]
    );
}


module.exports = { confirmChannelSend, confirmUnlock, confirmMessageDelivered, buildCommitment, buildPackedCommitment };

const uniqueFilename = require('unique-filename');
const TOML = require('@iarna/toml');
const fs = require('fs');
const os = require('os');

const channelContracts = {
    basic: {
        inbound: artifacts.require("BasicInboundChannel"),
        outbound: artifacts.require("BasicOutboundChannel")
    },
    incentivized: {
        inbound: artifacts.require("IncentivizedInboundChannel"),
        outbound: artifacts.require("IncentivizedOutboundChannel")
    },
}

const channels = {
    basic: {
        inbound: null,
        outbound: null
    },
    incentivized: {
        inbound: null,
        outbound: null
    },
}

const dump = (channels) => {

    const config = {
        ethereum: {
            endpoint: "ws://localhost:9545/",
            "descendants-until-final": 35,
            channels: {
                basic: {
                    inbound: channels.basic.inbound.address,
                    outbound: channels.basic.outbound.address,
                },
                incentivized: {
                    inbound: channels.incentivized.inbound.address,
                    outbound: channels.incentivized.outbound.address,
                },
            },
        },
        substrate: {
            endpoint: "ws://127.0.0.1:9944/"
        }
    }
    console.log(TOML.stringify(config))
}

module.exports = async (callback) => {
    try {
        channels.basic.inbound = await channelContracts.basic.inbound.deployed()
        channels.basic.outbound = await channelContracts.basic.outbound.deployed()
        channels.incentivized.inbound = await channelContracts.incentivized.inbound.deployed()
        channels.incentivized.outbound = await channelContracts.incentivized.outbound.deployed()
        dump(channels)
    } catch (error) {
        callback(error)
    }
}

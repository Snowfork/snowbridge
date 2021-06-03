const TOML = require('@iarna/toml');
const fs = require('fs');
const path = require('path');

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

const bridgeContracts = {
    lightclientbridge: artifacts.require("LightClientBridge"),
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

const bridge = {
    lightclientbridge: null
}

const dump = (tmpDir, channels, bridge) => {
    const config = {
        ethereum: {
            endpoint: "ws://localhost:8545/",
            startblock: 1,
            "descendants-until-final": 3,
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
            lightclientbridge: bridge.lightclientbridge.address
        },
        parachain: {
            endpoint: "ws://127.0.0.1:11144/"
        },
        relaychain: {
            endpoint: "ws://127.0.0.1:9944/"
        },
        database: {
            dialect: "sqlite3",
            dbpath: "tmp.db",
        },
        workers: {
            parachaincommitmentrelayer: {
                enabled: true,
                "restart-delay": 30,
            },
            beefyrelayer: {
                enabled: true,
                "restart-delay": 30,
            },
            ethrelayer: {
                enabled: true,
                "restart-delay": 30,
            },
        }
    }
    fs.writeFileSync(path.join(tmpDir, "config.toml"), TOML.stringify(config));
}

module.exports = async (callback) => {
    try {
        let configDir = process.argv[4].toString();
        bridge.lightclientbridge = await bridgeContracts.lightclientbridge.deployed();
        channels.basic.inbound = await channelContracts.basic.inbound.deployed();
        channels.basic.outbound = await channelContracts.basic.outbound.deployed();
        channels.incentivized.inbound = await channelContracts.incentivized.inbound.deployed();
        channels.incentivized.outbound = await channelContracts.incentivized.outbound.deployed();
        await dump(configDir, channels, bridge);
    } catch (error) {
        callback(error)
    }
}

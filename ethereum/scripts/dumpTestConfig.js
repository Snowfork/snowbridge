const TOML = require('@iarna/toml');
const fs = require('fs');
const path = require('path');
const { SubClient } = require('../../test/src/subclient');
const u8a = require('@polkadot/util/u8a');

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

const substrate = {
    endpoint: "ws://localhost:11144",
    account_whitelist: null,
}

const dump = (tmpDir, channels, substrate) => {
    const config = {
        ethereum: {
            endpoint: "ws://localhost:8545/",
            "descendants-until-final": 3,
            channels: {
                basic: {
                    inbound: channels.basic.inbound.address,
                    outbound: channels.basic.outbound.address,
                    account_whitelist: channels.basic.account_whitelist,
                },
                incentivized: {
                    inbound: channels.incentivized.inbound.address,
                    outbound: channels.incentivized.outbound.address,
                },
            },
        },
        substrate: {
            endpoint: substrate.endpoint,
            account_whitelist: substrate.account_whitelist,
        }
    }
    fs.writeFileSync(path.join(tmpDir, "config.toml"), TOML.stringify(config));
}

module.exports = async (callback) => {
    try {
        let configDir = process.argv[4].toString();
        let subClient = new SubClient(substrate.endpoint);
        await subClient.connect();
        let subAccounts = await subClient.api.query.system.account.entries();

        channels.basic.inbound = await channelContracts.basic.inbound.deployed();
        channels.basic.outbound = await channelContracts.basic.outbound.deployed();
        channels.basic.account_whitelist = await web3.eth.getAccounts();
        channels.incentivized.inbound = await channelContracts.incentivized.inbound.deployed();
        channels.incentivized.outbound = await channelContracts.incentivized.outbound.deployed();
        substrate.account_whitelist = subAccounts.map(account => {
            return u8a.u8aToHex(account[0].slice(-32));
        });
        subClient.disconnect();

        dump(configDir, channels, substrate);
    } catch (error) {
        callback(error)
    }
}

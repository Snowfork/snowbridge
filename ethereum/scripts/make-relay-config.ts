import { deployments } from "hardhat";
import TOML from '@iarna/toml';
import fs from 'fs';
import path from 'path';

const main = async () => {
    let channels = {
        basic: {
            inbound: await deployments.get("BasicInboundChannel"),
            outbound: await deployments.get("BasicOutboundChannel")
        },
        incentivized: {
            inbound: await deployments.get("IncentivizedInboundChannel"),
            outbound: await deployments.get("IncentivizedOutboundChannel")
        }
    }
    let beefy = await deployments.get("BeefyLightClient");

    const config = {
        global: {
            "data-dir": "/tmp/snowbridge-e2e-config",
        },
        ethereum: {
            endpoint: "ws://localhost:8546/",
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
            beefylightclient: beefy.address
        },
        parachain: {
            endpoint: "ws://127.0.0.1:11144/"
        },
        relaychain: {
            endpoint: "ws://127.0.0.1:9944/"
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
    console.log(TOML.stringify(config));
}

main()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error(error);
        process.exit(1);
    });

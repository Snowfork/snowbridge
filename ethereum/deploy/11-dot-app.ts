require("dotenv").config();

import { singletons } from "@openzeppelin/test-helpers";

import {HardhatRuntimeEnvironment} from "hardhat/types";

module.exports = async ({
    deployments,
    getUnnamedAccounts,
    network,
}: HardhatRuntimeEnvironment) => {
  let [deployer] = await getUnnamedAccounts();

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

  let scaleCodecLibrary = await deployments.get("ScaleCodec")

  if (['hardhat', 'localhost'].includes(network.name)) {
    await singletons.ERC1820Registry(deployer);
  }

  await deployments.deploy("DOTApp", {
    from: deployer,
    args:[
      "Snowfork DOT",
      "SnowDOT",
      channels.incentivized.outbound.address,
      {
        inbound: channels.basic.inbound.address,
        outbound: channels.basic.outbound.address,
      },
      {
        inbound: channels.incentivized.inbound.address,
        outbound: channels.incentivized.outbound.address,
      }
    ],
    libraries: {
        ScaleCodec: scaleCodecLibrary.address
    },
    log: true,
  });

};

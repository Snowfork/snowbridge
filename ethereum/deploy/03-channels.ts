import {HardhatRuntimeEnvironment} from "hardhat/types";

module.exports = async ({
    deployments,
    getUnnamedAccounts,
    ethers
}: HardhatRuntimeEnvironment) => {
  let [deployer] = await getUnnamedAccounts();

  if (!("BASIC_CHANNEL_SOURCE_ID" in process.env)) {
    throw "Missing BASIC_CHANNEL_SOURCE_ID in environment config"
  }
  const basicChannelSourceID = process.env.BASIC_CHANNEL_SOURCE_ID

  if (!("INCENTIVIZED_CHANNEL_SOURCE_ID" in process.env)) {
    throw "Missing INCENTIVIZED_CHANNEL_SOURCE_ID in environment config"
  }
  const incentivizedChannelSourceID = process.env.INCENTIVIZED_CHANNEL_SOURCE_ID

  let parachainClient = await deployments.get("ParachainClient")
  let scaleCodecLibrary = await deployments.get("ScaleCodec")

  await deployments.deploy("BasicInboundChannel", {
    from: deployer,
    args: [basicChannelSourceID, parachainClient.address],
    libraries: {
        ScaleCodec: scaleCodecLibrary.address,
    },
    log: true,
    autoMine: true,
  });

  await deployments.deploy("IncentivizedInboundChannel", {
    from: deployer,
    args:[incentivizedChannelSourceID, parachainClient.address],
    libraries: {
        ScaleCodec: scaleCodecLibrary.address,
    },
    log: true,
    autoMine: true,
  });

  await deployments.deploy("BasicOutboundChannel", {
    from: deployer,
    log: true,
    autoMine: true,
  });

  await deployments.deploy("IncentivizedOutboundChannel", {
    from: deployer,
    log: true,
    autoMine: true,
  });
};

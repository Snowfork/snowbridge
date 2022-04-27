import {HardhatRuntimeEnvironment} from "hardhat/types";

module.exports = async ({
    deployments,
    getUnnamedAccounts,
    ethers
}: HardhatRuntimeEnvironment) => {
  let [deployer] = await getUnnamedAccounts();

  let parachainClient = await deployments.get("ParachainClient")
  let scaleCodecLibrary = await deployments.get("ScaleCodec")

  await deployments.deploy("BasicInboundChannel", {
    from: deployer,
    args: [parachainClient.address],
    libraries: {
        ScaleCodec: scaleCodecLibrary.address,
    },
    log: true,
    autoMine: true,
  });

  await deployments.deploy("IncentivizedInboundChannel", {
    from: deployer,
    args:[parachainClient.address],
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

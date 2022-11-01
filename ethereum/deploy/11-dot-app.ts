import { singletons } from "@openzeppelin/test-helpers";

import { HardhatRuntimeEnvironment } from "hardhat/types";

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

  let tokenContract = await deployments.deploy("WrappedToken", {
    from: deployer,
    args: [
      "Wrapped DOT",
      "WDOT",
    ],
    log: true,
    autoMine: true,
  });

  let dotAppContract = await deployments.deploy("DOTApp", {
    from: deployer,
    args: [
      tokenContract.address,
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
    autoMine: true,
  });

  console.log("Configuring WrappedToken")
  await deployments.execute(
    "WrappedToken",
    {
      from: deployer,
      autoMine: true,
    },
    "transferOwnership",
    dotAppContract.address
  );
};

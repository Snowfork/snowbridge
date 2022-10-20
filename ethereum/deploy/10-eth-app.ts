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

  let vault = await deployments.deploy('EtherVault', {
    from: deployer,
    log: true,
    autoMine: true
  });

  let app = await deployments.deploy("ETHApp", {
    from: deployer,
    args:[
      channels.incentivized.inbound.address,
      vault.address,
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

  console.log("Transferring EtherVault ownership to ETHApp")
  await deployments.execute(
    "EtherVault", 
    {
      from: deployer,
      autoMine: true
    },
    "transferOwnership",
    app.address
  );
};

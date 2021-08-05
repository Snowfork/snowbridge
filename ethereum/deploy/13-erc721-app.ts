require("dotenv").config();

import { ethers } from "hardhat";
import {HardhatRuntimeEnvironment} from "hardhat/types";

module.exports = async ({
    deployments,
    getUnnamedAccounts,
    network,
}: HardhatRuntimeEnvironment) => {
  let [deployer, developer] = await getUnnamedAccounts();

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

  await deployments.deploy("ERC721App", {
    from: deployer,
    args:[
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

  await deployments.deploy("TestToken721", {
    from: deployer,
    args:["Test Token 721", "TEST721"],
    log: true,
    autoMine: true,
  });

  await deployments.deploy("TestToken721Enumerable", {
    from: deployer,
    args: ["Test Enum", "TESTE"],
    log: true,
  });

  const nft = await deployments.get('TestToken721Enumerable');
  const TestNft = await ethers.getContractAt('TestToken721Enumerable', nft.address);
  const signer = await ethers.getSigner(deployer);
  const NftWithSigner = await TestNft.connect(signer);

  for (let i = 0; i < 10; i++) {
    let tx = await NftWithSigner.mint(developer, Date.now().toString());
    await tx.wait();
  }
};

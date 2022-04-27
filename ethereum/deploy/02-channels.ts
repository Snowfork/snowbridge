import {HardhatRuntimeEnvironment} from "hardhat/types";

module.exports = async ({
    deployments,
    getUnnamedAccounts,
    ethers
}: HardhatRuntimeEnvironment) => {
  let [deployer] = await getUnnamedAccounts();

  let scaleCodecLibrary = await deployments.get("ScaleCodec")
  let merkleProofLibrary = await deployments.get("MerkleProof")
  let paraLibrary = await deployments.get("ParachainClient")
  let beefy = await deployments.get("BeefyClient")

  await deployments.deploy("BasicInboundChannel", {
    from: deployer,
    args: [beefy.address],
    libraries: {
        MerkleProof: merkleProofLibrary.address,
        ScaleCodec: scaleCodecLibrary.address,
        ParachainClient: paraLibrary.address
    },
    log: true,
    autoMine: true,
  });

  await deployments.deploy("IncentivizedInboundChannel", {
    from: deployer,
    args:[beefy.address],
    libraries: {
        MerkleProof: merkleProofLibrary.address,
        ScaleCodec: scaleCodecLibrary.address,
        ParachainClient: paraLibrary.address
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

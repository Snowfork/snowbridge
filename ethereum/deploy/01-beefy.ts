import { HardhatRuntimeEnvironment } from "hardhat/types";

module.exports = async ({
  deployments,
  getUnnamedAccounts,
  ethers
}: HardhatRuntimeEnvironment) => {
  let [deployer] = await getUnnamedAccounts();

  let scaleCodecLibrary = await deployments.get("ScaleCodec")
  let bitFieldLibrary = await deployments.get("Bitfield")
  let merkleProofLibrary = await deployments.get("MerkleProof")

  let mmr = await deployments.deploy("SimplifiedMMRVerification", {
    from: deployer,
    log: true,
    autoMine: true,
  });

  await deployments.deploy("BeefyClient", {
    from: deployer,
    args: [mmr.address],
    libraries: {
      MerkleProof: merkleProofLibrary.address,
      Bitfield: bitFieldLibrary.address,
      ScaleCodec: scaleCodecLibrary.address,
    },
    log: true,
    autoMine: true,
  });
};

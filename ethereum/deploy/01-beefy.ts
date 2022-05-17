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
  let mmrProofVerificationLibrary = await deployments.get("MMRProofVerification")

  await deployments.deploy("BeefyClient", {
    from: deployer,
    libraries: {
      MerkleProof: merkleProofLibrary.address,
      MMRProofVerification: mmrProofVerificationLibrary.address,
      Bitfield: bitFieldLibrary.address,
      ScaleCodec: scaleCodecLibrary.address,
    },
    log: true,
    autoMine: true,
  });
};

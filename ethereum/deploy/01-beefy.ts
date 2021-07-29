import { HardhatRuntimeEnvironment } from "hardhat/types";

module.exports = async ({
  deployments,
  getUnnamedAccounts,
  ethers
}: HardhatRuntimeEnvironment) => {
  let [deployer] = await getUnnamedAccounts();

  let root = "0x98f6718ff4e32a9404df85c2456de4cc179536107b6167acb20588412a67928d";
  let numValidators = 3;

  let scaleCodecLibrary = await deployments.get("ScaleCodec")
  let bitFieldLibrary = await deployments.get("Bitfield")
  let merkleProofLibrary = await deployments.get("MerkleProof")

  let registry = await deployments.deploy("ValidatorRegistry", {
    from: deployer,
    args: [root, numValidators, 0],
    libraries: {
      MerkleProof: merkleProofLibrary.address
    },
    log: true,
    autoMine: true,
  });

  let mmr = await deployments.deploy("MMRVerification", {
    from: deployer,
    log: true,
    autoMine: true,
  });

  let beefy = await deployments.deploy("BeefyLightClient", {
    from: deployer,
    args: [registry.address, mmr.address, 0],
    libraries: {
      Bitfield: bitFieldLibrary.address,
      ScaleCodec: scaleCodecLibrary.address,
    },
    log: true,
    autoMine: true,
  });
};

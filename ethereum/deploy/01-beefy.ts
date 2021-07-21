import {HardhatRuntimeEnvironment} from "hardhat/types";

module.exports = async ({
    deployments,
    getUnnamedAccounts,
    ethers
}: HardhatRuntimeEnvironment) => {
  let [deployer] = await getUnnamedAccounts();

  let root = "0x697ea2a8fe5b03468548a7a413424a6292ab44a82a6f5cc594c3fa7dda7ce402";
  let numValidators = 2;

  let scaleCodecLibrary = await deployments.get("ScaleCodec")
  let bitFieldLibrary = await deployments.get("Bitfield")
  let merkleProofLibrary = await deployments.get("MerkleProof")

  let registry = await deployments.deploy("ValidatorRegistry", {
    from: deployer,
    args:[root, numValidators, 0],
    libraries: {
        MerkleProof: merkleProofLibrary.address
    },
    log: true,
  });

  let mmr = await deployments.deploy("MMRVerification", {
    from: deployer,
    log: true,
  });

  let beefy = await deployments.deploy("BeefyLightClient", {
    from: deployer,
    args: [registry.address, mmr.address, 0],
    libraries: {
        Bitfield: bitFieldLibrary.address,
        ScaleCodec: scaleCodecLibrary.address,
    },
    log: true,
  });
};

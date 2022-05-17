import { HardhatRuntimeEnvironment } from "hardhat/types";

module.exports = async ({
  deployments,
  getUnnamedAccounts,
  ethers
}: HardhatRuntimeEnvironment) => {
  let [deployer] = await getUnnamedAccounts();

  if (!("PARACHAIN_ID" in process.env)) {
    throw "Missing PARACHAIN_ID in environment config"
  }
  const paraID = process.env.PARACHAIN_ID

  let merkleProofLibrary = await deployments.get("MerkleProof")
  let scaleCodecLibrary = await deployments.get("ScaleCodec")

  let beefyClient = await deployments.get("BeefyClient")

  await deployments.deploy("ParachainClient", {
    from: deployer,
    args: [beefyClient.address, paraID],
    libraries: {
      MerkleProof: merkleProofLibrary.address,
      ScaleCodec: scaleCodecLibrary.address,
    },
    log: true,
    autoMine: true,
  });
};

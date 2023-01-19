import { HardhatRuntimeEnvironment } from "hardhat/types"

module.exports = async ({ ethers, deployments, getUnnamedAccounts }: HardhatRuntimeEnvironment) => {
    let [deployer] = await getUnnamedAccounts()

  const feeData = await ethers.provider.getFeeData()

  await deployments.deploy('Bitfield', {
    from: deployer,
    maxFeePerGas: feeData.maxFeePerGas,
    maxPriorityFeePerGas: feeData.maxPriorityFeePerGas,
  });

  await deployments.deploy('MerkleProof', {
    from: deployer,
    maxFeePerGas: feeData.maxFeePerGas,
    maxPriorityFeePerGas: feeData.maxPriorityFeePerGas,
  });

  await deployments.deploy('MMRProof', {
    from: deployer,
    maxFeePerGas: feeData.maxFeePerGas,
    maxPriorityFeePerGas: feeData.maxPriorityFeePerGas,
  });
};

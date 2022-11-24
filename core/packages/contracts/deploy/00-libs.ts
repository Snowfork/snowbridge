import {HardhatRuntimeEnvironment} from 'hardhat/types';
const hre = require("hardhat")

module.exports = async ({deployments, getUnnamedAccounts}: HardhatRuntimeEnvironment) => {
  let [deployer] = await getUnnamedAccounts();

  const feeData = await hre.ethers.provider.getFeeData()

  await deployments.deploy('ScaleCodec', {
    from: deployer,
    log: true,
    autoMine: true,
    maxFeePerGas: feeData.maxFeePerGas,
    maxPriorityFeePerGas: feeData.maxPriorityFeePerGas,
  });

  await deployments.deploy('Bitfield', {
    from: deployer,
    log: true,
    autoMine: true,
    maxFeePerGas: feeData.maxFeePerGas,
    maxPriorityFeePerGas: feeData.maxPriorityFeePerGas,
  });

  await deployments.deploy('MerkleProof', {
    from: deployer,
    log: true,
    autoMine: true,
    maxFeePerGas: feeData.maxFeePerGas,
    maxPriorityFeePerGas: feeData.maxPriorityFeePerGas,
  });

  await deployments.deploy('MMRProofVerification', {
    from: deployer,
    log: true,
    autoMine: true,
    maxFeePerGas: feeData.maxFeePerGas,
    maxPriorityFeePerGas: feeData.maxPriorityFeePerGas,
  });
};

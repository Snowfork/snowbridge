import {HardhatRuntimeEnvironment} from 'hardhat/types';

module.exports = async ({deployments, getUnnamedAccounts}: HardhatRuntimeEnvironment) => {
  let [deployer] = await getUnnamedAccounts();

  let codec = await deployments.deploy('ScaleCodec', {
    from: deployer,
    log: true,
    autoMine: true,
  });

  await deployments.deploy('MerkleProof', {
    from: deployer,
    log: true,
    autoMine: true,
  });

  await deployments.deploy('Bitfield', {
    from: deployer,
    log: true,
    autoMine: true,
  });

  await deployments.deploy('ParachainLightClient', {
    from: deployer,
    libraries: {
      ScaleCodec: codec.address
    },
    log: true,
    autoMine: true,
  });

};

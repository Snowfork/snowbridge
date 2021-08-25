let { ApiPromise, WsProvider } = require('@polkadot/api');

const relaychainEndpoint = process.env.RELAYCHAIN_ENDPOINT;

async function configureBeefy() {
  const hre = require("hardhat");

  const signer = await hre.ethers.getSigner()

  const beefyDeployment = await hre.deployments.get("BeefyLightClient");

  const validatorRegistryDeployment = await hre.deployments.get("ValidatorRegistry");
  const validatorRegistryContract = await new hre.ethers.Contract(validatorRegistryDeployment.address,
    validatorRegistryDeployment.abi);

  const validatorRegistry = await validatorRegistryContract.connect(signer)

  const relayChainProvider = new WsProvider(relaychainEndpoint);
  const relaychainAPI = await ApiPromise.create({
    provider: relayChainProvider,
  })

  const authorities = await relaychainAPI.query.mmrLeaf.beefyNextAuthorities()
  const root = authorities.root.toString();
  const numValidators = authorities.len.toString();
  const id = authorities.id.toString();


  console.log("Configuring ValidatorRegistry with updated validators")
  console.log({
    root, numValidators, id
  });

  await validatorRegistry.update(root, numValidators, id)

  console.log("Transferring ownership of ValidatorRegistry to BeefyLightClient")
  console.log({
    beefyAddress: beefyDeployment.address,
  });

  await validatorRegistry.transferOwnership(beefyDeployment.address)

  return;
}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
configureBeefy()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });

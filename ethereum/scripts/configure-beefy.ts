require("dotenv").config();
const hre = require("hardhat");

const SubClient = require('../../test/src/subclient').SubClient;

const relaychainEndpoint = "ws://localhost:9944";

async function main() {
  const [deployer] = await hre.getUnnamedAccounts();
  const signer = await hre.ethers.getSigner()

  const beefyDeployment = await hre.deployments.get("BeefyLightClient");

  const validatorRegistryDeployment = await hre.deployments.get("ValidatorRegistry");
  const validatorRegistryContract = await new hre.ethers.Contract(validatorRegistryDeployment.address,
    validatorRegistryDeployment.abi);

  const validatorRegistry = await validatorRegistryContract.connect(signer)

  const subClient = new SubClient(relaychainEndpoint);
  await subClient.connect();

  const authorities = await subClient.api.query.mmrLeaf.beefyNextAuthorities()
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
main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });

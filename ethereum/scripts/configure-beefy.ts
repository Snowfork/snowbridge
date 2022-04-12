let { ApiPromise, WsProvider } = require('@polkadot/api');

const relaychainEndpoint = process.env.RELAYCHAIN_ENDPOINT;

async function configureBeefy() {
  const hre = require("hardhat");

  const signer = await hre.ethers.getSigner()

  const beefyDeployment = await hre.deployments.get("BeefyLightClient");
  const beefyLightClientContract = await new hre.ethers.Contract(beefyDeployment.address,
    beefyDeployment.abi);
  const beefyLightClient = await beefyLightClientContract.connect(signer)

  const relayChainProvider = new WsProvider(relaychainEndpoint);
  const relaychainAPI = await ApiPromise.create({
    provider: relayChainProvider,
  })

  const authorities = await relaychainAPI.query.mmrLeaf.beefyNextAuthorities()
  const id = authorities.id.toString();
  const root = authorities.root.toString();
  const numValidators = authorities.len.toString();

  console.log("Configuring BeefyLightClient with initial BEEFY state")
  console.log({
    root, numValidators, id
  });

  await beefyLightClient.initialize(0, id, root, numValidators)

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

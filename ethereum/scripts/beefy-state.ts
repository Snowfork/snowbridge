async function beefyState() {
  let hre = require("hardhat");

  let signer = await hre.ethers.getSigner();

  let beefyDeployment = await hre.deployments.get("BeefyClient");
  let beefyLightClientContract = await new hre.ethers.Contract(
    beefyDeployment.address,
    beefyDeployment.abi
  );
  let beefyLightClient = await beefyLightClientContract.connect(signer);

  let [cur, next, latestMMRRoot, latestBeefyBlock] = await Promise.all(
    [
      beefyLightClient.currentValidatorSet(),
      beefyLightClient.nextValidatorSet(),
      beefyLightClient.latestMMRRoot(),
      beefyLightClient.latestBeefyBlock()
    ]
  )

  console.log({
    current: {
      id: cur.id.toString(),
    },
    next: {
      id: next.id.toString(),
    },
    latestMMRRoot,
    latestBeefyBlock: latestBeefyBlock.toString(),
  });

  return;
}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
beefyState()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });

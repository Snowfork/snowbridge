require("dotenv").config();

import {HardhatRuntimeEnvironment} from "hardhat/types";

module.exports = async ({
    deployments,
    getUnnamedAccounts,
    network,
}: HardhatRuntimeEnvironment) => {
  let [deployer] = await getUnnamedAccounts();

  let beefy = await deployments.get("BeefyLightClient");

  console.log("Configuring ValidatorRegistry")
  await deployments.execute(
    "ValidatorRegistry",
    {
      from: deployer,
      autoMine: true,
    },
    "transferOwnership",
    beefy.address
  );

  // Mark deployment to run only once
  return true;
};

module.exports.id = "configure-beefy"

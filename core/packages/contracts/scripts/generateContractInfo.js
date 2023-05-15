const fs = require("fs");
const path = require("path");
const NetworkId = process.env.ETH_NETWORK_ID || 15;
const DeployInfoFile = `./broadcast/DeployScript.sol/${NetworkId}/run-latest.json`;
const BuildInfoDir = "./out";
const DestFile =
  process.argv.length >= 3 ? process.argv[2] : (process.env["output_dir"] + "/contracts.json");

const run = async () => {
  let contracts = {};
  const deploymentInfo = JSON.parse(fs.readFileSync(DeployInfoFile, "utf8"));
  for (let transaction of deploymentInfo.transactions) {
    if (transaction.transactionType === "CREATE") {
      let contractName = transaction.contractName;
      if (contractName) {
        let contractInfo = { address: transaction.contractAddress };
        let contractBuildingInfo = JSON.parse(
          fs.readFileSync(
            path.join(
              BuildInfoDir,
              contractName + ".sol",
              contractName + ".json"
            ),
            "utf8"
          )
        );
        contractInfo.abi = contractBuildingInfo.abi;
        contracts[contractName] = contractInfo;
      }
    }
  }
  fs.writeFileSync(DestFile, JSON.stringify({ contracts }, null, 2), "utf8");
};

run()
  .then(() => {
    console.log("Contract File generated successfully");
    process.exit(0);
  })
  .catch((err) => {
    console.error(err);
  });

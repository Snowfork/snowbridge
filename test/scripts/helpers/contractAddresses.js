const fs = require("fs");
const Web3 = require("web3");

const path = "/tmp/snowbridge";

// Read Contracts from /tmp/snowbridge/contracts.json
const readContractAddress = "contracts.json"; 
// Store Contract Address in /tmp/snowbridge/contractAddresses.json
const writeContractAddress = "contractAddresses.json"; 
const provider = "http://localhost:8545";

async function contractAddresses(contractPath, inputContractAddress, outputContractAddress) {
  try {
    const contractJSON = fs.readFileSync(
      `${contractPath}/${inputContractAddress}`,
      "utf-8"
    );
    const contract = JSON.parse(contractJSON);

    const web3 = new Web3(new Web3.providers.HttpProvider(provider));

    const contractInstance = new web3.eth.Contract(
      contract.contracts.DOTApp.abi,
      contract.contracts.DOTApp.address
    );
    const SnowDOTAddress = await contractInstance.methods.token().call();
    const { name, chainId, contracts } = contract;

    let contractAddress = JSON.parse(JSON.stringify(contracts));

    // fetch address from individual contract and store them into separate file
    for (let contract in contracts) {
        if (!contracts[contract].address) {
            throw new Error(`${contract} contract address not found`);
        }
        contractAddress[contract] = contracts[contract].address;
    }
    contractAddress.SnowDOTAddress = SnowDOTAddress;
    contractAddress.name = name;
    contractAddress.chainId = chainId;

    fs.writeFileSync(
      `${contractPath}/${outputContractAddress}`,
      JSON.stringify(contractAddress, null, 4)
    );
  } catch (error) {
    console.error(error);
  }
}
contractAddresses(path, readContractAddress, writeContractAddress);
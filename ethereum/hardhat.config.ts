import { config as dotenv } from "dotenv";
import { resolve } from "path";
import "solidity-coverage"

dotenv({ path: resolve(__dirname, ".env") });

import "@nomiclabs/hardhat-truffle5";
import "@nomiclabs/hardhat-ethers";
import "@nomiclabs/hardhat-web3";
import "@nomiclabs/hardhat-etherscan";
import "hardhat-deploy";
import { HardhatUserConfig } from "hardhat/config";
import "./tasks/upgrade";
import "./tasks/renounce";

const getenv = (name: string) => {
  if (name in process.env) {
    return process.env[name]
  } else {
    throw new Error(`Please set your ${name} in a .env file`);
  }
}

const ropstenPrivateKey = getenv("ROPSTEN_PRIVATE_KEY");
const infuraKey = getenv("INFURA_PROJECT_ID");
const etherscanKey = getenv("ETHERSCAN_API_KEY");

const config: HardhatUserConfig = {
  networks: {
    hardhat: {
      accounts: {
        mnemonic: "stone speak what ritual switch pigeon weird dutch burst shaft nature shove"
      },
      chainId: 15,
    },
    localhost: {
      url: "http://127.0.0.1:8545",
      accounts: {
        mnemonic: "stone speak what ritual switch pigeon weird dutch burst shaft nature shove"
      },
      chainId: 15,
    },
    ropsten: {
      chainId: 3,
      url: `https://ropsten.infura.io/v3/${infuraKey}`,
      accounts: [ropstenPrivateKey],
      gas: 6000000,
      gasPrice: 5000000000,
    }
  },
  solidity: {
    version: "0.8.6"
  },
  paths: {
    sources: "contracts",
    deployments: '.deployments',
    tests: "test",
    cache: ".cache",
    artifacts: "artifacts"
  },
  mocha: {
    timeout: 60000
  },
  etherscan: {
    apiKey: etherscanKey
  }
};

export default config;

import { resolve } from "path";
import "solidity-coverage"

import "@nomiclabs/hardhat-truffle5";
import "@nomiclabs/hardhat-ethers";
import "@nomiclabs/hardhat-web3";
import "@nomiclabs/hardhat-etherscan";
import "hardhat-deploy";
import "./tasks/upgrade";
import "./tasks/renounce";
import "./tasks/contractAddress";
import type { HardhatUserConfig } from "hardhat/config";


let INFURA_KEY = process.env.INFURA_PROJECT_ID
let ROPSTEN_KEY = process.env.ROPSTEN_PRIVATE_KEY || "0x0000000000000000000000000000000000000000000000000000000000000000"
let ETHERSCAN_KEY = process.env.ETHERSCAN_API_KEY

const config: HardhatUserConfig = {
  networks: {
    hardhat: {
      accounts: {
        mnemonic: "stone speak what ritual switch pigeon weird dutch burst shaft nature shove",
        // Need to give huge account balance to test certain constraints in EthApp.sol::lock()
        accountsBalance: "350000000000000000000000000000000000000"
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
      url: `https://ropsten.infura.io/v3/${INFURA_KEY}`,
      accounts: [ROPSTEN_KEY],
      gas: 6000000,
    }
  },
  solidity: {
    version: "0.8.6",
    settings: {
      optimizer: {
        enabled: true,
        runs: 200,
      }
    }
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
    apiKey: ETHERSCAN_KEY
  }
};

export default config;

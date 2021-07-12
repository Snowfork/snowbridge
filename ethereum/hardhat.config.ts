import { config as dotenv } from "dotenv";
import { resolve } from "path";

dotenv({ path: resolve(__dirname, ".env") });

import "@nomiclabs/hardhat-truffle5";
import "@nomiclabs/hardhat-ethers";

const getenv = (name: string) => {
  if (name in process.env) {
    return process.env[name]
  } else {
    throw new Error(`Please set your ${name} in a .env file`);
  }
}

const mnemonic = getenv("MNEMONIC");
const infuraKey = getenv("INFURA_PROJECT_ID");

export default {
  networks: {
    hardhat: {
      throwOnTransactionFailures: false
    },
    localhost: {
      url: "http://127.0.0.1:8545",
      accounts: {
        mnemonic: 'stone speak what ritual switch pigeon weird dutch burst shaft nature shove',
      }
    },
    ropsten: {
      chainId: 3,
      url: `https://ropsten.infura.io/v3/${infuraKey}`,
      accounts: {
        mnemonic: mnemonic,
      }
    }
  },
  solidity: {
    version: "0.8.6"
  },
  paths: {
    sources: "contracts",
    tests: "test",
    cache: "cache",
    artifacts: "artifacts"
  },
};

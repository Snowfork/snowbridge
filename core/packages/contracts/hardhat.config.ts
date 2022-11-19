import "solidity-coverage"
import "@nomicfoundation/hardhat-chai-matchers"
import "@nomiclabs/hardhat-etherscan"
import "@nomiclabs/hardhat-ethers"
import "@typechain/hardhat"
import "hardhat-gas-reporter"
import "hardhat-deploy"
import "hardhat-contract-sizer"
import "hardhat-storage-layout"

import "./tasks/contractAddress"

import "tsconfig-paths/register"

import type { HardhatUserConfig } from "hardhat/config"

let INFURA_KEY = process.env.INFURA_PROJECT_ID
let ROPSTEN_KEY =
    process.env.ROPSTEN_PRIVATE_KEY ||
    "0x0000000000000000000000000000000000000000000000000000000000000000"
let ETHERSCAN_KEY = process.env.ETHERSCAN_API_KEY

const config: HardhatUserConfig = {
    networks: {
        hardhat: {
            accounts: {
                mnemonic:
                    "stone speak what ritual switch pigeon weird dutch burst shaft nature shove",
                // Need to give huge account balance to test certain constraints in EthApp.sol::lock()
                accountsBalance: "350000000000000000000000000000000000000"
            },
            chainId: 15
        },
        localhost: {
            url: "http://127.0.0.1:8545",
            accounts: {
                mnemonic:
                    "stone speak what ritual switch pigeon weird dutch burst shaft nature shove"
            },
            chainId: 15
        },
        ropsten: {
            chainId: 3,
            url: `https://ropsten.infura.io/v3/${INFURA_KEY}`,
            accounts: [ROPSTEN_KEY],
            gas: 6000000
        },
        goerli: {
            chainId: 5,
            url: `https://goerli.infura.io/v3/${INFURA_KEY}`,
            accounts: [ROPSTEN_KEY],
            gas: 6000000
        }
    },
    solidity: {
        version: "0.8.9",
        settings: {
            optimizer: {
                enabled: true,
                runs: 200
            }
        }
    },
    paths: {
        sources: "contracts",
        // deployments: ".deployments",
        tests: "test",
        cache: ".cache",
        artifacts: "artifacts"
    },
    mocha: {
        timeout: 60000
    },
    etherscan: {
        apiKey: ETHERSCAN_KEY
    },
    gasReporter: {
        enabled: process.env.REPORT_GAS ? true : false,
        currency: "USD",
        coinmarketcap: process.env.COINMARKETCAP_API_KEY
    },
    typechain: {
        outDir: "src",
        target: "ethers-v5",
        alwaysGenerateOverloads: false // should overloads with full signatures like deposit(uint256) be generated always, even if there are no overloads?
    }
}

export default config

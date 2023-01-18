import "solidity-coverage"
import "@nomicfoundation/hardhat-chai-matchers"
import "@nomiclabs/hardhat-ethers"
import "@nomiclabs/hardhat-etherscan"
import "@nomiclabs/hardhat-ethers"
import "@typechain/hardhat"
import "hardhat-gas-reporter"
import "hardhat-deploy"
import "hardhat-contract-sizer"

import "./tasks/contractAddress"

import "tsconfig-paths/register"

import type { HardhatUserConfig } from "hardhat/config"
import { ethers } from "ethers"

let INFURA_KEY = process.env.INFURA_PROJECT_ID
let ROPSTEN_KEY =
    process.env.ROPSTEN_PRIVATE_KEY ||
    "0x0000000000000000000000000000000000000000000000000000000000000000"
let ETHERSCAN_KEY = process.env.ETHERSCAN_API_KEY

subtask(TASK_COMPILE_SOLIDITY_GET_SOURCE_PATHS).setAction(async (_, __, runSuper) => {
    const paths = await runSuper()

    return paths.filter((p) => !p.endsWith(".t.sol"))
})

subtask(TASK_TEST_GET_TEST_FILES).setAction(async (_, __, runSuper) => {
    const files = await runSuper()
    return files.filter((file) => !file.includes("test/beefy/validator-set.ts"))
})

const config: HardhatUserConfig = {
    networks: {
        hardhat: {
            accounts: {
                mnemonic:
                    "stone speak what ritual switch pigeon weird dutch burst shaft nature shove",
                // Need to give huge account balance to test certain constraints in EthApp.sol::lock()
                accountsBalance: "350000000000000000000000000000000000000",
            },
            chainId: 15,
        },
        localhost: {
            url: "http://127.0.0.1:8545",
            accounts: {
                mnemonic:
                    "stone speak what ritual switch pigeon weird dutch burst shaft nature shove",
            },
            chainId: 15,
        },
        goerli: {
            chainId: 5,
            url: `https://goerli.infura.io/v3/${INFURA_KEY}`,
            accounts: [ROPSTEN_KEY],
            maxFeePerGas: ethers.utils.parseUnits("200", "gwei"),
            maxPriorityFeePerGas: ethers.utils.parseUnits("20", "gwei"),
        }
    },
    solidity: {
        version: "0.8.9",
        settings: {
            optimizer: {
                enabled: true,
                runs: 200,
            },
        },
    },
    paths: {
        sources: "contracts",
        // deployments: ".deployments",
        tests: "test",
        cache: ".cache",
        artifacts: "artifacts",
    },
    mocha: {
        timeout: 60000,
        // parallel: true,
        // jobs: 4,
    },
    etherscan: {
        apiKey: ETHERSCAN_KEY,
    },
    gasReporter: {
        enabled: process.env.REPORT_GAS ? true : false,
        currency: "USD",
        coinmarketcap: process.env.COINMARKETCAP_API_KEY,
    },
    typechain: {
        outDir: "src",
        target: "ethers-v5",
        alwaysGenerateOverloads: false, // should overloads with full signatures like deposit(uint256) be generated always, even if there are no overloads?
    },
    contractSizer: {
        alphaSort: true,
        runOnCompile: false,
        disambiguatePaths: false,
        except: ["Mock*"],
    },
}

export default config

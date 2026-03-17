import "dotenv/config"
import { createApi } from "@snowbridge/api"
import { polkadot_mainnet } from "@snowbridge/registry"
import { EthersEthereumProvider } from "@snowbridge/provider-ethers"
import { ViemEthereumProvider } from "@snowbridge/provider-viem"
import {
    eth1ToPolkadot1000Dot as eth1ToPolkadot1000DotEthers,
    eth1ToPolkadot2000Eth as eth1ToPolkadot2000EthEthers,
    eth1ToPolkadot2004Weth as eth1ToPolkadot2004WethEthers,
    eth1ToPolkadot2030Eth as eth1ToPolkadot2030EthEthers,
    eth1ToPolkadot2034Usdc as eth1ToPolkadot2034UsdcEthers,
    eth1ToPolkadot2043Trac as eth1ToPolkadot2043TracEthers,
    eth1ToPolkadot3369Myth as eth1ToPolkadot3369MythEthers,
    ethereum1284ToEth1Weth as ethereum1284ToEth1WethEthers,
    ethereumL210ToPolkadot1000Eth as ethereumL210ToPolkadot1000EthEthers,
    ethereumL242161ToPolkadot1000Weth as ethereumL242161ToPolkadot1000WethEthers,
    ethereumL28453ToPolkadot1000Usdc as ethereumL28453ToPolkadot1000UsdcEthers,
    polkadot1000ToEth1Dot as polkadot1000ToEth1DotEthers,
    polkadot1000ToEthereumL210Eth as polkadot1000ToEthereumL210EthEthers,
    polkadot1000ToEthereumL242161Weth as polkadot1000ToEthereumL242161WethEthers,
    polkadot1000ToEthereumL28453Usdc as polkadot1000ToEthereumL28453UsdcEthers,
    polkadot1000ToPolkadot2034Usdc as polkadot1000ToPolkadot2034UsdcEthers,
    polkadot2000ToEth1Eth as polkadot2000ToEth1EthEthers,
    polkadot2004ToEth1Weth as polkadot2004ToEth1WethEthers,
    polkadot2030ToEth1Eth as polkadot2030ToEth1EthEthers,
    polkadot2034ToEth1Usdc as polkadot2034ToEth1UsdcEthers,
    polkadot2034ToPolkadot1000Usdc as polkadot2034ToPolkadot1000UsdcEthers,
    polkadot2043ToEth1Trac as polkadot2043ToEth1TracEthers,
    polkadot3369ToEth1Myth as polkadot3369ToEth1MythEthers,
    createAgent as createAgentEthers,
    registerToken as registerTokenEthers,
} from "./ethersCases"
import {
    eth1ToPolkadot1000Dot as eth1ToPolkadot1000DotViem,
    eth1ToPolkadot2000Eth as eth1ToPolkadot2000EthViem,
    eth1ToPolkadot2004Weth as eth1ToPolkadot2004WethViem,
    eth1ToPolkadot2030Eth as eth1ToPolkadot2030EthViem,
    eth1ToPolkadot2034Usdc as eth1ToPolkadot2034UsdcViem,
    eth1ToPolkadot2043Trac as eth1ToPolkadot2043TracViem,
    eth1ToPolkadot3369Myth as eth1ToPolkadot3369MythViem,
    ethereum1284ToEth1Weth as ethereum1284ToEth1WethViem,
    ethereumL210ToPolkadot1000Eth as ethereumL210ToPolkadot1000EthViem,
    ethereumL242161ToPolkadot1000Weth as ethereumL242161ToPolkadot1000WethViem,
    ethereumL28453ToPolkadot1000Usdc as ethereumL28453ToPolkadot1000UsdcViem,
    polkadot1000ToEth1Dot as polkadot1000ToEth1DotViem,
    polkadot1000ToEthereumL210Eth as polkadot1000ToEthereumL210EthViem,
    polkadot1000ToEthereumL242161Weth as polkadot1000ToEthereumL242161WethViem,
    polkadot1000ToEthereumL28453Usdc as polkadot1000ToEthereumL28453UsdcViem,
    polkadot1000ToPolkadot2034Usdc as polkadot1000ToPolkadot2034UsdcViem,
    polkadot2000ToEth1Eth as polkadot2000ToEth1EthViem,
    polkadot2004ToEth1Weth as polkadot2004ToEth1WethViem,
    polkadot2030ToEth1Eth as polkadot2030ToEth1EthViem,
    polkadot2034ToEth1Usdc as polkadot2034ToEth1UsdcViem,
    polkadot2034ToPolkadot1000Usdc as polkadot2034ToPolkadot1000UsdcViem,
    polkadot2043ToEth1Trac as polkadot2043ToEth1TracViem,
    polkadot3369ToEth1Myth as polkadot3369ToEth1MythViem,
    createAgent as createAgentViem,
    registerToken as registerTokenViem,
} from "./viemCases"

async function main() {
    const ethersApi = createApi({
        info: polkadot_mainnet,
        ethereumProvider: new EthersEthereumProvider(),
    })

    try {
        await eth1ToPolkadot1000DotEthers(ethersApi)
        await eth1ToPolkadot2000EthEthers(ethersApi)
        await eth1ToPolkadot2004WethEthers(ethersApi)
        await eth1ToPolkadot2030EthEthers(ethersApi)
        await eth1ToPolkadot2034UsdcEthers(ethersApi)
        await eth1ToPolkadot2043TracEthers(ethersApi)
        await eth1ToPolkadot3369MythEthers(ethersApi)
        await ethereum1284ToEth1WethEthers(ethersApi)
        await ethereumL210ToPolkadot1000EthEthers(ethersApi)
        await ethereumL242161ToPolkadot1000WethEthers(ethersApi)
        await ethereumL28453ToPolkadot1000UsdcEthers(ethersApi)
        await polkadot1000ToEth1DotEthers(ethersApi)
        await polkadot1000ToEthereumL210EthEthers(ethersApi)
        await polkadot1000ToEthereumL242161WethEthers(ethersApi)
        await polkadot1000ToEthereumL28453UsdcEthers(ethersApi)
        await polkadot1000ToPolkadot2034UsdcEthers(ethersApi)
        await polkadot2000ToEth1EthEthers(ethersApi)
        await polkadot2004ToEth1WethEthers(ethersApi)
        await polkadot2030ToEth1EthEthers(ethersApi)
        await polkadot2034ToEth1UsdcEthers(ethersApi)
        await polkadot2034ToPolkadot1000UsdcEthers(ethersApi)
        await polkadot2043ToEth1TracEthers(ethersApi)
        await polkadot3369ToEth1MythEthers(ethersApi)
        await createAgentEthers(ethersApi)
        await registerTokenEthers(ethersApi)
    } finally {
        await ethersApi.destroy()
    }

    const viemApi = createApi({
        info: polkadot_mainnet,
        ethereumProvider: new ViemEthereumProvider(),
    })

    try {
        await eth1ToPolkadot1000DotViem(viemApi)
        await eth1ToPolkadot2000EthViem(viemApi)
        await eth1ToPolkadot2004WethViem(viemApi)
        await eth1ToPolkadot2030EthViem(viemApi)
        await eth1ToPolkadot2034UsdcViem(viemApi)
        await eth1ToPolkadot2043TracViem(viemApi)
        await eth1ToPolkadot3369MythViem(viemApi)
        await ethereum1284ToEth1WethViem(viemApi)
        await ethereumL210ToPolkadot1000EthViem(viemApi)
        await ethereumL242161ToPolkadot1000WethViem(viemApi)
        await ethereumL28453ToPolkadot1000UsdcViem(viemApi)
        await polkadot1000ToEth1DotViem(viemApi)
        await polkadot1000ToEthereumL210EthViem(viemApi)
        await polkadot1000ToEthereumL242161WethViem(viemApi)
        await polkadot1000ToEthereumL28453UsdcViem(viemApi)
        await polkadot1000ToPolkadot2034UsdcViem(viemApi)
        await polkadot2000ToEth1EthViem(viemApi)
        await polkadot2004ToEth1WethViem(viemApi)
        await polkadot2030ToEth1EthViem(viemApi)
        await polkadot2034ToEth1UsdcViem(viemApi)
        await polkadot2034ToPolkadot1000UsdcViem(viemApi)
        await polkadot2043ToEth1TracViem(viemApi)
        await polkadot3369ToEth1MythViem(viemApi)
        await createAgentViem(viemApi)
        await registerTokenViem(viemApi)
    } finally {
        await viemApi.destroy()
    }
}

main().catch((error) => {
    console.error(error)
    if (error && typeof error === "object" && "validation" in error) {
        const validation = (error as { validation?: { logs?: unknown } }).validation
        if (validation?.logs !== undefined) {
            console.dir(validation.logs, { depth: 100 })
        }
    }
    process.exit(1)
})

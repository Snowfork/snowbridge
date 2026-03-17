import type { SnowbridgeApi } from "@snowbridge/api"
import type { ViemEthereumProvider } from "@snowbridge/provider-viem"
import { polkadot_mainnet } from "@snowbridge/registry"

export async function eth1ToPolkadot1000Dot(api: SnowbridgeApi<ViemEthereumProvider>) {
    const sourceAccount = process.env.ETHEREUM_ACCOUNT_PUBLIC
    const beneficiaryAccount = process.env.SUBSTRATE_ACCOUNT_PUBLIC
    if (!sourceAccount) throw new Error("Missing required env var for source account.")
    if (!beneficiaryAccount) throw new Error("Missing required env var for beneficiary account.")

    const {
        chains: { ethereum, assetHub },
    } = polkadot_mainnet

    const sender = api.sender(ethereum, assetHub)
    const transfer = await sender.build(
        sourceAccount,
        beneficiaryAccount,
        "0x196c20da81fbc324ecdf55501e95ce9f0bd84d14",
        100000000n,
    )

    console.log(
        JSON.stringify({
            route: "ethereum:1 -> polkadot:1000",
            token: "DOT",
            txHex: transfer.tx.data,
        }),
    )
}

export async function eth1ToPolkadot2000Eth(api: SnowbridgeApi<ViemEthereumProvider>) {
    const sourceAccount = process.env.ETHEREUM_ACCOUNT_PUBLIC
    const beneficiaryAccount = process.env.SUBSTRATE_ACCOUNT_PUBLIC
    if (!sourceAccount) throw new Error("Missing required env var for source account.")
    if (!beneficiaryAccount) throw new Error("Missing required env var for beneficiary account.")

    const {
        chains: { ethereum, acala },
    } = polkadot_mainnet

    const sender = api.sender(ethereum, acala)
    const transfer = await sender.build(
        sourceAccount,
        beneficiaryAccount,
        "0x0000000000000000000000000000000000000000",
        15000000000000n,
    )

    console.log(
        JSON.stringify({
            route: "ethereum:1 -> polkadot:2000",
            token: "ETH",
            txHex: transfer.tx.data,
        }),
    )
}

export async function eth1ToPolkadot2004Weth(api: SnowbridgeApi<ViemEthereumProvider>) {
    const sourceAccount = process.env.ETHEREUM_ACCOUNT_PUBLIC
    const beneficiaryAccount = process.env.ETHEREUM_ACCOUNT_PUBLIC
    if (!sourceAccount) throw new Error("Missing required env var for source account.")
    if (!beneficiaryAccount) throw new Error("Missing required env var for beneficiary account.")

    const {
        chains: { ethereum, moonbeamSubstrate },
    } = polkadot_mainnet

    const sender = api.sender(ethereum, moonbeamSubstrate)
    const transfer = await sender.build(
        sourceAccount,
        beneficiaryAccount,
        "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2",
        15000000000000n,
    )

    console.log(
        JSON.stringify({
            route: "ethereum:1 -> polkadot:2004",
            token: "WETH",
            txHex: transfer.tx.data,
        }),
    )
}

export async function eth1ToPolkadot2030Eth(api: SnowbridgeApi<ViemEthereumProvider>) {
    const sourceAccount = process.env.ETHEREUM_ACCOUNT_PUBLIC
    const beneficiaryAccount = process.env.SUBSTRATE_ACCOUNT_PUBLIC
    if (!sourceAccount) throw new Error("Missing required env var for source account.")
    if (!beneficiaryAccount) throw new Error("Missing required env var for beneficiary account.")

    const {
        chains: { ethereum, bifrostPolkadot },
    } = polkadot_mainnet

    const sender = api.sender(ethereum, bifrostPolkadot)
    const transfer = await sender.build(
        sourceAccount,
        beneficiaryAccount,
        "0x0000000000000000000000000000000000000000",
        15000000000000n,
    )

    console.log(
        JSON.stringify({
            route: "ethereum:1 -> polkadot:2030",
            token: "ETH",
            txHex: transfer.tx.data,
        }),
    )
}

export async function eth1ToPolkadot2034Usdc(api: SnowbridgeApi<ViemEthereumProvider>) {
    const sourceAccount = process.env.ETHEREUM_ACCOUNT_PUBLIC
    const beneficiaryAccount = process.env.SUBSTRATE_ACCOUNT_PUBLIC
    if (!sourceAccount) throw new Error("Missing required env var for source account.")
    if (!beneficiaryAccount) throw new Error("Missing required env var for beneficiary account.")

    const {
        chains: { ethereum, hydration },
    } = polkadot_mainnet

    const sender = api.sender(ethereum, hydration)
    const transfer = await sender.build(
        sourceAccount,
        beneficiaryAccount,
        "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
        10000n,
    )

    console.log(
        JSON.stringify({
            route: "ethereum:1 -> polkadot:2034",
            token: "USDC",
            txHex: transfer.tx.data,
        }),
    )
}

export async function eth1ToPolkadot2043Trac(api: SnowbridgeApi<ViemEthereumProvider>) {
    const tokenAddress = "0xaa7a9ca87d3694b5755f213b5d04094b8d0f0a6f"
    const sourceAccount = process.env.ETHEREUM_ACCOUNT_PUBLIC
    const beneficiaryAccount = process.env.SUBSTRATE_ACCOUNT_PUBLIC
    if (!sourceAccount) throw new Error("Missing required env var for source account.")
    if (!beneficiaryAccount) throw new Error("Missing required env var for beneficiary account.")

    const {
        chains: { ethereum, neuroWeb },
    } = polkadot_mainnet
    const amount =
        polkadot_mainnet.registry.parachains[neuroWeb.key].assets[tokenAddress].minimumBalance

    const sender = api.sender(ethereum, neuroWeb)
    const transfer = await sender.build(sourceAccount, beneficiaryAccount, tokenAddress, amount)

    console.log(
        JSON.stringify({
            route: "ethereum:1 -> polkadot:2043",
            token: "TRAC",
            txHex: transfer.tx.data,
        }),
    )
}

export async function eth1ToPolkadot3369Myth(api: SnowbridgeApi<ViemEthereumProvider>) {
    const tokenAddress = "0xba41ddf06b7ffd89d1267b5a93bfef2424eb2003"
    const sourceAccount = process.env.ETHEREUM_ACCOUNT_PUBLIC
    const beneficiaryAccount = process.env.ETHEREUM_ACCOUNT_PUBLIC
    if (!sourceAccount) throw new Error("Missing required env var for source account.")
    if (!beneficiaryAccount) throw new Error("Missing required env var for beneficiary account.")

    const {
        chains: { ethereum, mythos },
    } = polkadot_mainnet
    const amount =
        polkadot_mainnet.registry.parachains[mythos.key].assets[tokenAddress].minimumBalance

    const sender = api.sender(ethereum, mythos)
    const transfer = await sender.build(sourceAccount, beneficiaryAccount, tokenAddress, amount)

    console.log(
        JSON.stringify({
            route: "ethereum:1 -> polkadot:3369",
            token: "MYTH",
            txHex: transfer.tx.data,
        }),
    )
}

export async function ethereum1284ToEth1Weth(api: SnowbridgeApi<ViemEthereumProvider>) {
    const sourceAccount = process.env.ETHEREUM_ACCOUNT_PUBLIC
    const beneficiaryAccount = process.env.ETHEREUM_ACCOUNT_PUBLIC
    if (!sourceAccount) throw new Error("Missing required env var for source account.")
    if (!beneficiaryAccount) throw new Error("Missing required env var for beneficiary account.")

    const {
        chains: { moonbeam, ethereum },
    } = polkadot_mainnet

    const sender = api.sender(moonbeam, ethereum)
    const transfer = await sender.build(
        sourceAccount,
        beneficiaryAccount,
        "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2",
        1n,
    )

    console.log(
        JSON.stringify({
            route: "ethereum:1284 -> ethereum:1",
            token: "WETH",
            txHex: transfer.tx.data,
        }),
    )
}

export async function ethereumL210ToPolkadot1000Eth(api: SnowbridgeApi<ViemEthereumProvider>) {
    const sourceAccount = process.env.ETHEREUM_ACCOUNT_PUBLIC
    const beneficiaryAccount = process.env.SUBSTRATE_ACCOUNT_PUBLIC
    if (!sourceAccount) throw new Error("Missing required env var for source account.")
    if (!beneficiaryAccount) throw new Error("Missing required env var for beneficiary account.")

    const {
        chains: { optimism, assetHub },
    } = polkadot_mainnet
    const tokenAddress =
        polkadot_mainnet.registry.ethereumChains.ethereum_l2_10.assets[
            "0x0000000000000000000000000000000000000000"
        ].token

    const sender = api.sender(optimism, assetHub)
    const transfer = await sender.build(
        tokenAddress,
        400000000000000n,
        sourceAccount,
        beneficiaryAccount,
    )

    console.log(
        JSON.stringify({
            route: "ethereum_l2:10 -> polkadot:1000",
            token: "ETH",
            txHex: transfer.tx.data,
        }),
    )
}

export async function ethereumL242161ToPolkadot1000Weth(api: SnowbridgeApi<ViemEthereumProvider>) {
    const sourceAccount = process.env.ETHEREUM_ACCOUNT_PUBLIC
    const beneficiaryAccount = process.env.SUBSTRATE_ACCOUNT_PUBLIC
    if (!sourceAccount) throw new Error("Missing required env var for source account.")
    if (!beneficiaryAccount) throw new Error("Missing required env var for beneficiary account.")

    const {
        chains: { arbitrum, assetHub },
    } = polkadot_mainnet
    const tokenAddress =
        polkadot_mainnet.registry.ethereumChains.ethereum_l2_42161.assets[
            "0x82af49447d8a07e3bd95bd0d56f35241523fbab1"
        ].token

    const sender = api.sender(arbitrum, assetHub)
    const transfer = await sender.build(
        tokenAddress,
        400000000000000n,
        sourceAccount,
        beneficiaryAccount,
    )

    console.log(
        JSON.stringify({
            route: "ethereum_l2:42161 -> polkadot:1000",
            token: "WETH",
            txHex: transfer.tx.data,
        }),
    )
}

export async function ethereumL28453ToPolkadot1000Usdc(api: SnowbridgeApi<ViemEthereumProvider>) {
    const sourceAccount = process.env.ETHEREUM_ACCOUNT_PUBLIC
    const beneficiaryAccount = process.env.SUBSTRATE_ACCOUNT_PUBLIC
    if (!sourceAccount) throw new Error("Missing required env var for source account.")
    if (!beneficiaryAccount) throw new Error("Missing required env var for beneficiary account.")

    const {
        chains: { base, assetHub },
    } = polkadot_mainnet
    const tokenAddress =
        polkadot_mainnet.registry.ethereumChains.ethereum_l2_8453.assets[
            "0x833589fcd6edb6e08f4c7c32d4f71b54bda02913"
        ].token

    const sender = api.sender(base, assetHub)
    const transfer = await sender.build(tokenAddress, 1000000n, sourceAccount, beneficiaryAccount)

    console.log(
        JSON.stringify({
            route: "ethereum_l2:8453 -> polkadot:1000",
            token: "USDC",
            txHex: transfer.tx.data,
        }),
    )
}

export async function polkadot1000ToEth1Dot(api: SnowbridgeApi<ViemEthereumProvider>) {
    const sourceAccount = process.env.SUBSTRATE_ACCOUNT_PUBLIC
    const beneficiaryAccount = process.env.ETHEREUM_ACCOUNT_PUBLIC
    if (!sourceAccount) throw new Error("Missing required env var for source account.")
    if (!beneficiaryAccount) throw new Error("Missing required env var for beneficiary account.")

    const {
        chains: { assetHub, ethereum },
    } = polkadot_mainnet

    const sender = api.sender(assetHub, ethereum)
    const transfer = await sender.build(
        sourceAccount,
        beneficiaryAccount,
        "0x196c20da81fbc324ecdf55501e95ce9f0bd84d14",
        1n,
    )

    console.log(
        JSON.stringify({
            route: "polkadot:1000 -> ethereum:1",
            token: "DOT",
            txHex: transfer.tx.toHex(),
        }),
    )
}

export async function polkadot1000ToEthereumL210Eth(api: SnowbridgeApi<ViemEthereumProvider>) {
    const sourceAccount = process.env.SUBSTRATE_ACCOUNT_PUBLIC
    const beneficiaryAccount = process.env.ETHEREUM_ACCOUNT_PUBLIC
    if (!sourceAccount) throw new Error("Missing required env var for source account.")
    if (!beneficiaryAccount) throw new Error("Missing required env var for beneficiary account.")

    const {
        chains: { assetHub, optimism },
    } = polkadot_mainnet

    const sender = api.sender(assetHub, optimism)
    const transfer = await sender.build(
        "0x0000000000000000000000000000000000000000",
        400000000000000n,
        sourceAccount,
        beneficiaryAccount,
    )

    console.log(
        JSON.stringify({
            route: "polkadot:1000 -> ethereum_l2:10",
            token: "ETH",
            txHex: transfer.tx.toHex(),
        }),
    )
}

export async function polkadot1000ToEthereumL242161Weth(api: SnowbridgeApi<ViemEthereumProvider>) {
    const sourceAccount = process.env.SUBSTRATE_ACCOUNT_PUBLIC
    const beneficiaryAccount = process.env.ETHEREUM_ACCOUNT_PUBLIC
    if (!sourceAccount) throw new Error("Missing required env var for source account.")
    if (!beneficiaryAccount) throw new Error("Missing required env var for beneficiary account.")

    const {
        chains: { assetHub, arbitrum },
    } = polkadot_mainnet

    const sender = api.sender(assetHub, arbitrum)
    const transfer = await sender.build(
        "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2",
        400000000000000n,
        sourceAccount,
        beneficiaryAccount,
    )

    console.log(
        JSON.stringify({
            route: "polkadot:1000 -> ethereum_l2:42161",
            token: "WETH",
            txHex: transfer.tx.toHex(),
        }),
    )
}

export async function polkadot1000ToEthereumL28453Usdc(api: SnowbridgeApi<ViemEthereumProvider>) {
    const sourceAccount = process.env.SUBSTRATE_ACCOUNT_PUBLIC
    const beneficiaryAccount = process.env.ETHEREUM_ACCOUNT_PUBLIC
    if (!sourceAccount) throw new Error("Missing required env var for source account.")
    if (!beneficiaryAccount) throw new Error("Missing required env var for beneficiary account.")

    const {
        chains: { assetHub, base },
    } = polkadot_mainnet

    const sender = api.sender(assetHub, base)
    const transfer = await sender.build(
        "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
        1000000n,
        sourceAccount,
        beneficiaryAccount,
    )

    console.log(
        JSON.stringify({
            route: "polkadot:1000 -> ethereum_l2:8453",
            token: "USDC",
            txHex: transfer.tx.toHex(),
        }),
    )
}

export async function polkadot1000ToPolkadot2034Usdc(api: SnowbridgeApi<ViemEthereumProvider>) {
    const sourceAccount = process.env.SUBSTRATE_ACCOUNT_PUBLIC
    const beneficiaryAccount = process.env.SUBSTRATE_ACCOUNT_PUBLIC
    if (!sourceAccount) throw new Error("Missing required env var for source account.")
    if (!beneficiaryAccount) throw new Error("Missing required env var for beneficiary account.")

    const {
        chains: { assetHub, hydration },
    } = polkadot_mainnet

    const sender = api.sender(assetHub, hydration)
    const transfer = await sender.build(
        sourceAccount,
        beneficiaryAccount,
        "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
        10000n,
    )

    console.log(
        JSON.stringify({
            route: "polkadot:1000 -> polkadot:2034",
            token: "USDC",
            txHex: transfer.tx.toHex(),
        }),
    )
}

export async function polkadot2000ToEth1Eth(api: SnowbridgeApi<ViemEthereumProvider>) {
    const sourceAccount = process.env.SUBSTRATE_ACCOUNT_PUBLIC
    const beneficiaryAccount = process.env.ETHEREUM_ACCOUNT_PUBLIC
    if (!sourceAccount) throw new Error("Missing required env var for source account.")
    if (!beneficiaryAccount) throw new Error("Missing required env var for beneficiary account.")

    const {
        chains: { acala, ethereum },
    } = polkadot_mainnet

    const sender = api.sender(acala, ethereum)
    const transfer = await sender.build(
        sourceAccount,
        beneficiaryAccount,
        "0x0000000000000000000000000000000000000000",
        1n,
    )

    console.log(
        JSON.stringify({
            route: "polkadot:2000 -> ethereum:1",
            token: "ETH",
            txHex: transfer.tx.toHex(),
        }),
    )
}

export async function polkadot2004ToEth1Weth(api: SnowbridgeApi<ViemEthereumProvider>) {
    const sourceAccount = process.env.ETHEREUM_ACCOUNT_PUBLIC
    const beneficiaryAccount = process.env.ETHEREUM_ACCOUNT_PUBLIC
    if (!sourceAccount) throw new Error("Missing required env var for source account.")
    if (!beneficiaryAccount) throw new Error("Missing required env var for beneficiary account.")

    const {
        chains: { moonbeamSubstrate, ethereum },
    } = polkadot_mainnet

    const sender = api.sender(moonbeamSubstrate, ethereum)
    const transfer = await sender.build(
        sourceAccount,
        beneficiaryAccount,
        "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2",
        1n,
    )

    console.log(
        JSON.stringify({
            route: "polkadot:2004 -> ethereum:1",
            token: "WETH",
            txHex: transfer.tx.toHex(),
        }),
    )
}

export async function polkadot2030ToEth1Eth(api: SnowbridgeApi<ViemEthereumProvider>) {
    const sourceAccount = process.env.SUBSTRATE_ACCOUNT_PUBLIC
    const beneficiaryAccount = process.env.ETHEREUM_ACCOUNT_PUBLIC
    if (!sourceAccount) throw new Error("Missing required env var for source account.")
    if (!beneficiaryAccount) throw new Error("Missing required env var for beneficiary account.")

    const {
        chains: { bifrostPolkadot, ethereum },
    } = polkadot_mainnet

    const sender = api.sender(bifrostPolkadot, ethereum)
    const transfer = await sender.build(
        sourceAccount,
        beneficiaryAccount,
        "0x0000000000000000000000000000000000000000",
        1n,
    )

    console.log(
        JSON.stringify({
            route: "polkadot:2030 -> ethereum:1",
            token: "ETH",
            txHex: transfer.tx.toHex(),
        }),
    )
}

export async function polkadot2034ToEth1Usdc(api: SnowbridgeApi<ViemEthereumProvider>) {
    const sourceAccount = process.env.SUBSTRATE_ACCOUNT_PUBLIC
    const beneficiaryAccount = process.env.ETHEREUM_ACCOUNT_PUBLIC
    if (!sourceAccount) throw new Error("Missing required env var for source account.")
    if (!beneficiaryAccount) throw new Error("Missing required env var for beneficiary account.")

    const {
        chains: { hydration, ethereum },
    } = polkadot_mainnet

    const sender = api.sender(hydration, ethereum)
    const transfer = await sender.build(
        sourceAccount,
        beneficiaryAccount,
        "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
        1n,
    )

    console.log(
        JSON.stringify({
            route: "polkadot:2034 -> ethereum:1",
            token: "USDC",
            txHex: transfer.tx.toHex(),
        }),
    )
}

export async function polkadot2034ToPolkadot1000Usdc(api: SnowbridgeApi<ViemEthereumProvider>) {
    const sourceAccount = process.env.SUBSTRATE_ACCOUNT_PUBLIC
    const beneficiaryAccount = process.env.SUBSTRATE_ACCOUNT_PUBLIC
    if (!sourceAccount) throw new Error("Missing required env var for source account.")
    if (!beneficiaryAccount) throw new Error("Missing required env var for beneficiary account.")

    const {
        chains: { hydration, assetHub },
    } = polkadot_mainnet

    const sender = api.sender(hydration, assetHub)
    const transfer = await sender.build(
        sourceAccount,
        beneficiaryAccount,
        "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
        10000n,
    )

    console.log(
        JSON.stringify({
            route: "polkadot:2034 -> polkadot:1000",
            token: "USDC",
            txHex: transfer.tx.toHex(),
        }),
    )
}

export async function polkadot2043ToEth1Trac(api: SnowbridgeApi<ViemEthereumProvider>) {
    const sourceAccount = process.env.SUBSTRATE_ACCOUNT_PUBLIC
    const beneficiaryAccount = process.env.ETHEREUM_ACCOUNT_PUBLIC
    if (!sourceAccount) throw new Error("Missing required env var for source account.")
    if (!beneficiaryAccount) throw new Error("Missing required env var for beneficiary account.")

    const {
        chains: { neuroWeb, ethereum },
    } = polkadot_mainnet

    const sender = api.sender(neuroWeb, ethereum)
    const transfer = await sender.build(
        sourceAccount,
        beneficiaryAccount,
        "0xaa7a9ca87d3694b5755f213b5d04094b8d0f0a6f",
        1n,
    )

    console.log(
        JSON.stringify({
            route: "polkadot:2043 -> ethereum:1",
            token: "TRAC",
            txHex: transfer.tx.toHex(),
        }),
    )
}

export async function polkadot3369ToEth1Myth(api: SnowbridgeApi<ViemEthereumProvider>) {
    const sourceAccount = process.env.ETHEREUM_ACCOUNT_PUBLIC
    const beneficiaryAccount = process.env.ETHEREUM_ACCOUNT_PUBLIC
    if (!sourceAccount) throw new Error("Missing required env var for source account.")
    if (!beneficiaryAccount) throw new Error("Missing required env var for beneficiary account.")

    const {
        chains: { mythos, ethereum },
    } = polkadot_mainnet

    const sender = api.sender(mythos, ethereum)
    const transfer = await sender.build(
        sourceAccount,
        beneficiaryAccount,
        "0xba41ddf06b7ffd89d1267b5a93bfef2424eb2003",
        1n,
    )

    console.log(
        JSON.stringify({
            route: "polkadot:3369 -> ethereum:1",
            token: "MYTH",
            txHex: transfer.tx.toHex(),
        }),
    )
}

export async function createAgent(api: SnowbridgeApi<ViemEthereumProvider>) {
    const sourceAccount = process.env.ETHEREUM_ACCOUNT_PUBLIC
    const agentId = process.env.TEST_AGENT_ID
    if (!sourceAccount) throw new Error("Missing required env var for source account.")
    if (!agentId) throw new Error("Missing required env var TEST_AGENT_ID.")

    const creationImpl = api.createAgent()
    const creation = await creationImpl.tx(sourceAccount, agentId)
    const validation = await creationImpl.validate(creation)

    if (!validation.success) {
        throw new Error(`CreateAgent validation failed: ${JSON.stringify(validation.logs)}`)
    }

    console.log(
        JSON.stringify({
            route: "ethereum:1 -> createAgent",
            agentId,
            txHex: creation.tx.data,
            estimatedGas: validation.data.feeInfo?.estimatedGas?.toString(),
        }),
    )
}

export async function registerToken(api: SnowbridgeApi<ViemEthereumProvider>) {
    const sourceAccount = process.env.ETHEREUM_ACCOUNT_PUBLIC
    const tokenAddress = process.env.TEST_REGISTER_TOKEN_ADDRESS
    if (!sourceAccount) throw new Error("Missing required env var for source account.")
    if (!tokenAddress) throw new Error("Missing required env var TEST_REGISTER_TOKEN_ADDRESS.")

    const relayerFee = 100_000_000_000_000n
    const registrationImpl = api.registerToken()
    const fee = await registrationImpl.fee(relayerFee)
    const registration = await registrationImpl.tx(sourceAccount, tokenAddress.toLowerCase(), fee)
    const validation = await registrationImpl.validate(registration)

    if (!validation.success) {
        throw new Error(`RegisterToken validation failed: ${JSON.stringify(validation.logs)}`)
    }

    console.log(
        JSON.stringify({
            route: "ethereum:1 -> registerToken",
            token: tokenAddress.toLowerCase(),
            txHex: registration.tx.data,
            estimatedGas: validation.data.feeInfo?.estimatedGas?.toString(),
        }),
    )
}

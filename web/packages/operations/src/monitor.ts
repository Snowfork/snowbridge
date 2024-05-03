
import { u8aToHex } from '@polkadot/util'
import { blake2AsU8a } from "@polkadot/util-crypto"
import { contextFactory, destroyContext, environment, status, utils } from '@snowbridge/api'

const monitor = async () => {
    let env = 'local_e2e'
    if (process.env.NODE_ENV !== undefined) {
        env = process.env.NODE_ENV
    }
    const snwobridgeEnv = environment.SNOWBRIDGE_ENV[env]
    if (snwobridgeEnv === undefined) { throw Error(`Unknown environment '${env}'`) }

    const { config } = snwobridgeEnv

    const infuraKey = process.env.REACT_APP_INFURA_KEY || ''

    const context = await contextFactory({
        ethereum: { execution_url: config.ETHEREUM_WS_API(infuraKey), beacon_url: config.BEACON_HTTP_API },
        polkadot: {
            url: {
                bridgeHub: config.BRIDGE_HUB_WS_URL,
                assetHub: config.ASSET_HUB_WS_URL,
                relaychain: config.RELAY_CHAIN_WS_URL,
            },
        },
        appContracts: {
            gateway: config.GATEWAY_CONTRACT,
            beefy: config.BEEFY_CONTRACT,
        },
    })

    const bridegStatus = await status.bridgeStatusInfo(context)
    console.log('Bridge Status:', bridegStatus)
    const assethub = await status.channelStatusInfo(context, utils.paraIdToChannelId(config.ASSET_HUB_PARAID))
    console.log('Asset Hub Channel:', assethub)
    const primaryGov = await status.channelStatusInfo(context, config.PRIMARY_GOVERNANCE_CHANNEL_ID)
    console.log('Primary Governance Channel:', primaryGov)
    const secondaryGov = await status.channelStatusInfo(context, config.SECONDARY_GOVERNANCE_CHANNEL_ID)
    console.log('Secondary Governance Channel:', secondaryGov)

    let assetHubSovereign = BigInt(((await context.polkadot.api.bridgeHub.query.system.account(utils.paraIdToSovereignAccount("sibl", config.ASSET_HUB_PARAID))).toPrimitive() as any).data.free)
    console.log('Asset Hub Sovereign balance on bridgehub:', assetHubSovereign)

    let assetHubAgentBalance = (await context.ethereum.api.getBalance(
        await context.ethereum.contracts.gateway.agentOf(
            utils.paraIdToAgentId(context.polkadot.api.bridgeHub.registry, config.ASSET_HUB_PARAID)
        )
    ))
    console.log('Asset Hub Agent balance:', assetHubAgentBalance)

    const bridgeHubAgentId = u8aToHex(blake2AsU8a("0x00", 256))
    let bridgeHubAgentBalance = (await context.ethereum.api.getBalance(
        await context.ethereum.contracts.gateway.agentOf(bridgeHubAgentId)
    ))
    console.log('Bridge Hub Agent balance:', bridgeHubAgentBalance)

    console.log('Relayers:')
    for (const relayer of config.RELAYERS) {
        let balance = 0n
        switch (relayer.type) {
            case "ethereum":
                balance = await context.ethereum.api.getBalance(relayer.account)
                break
            case "substrate":
                balance = BigInt(((await context.polkadot.api.bridgeHub.query.system.account(relayer.account)).toPrimitive() as any).data.free)
                break
        }
        console.log('\t', balance, ':', relayer.type, 'balance ->', relayer.name)
    }

    await destroyContext(context)
}


monitor()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error(error)
        process.exit(1)
    })

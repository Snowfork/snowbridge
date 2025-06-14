import { ApiPromise } from "@polkadot/api"
import { ParachainBase } from "./parachainBase"
import { HydrationParachain } from "./hydration"
import { AssetHubParachain } from "./assethub"
import { BifrostParachain } from "./bifrost"
import { MoonbeamParachain } from "./moonbeam"
import { MythosParachain } from "./mythos"
import { GenericChain } from "./generic"
import { AssetHubKusamaParachain } from "./assethubKusama"
import { AcalaParachain } from "./acala"
import { PenpalParachain } from "./penpal"

export async function paraImplementation(provider: ApiPromise): Promise<ParachainBase> {
    let parachainId = 0
    if (provider.query.parachainInfo) {
        const encoded = await provider.query.parachainInfo.parachainId()
        parachainId = Number(encoded.toPrimitive())
    }
    const { specName, specVersion } = provider.consts.system.version.toJSON() as any
    switch (specName) {
        case "acala":
            return new AcalaParachain(provider, parachainId, specName, specVersion)
        case "basilisk":
        case "hydradx":
            return new HydrationParachain(provider, parachainId, specName, specVersion)
        case "penpal-parachain":
            return new PenpalParachain(provider, parachainId, specName, specVersion)
        case "asset-hub-paseo":
        case "westmint":
        case "statemint":
            return new AssetHubParachain(provider, parachainId, specName, specVersion)
        case "statemine":
            return new AssetHubKusamaParachain(provider, parachainId, specName, specVersion)
        case "bifrost":
        case "bifrost_paseo":
        case "bifrost_polkadot":
            return new BifrostParachain(provider, parachainId, specName, specVersion)
        case "moonriver":
        case "moonbeam":
            return new MoonbeamParachain(provider, parachainId, specName, specVersion)
        case "muse":
        case "mythos":
            return new MythosParachain(provider, parachainId, specName, specVersion)
        case "westend":
        case "paseo":
        case "polkadot":
        case "bridge-hub-paseo":
        case "bridge-hub-westend":
        case "bridge-hub-polkadot":
            return new GenericChain(provider, parachainId, specName, specVersion)
        default:
            throw Error(
                `No parachain provider for ParaId = ${parachainId}, Spec = ${specName}, Version = ${specVersion}`
            )
    }
}

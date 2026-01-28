import { Context } from "@snowbridge/api"
import { BridgeInfo, ChainId, ChainKind } from "@snowbridge/base-types"
import { polkadot_mainnet } from "@snowbridge/registry"


interface A { f1(): void }
interface B { f2(): void }
interface C { f3(): void }
interface D { f4(): void }
interface E { f5(): void }
interface F { f6(): void }

type Implementations =
  | { from: "ethereum"; to: "polkadot"; sender: A }
  | { from: "ethereum"; to: "kusama"; sender: F }
  | { from: "polkadot"; to: "ethereum"; sender: B }
  | { from: "polkadot"; to: "kusama"; sender: C }
  | { from: "polkadot"; to: "ethereum_l2"; sender: D }
  | { from: "ethereum_l2"; to: "polkadot"; sender: E }
  | { from: "kusama"; to: "polkadot"; sender: F }

type Implementations2 =
  | { kind: "ethereum->polkadot" } & A
  | { kind: "ethereum->kusama" } & F
  | { kind: "polkadot->ethereum" } & B
  | { kind: "polkadot->kusama" } & C
  | { kind: "polkadot->ethereum_l2" } & D
  | { kind: "ethereum_l2->polkadot" } & E
  | { kind: "kusama->polkadot" } & F

type IsUnion<T, U = T> = T extends any ? ([U] extends [T] ? false : true) : never
type NoUnion<T> = IsUnion<T> extends true ? never : T

type SnowbridgeApiParams = {
    bridge: BridgeInfo
}
class SnowbridgeApi {
    #bridge: BridgeInfo
    constructor(params: SnowbridgeApiParams) {
        this.#bridge = params.bridge
    }

    transfer<F extends Implementations["from"], T extends Extract<Implementations, { from: F }>["to"]>(
        from: { kind: NoUnion<F>; id: ChainId["id"] },
        to: { kind: T; id: ChainId["id"] },
    ): Extract<Implementations, { from: F; to: T }>["sender"]
    transfer(from: ChainId, to: ChainId) {
        return this.transferUntyped(from, to)
    }
    transferUntyped(
        from: ChainId,
        to: ChainId,
    ): Implementations["sender"] {
        throw new Error("Not implemented")
    }
}
function snowbridgeApi(params: SnowbridgeApiParams): SnowbridgeApi {
    return new SnowbridgeApi(params);
}

function test(from: ChainId, to: ChainId) {
    const api = snowbridgeApi({
        bridge: polkadot_mainnet
    })

    const b = api.transferUntyped(from, to)

    const a = api.transfer(
        { kind: "polkadot", id: to.id}, 
        { kind: "ethereum_l2", id: to.id })
    a.f4()
}

;(async () => {
    const api = snowbridgeApi({
        bridge: polkadot_mainnet
    })

    const a = api.transfer(
        { kind: "polkadot", id: 1000}, 
        { kind: "ethereum_l2", id: 8112 })
    a.f4()
})()

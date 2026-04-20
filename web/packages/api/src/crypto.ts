import { Registry } from "@polkadot/types/types"
import { bnToU8a, isHex, stringToU8a, u8aToHex } from "@polkadot/util"
import { blake2AsU8a, decodeAddress, keccak256AsU8a } from "@polkadot/util-crypto"

export const paraIdToSovereignAccount = (type: "para" | "sibl", paraId: number): string => {
    const typeEncoded = stringToU8a(type)
    const paraIdEncoded = bnToU8a(paraId)
    const zeroPadding = new Uint8Array(32 - typeEncoded.length - paraIdEncoded.length).fill(0)
    const address = new Uint8Array([...typeEncoded, ...paraIdEncoded, ...zeroPadding])
    return u8aToHex(address)
}

export function resolveBeneficiary(address: string) {
    if (isHex(address)) {
        if (address.length === 42) {
            return {
                hexAddress: address,
                kind: 2,
            }
        } else if (address.length === 66) {
            return {
                hexAddress: address,
                kind: 1,
            }
        } else {
            throw new Error("Unknown Beneficiary address format.")
        }
    } else {
        return {
            hexAddress: u8aToHex(decodeAddress(address)),
            kind: 1,
        }
    }
}

export const paraIdToAgentId = (registry: Registry, paraId: number): string => {
    const typeEncoded = stringToU8a("SiblingChain")
    const paraIdEncoded = registry.createType("Compact<u32>", paraId).toU8a()
    const joined = new Uint8Array([...typeEncoded, ...paraIdEncoded, 0x00])
    const agentId = blake2AsU8a(joined, 256)
    return u8aToHex(agentId)
}

export const paraIdToChannelId = (paraId: number): string => {
    const typeEncoded = stringToU8a("para")
    const paraIdEncoded = bnToU8a(paraId, { bitLength: 32, isLe: false })
    const joined = new Uint8Array([...typeEncoded, ...paraIdEncoded])
    const channelId = keccak256AsU8a(joined)
    return u8aToHex(channelId)
}

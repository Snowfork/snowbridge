import { createType } from "@polkadot/types";
import { bnToU8a, stringToU8a, u8aToHex } from "@polkadot/util";
import { keccak256AsU8a } from "@polkadot/util-crypto";

export const paraIdToSovereignAccount = (type: 'para' | 'sibl', paraId: number): string => {
    const typeEncoded = stringToU8a(type)
    const paraIdEncoded = bnToU8a(paraId)
    const zeroPadding = new Uint8Array(32 - typeEncoded.length - paraIdEncoded.length).fill(0)
    const address = new Uint8Array([...typeEncoded, ...paraIdEncoded, ...zeroPadding])
    return u8aToHex(address)
}

export const paraIdToChannelId = (paraId: number): string => {
    const typeEncoded = stringToU8a('para')
    const paraIdEncoded = bnToU8a(paraId, { bitLength: 32, isLe: false })
    const joined = new Uint8Array([...typeEncoded, ...paraIdEncoded])
    const channelId = keccak256AsU8a(joined)
    return u8aToHex(channelId)
}

import { bnToU8a, stringToU8a, u8aToHex } from "@polkadot/util";

export const paraIdToSovereignAccount = (type: 'para' | 'sibl', paraId: number): string => {
    let typeEncoded = stringToU8a(type);
    let paraIdEncoded = bnToU8a(paraId);
    let zeroPadding = new Uint8Array(32 - typeEncoded.length - paraIdEncoded.length).fill(0);
    let address = new Uint8Array([...typeEncoded, ...paraIdEncoded, ...zeroPadding]);
    return u8aToHex(address)
}

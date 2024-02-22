import { bnToU8a, isHex, stringToU8a, u8aToHex } from "@polkadot/util";
import { decodeAddress, keccak256AsU8a } from "@polkadot/util-crypto";
import { MultiAddressStruct } from "@snowbridge/contract-types/src/IGateway";
import { ethers } from "ethers";

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

export const beneficiaryMultiAddress = (beneficiary: string) => {
    const abi = ethers.AbiCoder.defaultAbiCoder()

    let address: MultiAddressStruct;
    let hexAddress: string;
    if (isHex(beneficiary)) {
        hexAddress = beneficiary
        if (beneficiary.length === 42) {
            // 20 byte address
            address = {
                kind: 2,
                data: abi.encode(['bytes20'], [hexAddress]),
            }
        } else if (beneficiary.length === 66) {
            // 32 byte address
            address = {
                kind: 1,
                data: abi.encode(['bytes32'], [hexAddress]),
            }
        } else {
            throw new Error('Unknown Beneficiary address format.')
        }
    } else {
        // SS58 address
        hexAddress = u8aToHex(decodeAddress(beneficiary))
        address = {
            kind: 1,
            data: abi.encode(['bytes32'], [hexAddress]),
        }
    }
    return { address, hexAddress }
}
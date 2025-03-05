import { Registry } from "@polkadot/types/types"
import { bnToU8a, hexToU8a, isHex, stringToU8a, u8aToHex } from "@polkadot/util"
import { blake2AsU8a, decodeAddress, keccak256AsU8a } from "@polkadot/util-crypto"
import { MultiAddressStruct } from "@snowbridge/contract-types/src/IGateway"
import { ethers } from "ethers"

export const paraIdToSovereignAccount = (type: "para" | "sibl", paraId: number): string => {
    const typeEncoded = stringToU8a(type)
    const paraIdEncoded = bnToU8a(paraId)
    const zeroPadding = new Uint8Array(32 - typeEncoded.length - paraIdEncoded.length).fill(0)
    const address = new Uint8Array([...typeEncoded, ...paraIdEncoded, ...zeroPadding])
    return u8aToHex(address)
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

export const forwardedTopicId = (messageId: string): string => {
    // From rust code
    // (b"forward_id_for", original_id).using_encoded(sp_io::hashing::blake2_256)
    const typeEncoded = stringToU8a("forward_id_for")
    const paraIdEncoded = hexToU8a(messageId)
    const joined = new Uint8Array([...typeEncoded, ...paraIdEncoded])
    const newTopicId = blake2AsU8a(joined, 256)
    return u8aToHex(newTopicId)
}

export const beneficiaryMultiAddress = (beneficiary: string) => {
    const abi = ethers.AbiCoder.defaultAbiCoder()

    let address: MultiAddressStruct
    let hexAddress: string
    if (isHex(beneficiary)) {
        hexAddress = beneficiary
        if (beneficiary.length === 42) {
            // 20 byte address
            address = {
                kind: 2,
                data: abi.encode(["bytes20"], [hexAddress]),
            }
        } else if (beneficiary.length === 66) {
            // 32 byte address
            address = {
                kind: 1,
                data: abi.encode(["bytes32"], [hexAddress]),
            }
        } else {
            throw new Error("Unknown Beneficiary address format.")
        }
    } else {
        // SS58 address
        hexAddress = u8aToHex(decodeAddress(beneficiary))
        address = {
            kind: 1,
            data: abi.encode(["bytes32"], [hexAddress]),
        }
    }
    return { address, hexAddress }
}

export const fetchBeaconSlot = async (
    beaconUrl: string,
    blockId: `0x${string}` | number | "head" | "finalized"
): Promise<{
    data: {
        message: {
            slot: number
            body: {
                execution_payload?: {
                    block_number: `${number}`
                    block_hash: `0x${string}`
                }
            }
        }
    }
}> => {
    let url = beaconUrl.trim()
    if (!url.endsWith("/")) {
        url += "/"
    }
    url += `eth/v2/beacon/blocks/${blockId}`
    let response = await fetch(url)
    if (!response.ok) {
        throw new Error(response.statusText)
    }
    return await response.json()
}

export const fetchFinalityUpdate = async (
    beaconUrl: string
): Promise<{ finalized_slot: number; attested_slot: number }> => {
    let url = beaconUrl.trim()
    if (!url.endsWith("/")) {
        url += "/"
    }
    url += `eth/v1/beacon/light_client/finality_update`
    let response = await fetch(url)
    if (!response.ok) {
        throw new Error(response.statusText)
    }
    let result: any = await response.json()
    return {
        finalized_slot: Number(result?.data?.finalized_header?.beacon?.slot),
        attested_slot: Number(result?.data?.attested_header?.beacon?.slot),
    }
}

export const getEventIndex = (id: string) => {
    let parts = id.split("-")
    let blockNumber = parseInt(parts[0])
    let eventIndex = parseInt(parts[2])
    return `${blockNumber}-${eventIndex}`
}

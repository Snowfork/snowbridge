export const fetchBeaconSlot = async (
    beaconUrl: string,
    blockId: `0x${string}` | number | "head" | "finalized",
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
    beaconUrl: string,
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
    // Extract eventIndex for compatibility
    let eventIndex
    if (parts.length == 2) {
        eventIndex = parseInt(parts[1])
    } else {
        eventIndex = parseInt(parts[2])
    }
    return `${blockNumber}-${eventIndex}`
}

export function padFeeByPercentage(fee: bigint, padPercent: bigint) {
    if (padPercent < 0 || padPercent > 100) {
        throw Error(`padPercent ${padPercent} not in range of 0 to 100.`)
    }
    return (fee * (100n + padPercent)) / 100n
}

export class ValidationError<
    T extends { success: boolean; logs: { message: string }[] },
> extends Error {
    readonly validation: T

    constructor(validation: T) {
        super("Validation failed.")
        this.name = "ValidationError"
        this.validation = validation
    }
}

export function ensureValidationSuccess<
    T extends { success: boolean; logs: { message: string }[] },
>(validation: T): T {
    if (validation.success) {
        return validation
    }
    throw new ValidationError(validation)
}

export function u32ToLeBytes(value: number): Uint8Array {
    if (!Number.isInteger(value) || value < 0 || value > 0xffffffff) {
        throw new Error(`Value out of u32 range: ${value}`)
    }
    const out = new Uint8Array(4)
    new DataView(out.buffer).setUint32(0, value, true)
    return out
}

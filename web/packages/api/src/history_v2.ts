import {
    ToPolkadotTransferResult,
    ToEthereumTransferResult,
    buildToPolkadotTransferResult,
    buildToEthereumTransferResult,
} from "./history"
import {
    fetchToPolkadotTransfers as fetchToPolkadotTransfersV2,
    fetchToEthereumTransfers as fetchToEthereumTransfersV2,
    fetchToPolkadotTransferById as fetchToPolkadotTransferByIdV2,
    fetchToEthereumTransferById as fetchToEthereumTransferByIdV2,
    fetchToEthereumTransfersBySenders,
    fetchToPolkadotTransfersBySenders,
    fetchToPolkadotPendingTransfers,
    fetchToEthereumPendingTransfers,
} from "./subsquid_v2"

export const toPolkadotHistory = async (
    graphqlApiUrl: string,
    graphqlQuerySize: number = 100,
): Promise<ToPolkadotTransferResult[]> => {
    const allTransfers = await fetchToPolkadotTransfersV2(graphqlApiUrl, graphqlQuerySize)
    const results: ToPolkadotTransferResult[] = []
    for (const transfer of allTransfers) {
        let result = buildToPolkadotTransferResult(transfer)
        results.push(result)
    }
    return results
}

export const toEthereumHistory = async (
    graphqlApiUrl: string,
    graphqlQuerySize: number = 100,
): Promise<ToEthereumTransferResult[]> => {
    const allTransfers = await fetchToEthereumTransfersV2(graphqlApiUrl, graphqlQuerySize)
    const results: ToEthereumTransferResult[] = []
    for (const transfer of allTransfers) {
        let result = buildToEthereumTransferResult(transfer)
        results.push(result)
    }
    return results
}

export const toPolkadotTransferById = async (
    graphqlApiUrl: string,
    id: string,
): Promise<ToPolkadotTransferResult | undefined> => {
    const transfers = await fetchToPolkadotTransferByIdV2(graphqlApiUrl, id)
    if (transfers?.length > 0) {
        let result = buildToPolkadotTransferResult(transfers[0])
        return result
    }
    return
}

export const toEthereumTransferById = async (
    graphqlApiUrl: string,
    id: string,
): Promise<ToEthereumTransferResult | undefined> => {
    const transfers = await fetchToEthereumTransferByIdV2(graphqlApiUrl, id)
    if (transfers?.length > 0) {
        let result = buildToEthereumTransferResult(transfers[0])
        return result
    }
    return
}

export const toPolkadotTransfersBySenders = async (
    graphqlApiUrl: string,
    graphqlQuerySize: number = 100,
    senders: string[],
): Promise<ToPolkadotTransferResult[]> => {
    const allTransfers = await fetchToPolkadotTransfersBySenders(
        graphqlApiUrl,
        graphqlQuerySize,
        senders,
    )
    const results: ToPolkadotTransferResult[] = []
    for (const transfer of allTransfers) {
        let result = buildToPolkadotTransferResult(transfer)
        results.push(result)
    }
    return results
}

export const toEthereumTransfersBySenders = async (
    graphqlApiUrl: string,
    graphqlQuerySize: number = 100,
    senders: string[],
): Promise<ToEthereumTransferResult[]> => {
    const allTransfers = await fetchToEthereumTransfersBySenders(
        graphqlApiUrl,
        graphqlQuerySize,
        senders,
    )
    const results: ToEthereumTransferResult[] = []
    for (const transfer of allTransfers) {
        let result = buildToEthereumTransferResult(transfer)
        results.push(result)
    }
    return results
}

export const toPolkadotPendingTransfers = async (
    graphqlApiUrl: string,
    graphqlQuerySize: number = 100,
): Promise<ToPolkadotTransferResult[]> => {
    const allTransfers = await fetchToPolkadotPendingTransfers(graphqlApiUrl, graphqlQuerySize)
    const results: ToPolkadotTransferResult[] = []
    for (const transfer of allTransfers) {
        let result = buildToPolkadotTransferResult(transfer)
        results.push(result)
    }
    return results
}

export const toEthereumPendingTransfers = async (
    graphqlApiUrl: string,
    graphqlQuerySize: number = 100,
): Promise<ToEthereumTransferResult[]> => {
    const allTransfers = await fetchToEthereumPendingTransfers(graphqlApiUrl, graphqlQuerySize)
    const results: ToEthereumTransferResult[] = []
    for (const transfer of allTransfers) {
        if (transfer.status === 0) {
            let result = buildToEthereumTransferResult(transfer)
            results.push(result)
        }
    }
    return results
}

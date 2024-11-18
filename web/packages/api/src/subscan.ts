export type SubscanResult = {
    status: number
    statusText: string
    json: any
    rateLimit: SubscanRateLimit
}

export type SubscanRateLimit = {
    limit: number | null
    reset: number | null
    remaining: number | null
    retryAfter: number | null
}

export type SubscanApiPost = (subUrl: string, body: any) => Promise<SubscanResult>
export interface SubscanApi {
    post: SubscanApiPost
}

const sleepMs = async (ms: number) => {
    await new Promise<void>((resolve) => {
        const id = setTimeout(() => {
            resolve()
            clearTimeout(id)
        }, ms)
    })
}

export const createApi = (baseUrl: string, apiKey: string, options = { limit: 1 }): SubscanApi => {
    let url = baseUrl.trim()
    if (!url.endsWith("/")) {
        url += "/"
    }

    const headers = new Headers()
    headers.append("Content-Type", "application/json")
    headers.append("x-api-key", apiKey)

    let rateLimit: SubscanRateLimit = {
        limit: options.limit,
        reset: 0,
        remaining: options.limit,
        retryAfter: 0,
    }
    const post: SubscanApiPost = async (subUrl: string, body: any) => {
        const request: RequestInit = {
            method: "POST",
            headers,
            body: JSON.stringify(body),
            redirect: "follow",
        }

        if (rateLimit.remaining === 0 && rateLimit.retryAfter !== null && rateLimit.retryAfter > 0) {
            console.log("Being rate limited: retryAfter", rateLimit)
            await sleepMs(rateLimit.retryAfter * 1000)
        }
        if (rateLimit.remaining === 0 && rateLimit.reset !== null && rateLimit.reset > 0) {
            console.log("Being rate limited: reset", rateLimit)
            await sleepMs(rateLimit.reset * 1000)
        }

        const response = await fetch(`${url}${subUrl}`, request)

        rateLimit.limit = Number(response.headers.get("ratelimit-limit"))
        rateLimit.reset = Number(response.headers.get("ratelimit-reset"))
        rateLimit.remaining = Number(response.headers.get("ratelimit-remaining"))
        rateLimit.retryAfter = Number(response.headers.get("retry-after"))

        if (response.status !== 200) {
            throw new Error(
                `Failed to fetch from Subscan: ${response.status} ${response.statusText}`
            )
        }

        const json = await response.json()
        return {
            status: response.status,
            statusText: response.statusText,
            json,
            rateLimit: { ...rateLimit },
        }
    }

    return {
        post,
    }
}

export const fetchBlockNearTimestamp = async (api: SubscanApi, utcTimestamp: number) => {
    const body = {
        block_timestamp: utcTimestamp,
        only_head: true,
    }
    const response = await api.post("api/scan/block", body)
    if (response.json.data !== null) {
        return {
            block_num: response.json.data.block_num,
            block_hash: response.json.data.block_hash,
            block_timestamp: response.json.data.block_timestamp,
            status: response.status,
            statusText: response.statusText,
        }
    } else {
        return {
            status: response.status,
            statusText: response.statusText,
        }
    }
}

export const fetchEvents = async <T>(
    api: SubscanApi,
    module: string,
    eventIds: string[],
    fromBlock: number,
    toBlock: number,
    page: number,
    rows: number,
    filterMap: (events: any, params: any) => Promise<T | null>
) => {
    const eventsBody = {
        module,
        block_range: `${fromBlock}-${toBlock}`,
        event_id: eventIds.length === 1 ? eventIds[0] : undefined,
        row: rows,
        page,
    }

    const eventResponse = await api.post("api/v2/scan/events", eventsBody)

    let endOfPages = false
    if (eventResponse.json.data.events === null) {
        eventResponse.json.data.events = []
        endOfPages = true
    }

    const map = new Map<string, any>()
    eventResponse.json.data.events
        .filter((e: any) => eventIds.includes(e.event_id))
        .forEach((e: any) => {
            map.set(e.event_index, e)
        })

    const events = []

    if (map.size > 0) {
        const paramsBody = { event_index: Array.from(map.keys()) }
        const paramsResponse = await api.post("api/scan/event/params", paramsBody)

        if (paramsResponse.json.data === null) {
            paramsResponse.json.data = []
        }

        for (const { event_index, params } of paramsResponse.json.data) {
            if (params === undefined) {
                console.warn("Event does not have any params", event_index)
                continue;
            }

            const event = map.get(event_index)
            const transform = await filterMap(event, params)
            if (transform === null) {
                continue
            }
            events.push({ ...event, params, data: transform })
        }
    }
    return {
        status: eventResponse.status,
        statusText: eventResponse.statusText,
        events,
        endOfPages,
    }
}

export const fetchExtrinsics = async <T>(
    api: SubscanApi,
    module: string,
    call: string,
    fromBlock: number,
    toBlock: number,
    page: number,
    rows: number,
    filterMap: (extrinsic: any, params: any) => Promise<T | null>
) => {
    const extBody = {
        module,
        call,
        block_range: `${fromBlock}-${toBlock}`,
        row: rows,
        page,
    }
    const extResponse = await api.post("api/v2/scan/extrinsics", extBody)

    let endOfPages = false
    if (extResponse.json.data.extrinsics === null) {
        extResponse.json.data.extrinsics = []
        endOfPages = true
    }
    const map = new Map<string, any>()
    extResponse.json.data.extrinsics.forEach((e: any) => {
        map.set(e.extrinsic_index, e)
    })

    const extrinsics = []

    if (map.size > 0) {
        const paramsBody = { extrinsic_index: Array.from(map.keys()) }
        const extParams = await api.post("api/scan/extrinsic/params", paramsBody)

        if (extParams.json.data === null) {
            extParams.json.data = []
        }

        for (const { extrinsic_index, params } of extParams.json.data) {
            const event = map.get(extrinsic_index)
            const transform = await filterMap(event, params)
            if (transform === null) {
                continue
            }

            extrinsics.push({ ...event, params, data: transform })
        }
    }
    return {
        status: extResponse.status,
        statusText: extResponse.statusText,
        extrinsics,
        endOfPages,
    }
}

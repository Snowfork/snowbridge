/**
 * Empirically verify the assumption behind BeefyClient's non-consecutive
 * "skip-ahead" validator set updates, against a live Polkadot relay chain:
 *
 *   - the BEEFY `validatorSetId` increments every session (~4h), but
 *   - the validator *membership* root only rotates at era boundaries
 *     (~24h, i.e. every 6 sessions), so
 *   - within an era `current.root == next.root` holds for every session
 *     except the era's last one.
 *
 * The `cur==next` column below is exactly BeefyClient.canSkipAhead's
 * "confirmed-stable era" gate, and `keysetCommitment` is the membership root
 * the contract authenticates against (via beefyMmrApi.{authoritySetProof,
 * nextAuthoritySetProof} / beefyMmrLeaf.beefy{,Next}Authorities).
 *
 * Connects to a live relay-chain RPC by default; override with RELAY_WS.
 * Note: requires an archive node so historical state is available across the
 * sampled window.
 *
 * Usage: `npx ts-node web/packages/operations/src/verify_beefy_rotation.ts [hours]`
 */

import { ApiPromise, WsProvider } from "@polkadot/api"

const RELAY_WS = process.env.RELAY_WS ?? "wss://polkadot.api.onfinality.io/public-ws"
const HOURS = Number(process.argv[2] ?? 36)

const short = (h: { toString(): string }): string => {
    const s = h.toString()
    return s.slice(0, 8) + "…" + s.slice(-4)
}

interface Sample {
    block: number
    session: number
    vsetId: number
    curRoot: string
    nextRoot: string
}

// Structural views over the generic Codec results, for just the fields we read.
type NumCodec = { toNumber(): number }
type HexCodec = { toHex(): string; toString(): string }
type AuthoritySet = {
    id: { toString(): string }
    len: { toString(): string }
    keysetCommitment: HexCodec
}

// Public RPC nodes rate-limit and can silently drop an in-flight request on a
// reconnect; without a timeout that leaves a query pending forever and the
// sweep appears to hang. Bound every request so a lost one rejects instead.
const RPC_TIMEOUT_MS = 30_000

async function main() {
    console.log(`Connecting to ${RELAY_WS}`)
    const api = await ApiPromise.create({
        provider: new WsProvider(RELAY_WS, 2_500, {}, RPC_TIMEOUT_MS),
    })

    try {
        const head = await api.rpc.chain.getFinalizedHead()
        const N = (await api.rpc.chain.getHeader(head)).number.toNumber()
        console.log(`Finalized block: ${N}`)

        // The runtime calls the contract's authority set is derived from.
        const curProof = (await api.call.beefyMmrApi.authoritySetProof()) as unknown as AuthoritySet
        const nextProof =
            (await api.call.beefyMmrApi.nextAuthoritySetProof()) as unknown as AuthoritySet
        console.log(
            `\nbeefyMmrApi.authoritySetProof()     -> id=${curProof.id} len=${curProof.len} root=${short(curProof.keysetCommitment)}`,
        )
        console.log(
            `beefyMmrApi.nextAuthoritySetProof() -> id=${nextProof.id} len=${nextProof.len} root=${short(nextProof.keysetCommitment)}`,
        )

        // Fail fast if the endpoint does not carry the pallets we sample. Without this the
        // per-sample catch below would swallow a TypeError on every iteration and the run would
        // end up reporting "verified" over zero rows.
        for (const [section, item] of [
            ["session", "currentIndex"],
            ["beefy", "validatorSetId"],
            ["beefyMmrLeaf", "beefyAuthorities"],
            ["beefyMmrLeaf", "beefyNextAuthorities"],
        ] as const) {
            if (
                !(api.query as Record<string, Record<string, unknown> | undefined>)[section]?.[item]
            ) {
                throw new Error(
                    `${RELAY_WS} does not expose query.${section}.${item}. ` +
                        `Point RELAY_WS at a relay chain running BEEFY.`,
                )
            }
        }

        // Historical sweep, sampling ~every 30 min to catch every 4h session.
        const STEP = 300 // ~30 min at 6s block time
        const SPAN = Math.round((HOURS * 3600) / 6) // window in blocks
        const total = Math.floor(SPAN / STEP) + 1
        const rows: Sample[] = []
        // Kept apart: unavailable state is expected on a non-archive node, whereas a query that
        // throws points at a real problem. Collapsing both into one "pruned" tally is what let a
        // fully failed run read as a clean one.
        let unavailable = 0
        let queryFailed = 0
        let firstQueryError: string | null = null
        let i = 0
        console.log(`\nSampling ${total} blocks (every ${STEP} blocks) ...`)
        for (let b = N; b >= N - SPAN; b -= STEP) {
            process.stdout.write(`\r  sample ${++i}/${total} @ block ${b}   `)
            let at
            try {
                at = await api.at(await api.rpc.chain.getBlockHash(b))
            } catch {
                unavailable++
                continue
            }
            try {
                const [sess, vid, cur, nxt] = await Promise.all([
                    at.query.session.currentIndex(),
                    at.query.beefy.validatorSetId(),
                    at.query.beefyMmrLeaf.beefyAuthorities(),
                    at.query.beefyMmrLeaf.beefyNextAuthorities(),
                ])
                rows.push({
                    block: b,
                    session: (sess as unknown as NumCodec).toNumber(),
                    vsetId: (vid as unknown as NumCodec).toNumber(),
                    curRoot: (cur as unknown as AuthoritySet).keysetCommitment.toHex(),
                    nextRoot: (nxt as unknown as AuthoritySet).keysetCommitment.toHex(),
                })
            } catch (e) {
                queryFailed++
                if (firstQueryError === null) {
                    firstQueryError = e instanceof Error ? e.message : String(e)
                }
            }
        }
        process.stdout.write("\n")
        rows.reverse()
        if (unavailable) {
            console.log(
                `\n(${unavailable}/${i} samples: state unavailable — pruned, or ${RELAY_WS} is not an archive node)`,
            )
        }
        if (queryFailed) {
            console.log(
                `\n(${queryFailed}/${i} samples: storage query failed${firstQueryError ? ` — first error: ${firstQueryError}` : ""})`,
            )
        }

        // Collapse consecutive samples that share the same state.
        const key = (r: Sample) => `${r.session}|${r.vsetId}|${r.curRoot}|${r.nextRoot}`
        const collapsed: Sample[] = []
        for (const r of rows) {
            const last = collapsed[collapsed.length - 1]
            if (last && key(last) === key(r)) continue
            collapsed.push({ ...r })
        }

        // Nothing observed means nothing verified. Reporting the summary here would print
        // "vsetIds consecutive (+1) : true" off an empty array and read as a confirmation.
        if (collapsed.length === 0) {
            console.error(
                `\nERROR: 0 of ${i} samples produced state. Nothing was verified.\n` +
                    `  ${unavailable} unavailable, ${queryFailed} failed to query.\n` +
                    `  Set RELAY_WS to an archive endpoint that serves the sampled range and retry.`,
            )
            process.exitCode = 1
            return
        }

        const failed = unavailable + queryFailed
        if (failed > i / 4) {
            console.log(
                `\nWARNING: ${failed} of ${i} samples failed. Coverage is partial — treat the` +
                    ` summary below as indicative, not as a verification.`,
            )
        }

        console.log(`\nPer-session BEEFY state over ~${HOURS}h (oldest -> newest):`)
        console.log("  session  vsetId  curRoot       nextRoot      cur==next")
        let prevRoot: string | null = null
        for (const r of collapsed) {
            const rootChanged = prevRoot !== null && r.curRoot !== prevRoot
            console.log(
                `  ${String(r.session).padEnd(7)}  ${String(r.vsetId).padEnd(6)}  ${short({ toString: () => r.curRoot }).padEnd(13)}  ${short({ toString: () => r.nextRoot }).padEnd(13)}  ${r.curRoot === r.nextRoot ? "YES" : "no "}${rootChanged ? "   <== membership root changed (era rotation)" : ""}`,
            )
            prevRoot = r.curRoot
        }

        const sessions = [...new Set(collapsed.map((r) => r.session))]
        const vsetIds = [...new Set(collapsed.map((r) => r.vsetId))]
        const roots = [...new Set(collapsed.map((r) => r.curRoot))]
        const lastOfEra = collapsed.filter((r) => r.curRoot !== r.nextRoot)
        // A single observed id satisfies `every` vacuously, which says nothing about whether ids
        // advance by one. Only claim the property when there are at least two to compare.
        const consecutive = vsetIds.every((v, n, a) => n === 0 || v === a[n - 1] + 1)
        console.log("\nSummary:")
        console.log(
            `  sessions observed        : ${sessions.length} (${sessions[0]}…${sessions[sessions.length - 1]})`,
        )
        console.log(
            `  vsetIds consecutive (+1) : ${vsetIds.length > 1 ? consecutive : `n/a (only ${vsetIds.length} observed)`}`,
        )
        console.log(`  distinct membership roots: ${roots.length}`)
        console.log(
            `  skip-ahead available in  : ${collapsed.length - lastOfEra.length} of ${collapsed.length} sessions (refused only in each era's last session)`,
        )

        // The script exists to test this premise, so a run that disproves it must not exit 0.
        if (vsetIds.length > 1 && !consecutive) {
            console.error(
                `\nERROR: vsetIds are not consecutive over the sampled window.` +
                    ` The skip-ahead premise does not hold here.`,
            )
            process.exitCode = 1
        }
    } finally {
        await api.disconnect()
    }
}

main().catch((e) => {
    console.error(e)
    process.exit(1)
})

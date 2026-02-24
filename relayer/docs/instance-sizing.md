# EC2 Instance Sizing Results

This document tracks performance benchmarks for running Snowbridge relayers on different EC2 instance types.

## Test Configuration

- **Network**: Mainnet (Polkadot + Ethereum)
- **Services**: All relayers including beefy (expensive profile)
- **Metric**: Beacon state proof generation time

## Results

| Instance Type | vCPUs | RAM | Unmarshal (ms) | Tree (ms) | Proofs (ms) | Total (ms) | Status |
|---------------|-------|-----|----------------|-----------|-------------|------------|--------|
| m6a.xlarge | 4 | 16 GB | ~3,000 | ~5 | ~10 | ~3,100 | :white_check_mark: Over-provisioned |
| **m6a.large** | 2 | 8 GB | ~3,000 | ~5 | ~9 | ~3,000 | :white_check_mark: **Recommended** |
| t3.large | 2 | 8 GB | ~22,000 | ~7 | ~37 | ~22,000 | :x: Too slow |

## Detailed Results

### m6a.xlarge (4 vCPU, 16 GB RAM)

**Beacon State Proof Generation:**
```
slot=13614560: unmarshalMs=3092, treeMs=6, proofsMs=10, totalMs=3133
slot=13614625: unmarshalMs=2884, treeMs=3, proofsMs=7,  totalMs=2897
```

**Beacon State Download:**
- ~1.5-2 seconds per state

**Full Cycle Time:**
- ~5 seconds (download + proof generation)

**Verdict:** Good performance, but over-provisioned on memory (using ~2GB of 16GB).

---

### t3.large (2 vCPU, 8 GB RAM) - Burstable

**Beacon State Proof Generation:**
```
slot=13614592: unmarshalMs=22189, treeMs=9, proofsMs=39, totalMs=22271
slot=13614657: unmarshalMs=23143, treeMs=6, proofsMs=36, totalMs=23188
slot=13614624: unmarshalMs=21733, treeMs=8, proofsMs=39, totalMs=21813
slot=13614688: unmarshalMs=21725, treeMs=6, proofsMs=36, totalMs=21771
```

**Beacon State Download:**
- ~1.6-1.7 seconds per state

**Full Cycle Time:**
- ~44 seconds for 2 states (download + proof generation)

**Issues Observed:**
- `503` errors: Beacon relay requesting proofs before ready
- 7x slower than m6a.xlarge due to burstable CPU limits

**Verdict:** Too slow. CPU-intensive SSZ hashing exhausts burst credits.

---

### m6a.large (2 vCPU, 8 GB RAM) - Recommended

**Beacon State Proof Generation:**
```
slot=13614656: unmarshalMs=3128, treeMs=7, proofsMs=10, totalMs=3165
slot=13614721: unmarshalMs=2877, treeMs=3, proofsMs=7,  totalMs=2891
```

**Beacon State Download:**
- ~1.6-1.7 seconds per state

**Full Cycle Time:**
- ~5-6 seconds for 2 states (download + proof generation)

**Verdict:** Same performance as m6a.xlarge at half the cost. The SSZ hashing workload is single-threaded, so 2 dedicated vCPUs is sufficient. 8 GB RAM provides adequate headroom.

---

## Recommendation

**Use m6a.large (2 vCPU, 8 GB RAM)** for running Snowbridge relayers.

Key findings:
- The SSZ hashing workload is **single-threaded**, so extra vCPUs don't help
- **Dedicated CPU is essential** - burstable instances (t3) cannot sustain the hashing workload
- **8 GB RAM is sufficient** - actual usage is ~2-3 GB with headroom for spikes
- Estimated monthly cost: ~$70 (vs ~$140 for m6a.xlarge)

**Storage:** 50 GB gp3 EBS volume. Actual usage is well under 10 GB (Docker image, beacon state cache, SQLite datastores), but 50 GB provides comfortable headroom at ~$4/mo.

**Avoid:**
- Burstable instances (t3.*, t4g.*) - CPU throttling causes 7x slowdown
- Over-provisioned instances (m6a.xlarge) - extra resources unused

---

*Last updated: 2026-02-04*

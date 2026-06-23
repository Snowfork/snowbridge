import urllib.request
import json
import sys
import time

RPC_URLS = [
    "https://eth.merkle.io",
    "https://ethereum-rpc.publicnode.com",
    "https://eth.llamarpc.com",
    "https://rpc.ankr.com/eth",
    "https://eth-mainnet.public.blastapi.io",
]
RPC_URL = RPC_URLS[0]
ADDRESS = "0x7cfc5C8b341991993080Af67D940B6aD19a010E1"

NEW_TICKET = "0xbee983fc706c692efb9b0240bddc5666c010a53af55ed5fb42d226e7e4293869"   # NewTicket(address,uint64) -> submitInitial
NEW_MMR_ROOT = "0xd95fe1258d152dc91c81b09380498adc76ed36a6079bcb2ed31eff622ae2d0f1"  # NewMMRRoot(bytes32,uint64) -> submitFinal

SEL_INITIAL = "0xbb51f1eb"
SEL_FINAL = "0x623b223d"

_id = 0


def rpc(method, params):
    global _id, RPC_URL
    _id += 1
    payload = json.dumps({
        "jsonrpc": "2.0", "method": method, "params": params, "id": _id
    }).encode()
    last_err = None
    for url in RPC_URLS:
        for attempt in range(3):
            try:
                req = urllib.request.Request(url, data=payload, headers={
                    'Content-Type': 'application/json',
                    'User-Agent': 'Mozilla/5.0',
                    'Accept': 'application/json',
                })
                with urllib.request.urlopen(req, timeout=60) as response:
                    out = json.loads(response.read())
                if 'error' in out:
                    raise RuntimeError(out['error'])
                RPC_URL = url
                return out['result']
            except Exception as e:
                last_err = e
                time.sleep(1.0 * (attempt + 1))
    raise last_err


latest = int(rpc("eth_blockNumber", []), 16)
print(f"Latest block: {latest}")

CHUNK = 9000
initials = {}  # beefyBlock -> (tx_hash, eth_block)
finals = {}    # beefyBlock -> (tx_hash, eth_block)

# scan backwards until we have at least 6 complete pairs
hi = latest
PAIRS_WANTED = 6
while hi > 0:
    lo = max(0, hi - CHUNK)
    logs = rpc("eth_getLogs", [{
        "address": ADDRESS,
        "fromBlock": hex(lo),
        "toBlock": hex(hi),
        "topics": [[NEW_TICKET, NEW_MMR_ROOT]],
    }])
    for log in logs:
        t0 = log['topics'][0]
        eth_block = int(log['blockNumber'], 16)
        tx_hash = log['transactionHash']
        # NewTicket: blockNumber is non-indexed (in data, 2nd word). NewMMRRoot: blockNumber non-indexed (2nd word).
        data = log['data'][2:]
        beefy_block = int(data[64:128], 16)
        if t0 == NEW_TICKET:
            initials[beefy_block] = (tx_hash, eth_block)
        else:
            finals[beefy_block] = (tx_hash, eth_block)
    pairs = sorted(set(initials) & set(finals), reverse=True)
    print(f"scanned [{lo},{hi}] initials={len(initials)} finals={len(finals)} pairs={len(pairs)}")
    if len(pairs) >= PAIRS_WANTED:
        break
    hi = lo - 1

pairs = sorted(set(initials) & set(finals), reverse=True)[:5]
print(f"\nSelected {len(pairs)} most recent pairs (by beefy block):")

result = []
for bb in pairs:
    ih, ieb = initials[bb]
    fh, feb = finals[bb]
    itx = rpc("eth_getTransactionByHash", [ih])
    ftx = rpc("eth_getTransactionByHash", [fh])
    assert itx['input'].startswith(SEL_INITIAL), f"{ih} not submitInitial: {itx['input'][:10]}"
    assert ftx['input'].startswith(SEL_FINAL), f"{fh} not submitFinal: {ftx['input'][:10]}"
    assert itx['to'].lower() == ADDRESS.lower()
    assert ftx['to'].lower() == ADDRESS.lower()
    rec = {
        "beefyBlock": bb,
        "submitInitial": {"tx": ih, "ethBlock": ieb, "from": itx['from'], "input": itx['input']},
        "submitFinal": {"tx": fh, "ethBlock": feb, "from": ftx['from'], "input": ftx['input']},
    }
    result.append(rec)
    print(f"  beefyBlock={bb}")
    print(f"    initial tx={ih} block={ieb} from={itx['from']}")
    print(f"    final   tx={fh} block={feb} from={ftx['from']}")

with open("pairs.json", "w") as f:
    json.dump(result, f, indent=2)
print("\nWrote pairs.json")

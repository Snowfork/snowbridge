# Operations Scripts

Scripts for operating the bridge.

## Monitor bridge status

Monitor bridge status and important channels.

```console
pnpx ts-node src/monitor.ts 
```
```
Bridge Status: {
  toEthereum: {
    operatingMode: { outbound: 'Normal' },
    latestPolkadotBlockOnEthereum: 89,
    latestPolkaotBlock: 100,
    blockLatency: 11,
    latencySeconds: 66
  },
  toPolkadot: {
    operatingMode: { beacon: 'Normal', inbound: 'Normal', outbound: 'Normal' },
    latestEthereumBlockOnPolkadot: 368,
    latestEthereumBlock: 614,
    blockLatency: 246,
    latencySeconds: 2952
  }
}
Asset Hub Channel: {
  toEthereum: { outbound: 0, inbound: 0 },
  toPolkadot: { operatingMode: { outbound: 'Normal' }, outbound: 0, inbound: 0 }
}
Primary Governance Channel: {
  toEthereum: { outbound: 0, inbound: 0 },
  toPolkadot: { operatingMode: { outbound: 'Normal' }, outbound: 0, inbound: 0 }
}
Secondary Governance Channel: {
  toEthereum: { outbound: 0, inbound: 0 },
  toPolkadot: { operatingMode: { outbound: 'Normal' }, outbound: 0, inbound: 0 }
}
```
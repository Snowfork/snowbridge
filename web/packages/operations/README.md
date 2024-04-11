# Operations Scripts

Scripts for operating the bridge.

## Monitor bridge status

Monitor bridge status and important channels.

1. Monitoring script. Currently just prints information about the current bridge setup. (Works in both e2e and prod)
For production:
```
$ NODE_ENV=production REACT_APP_INFURA_KEY=... npx ts-node src/monitor.ts
Bridge Status: {
  toEthereum: {
    operatingMode: { outbound: 'Normal' },
    latestPolkadotBlockOnEthereum: 9451242,
    latestPolkaotBlock: 9468776,
    blockLatency: 17534,
    latencySeconds: 105204
  },
  toPolkadot: {
    operatingMode: { beacon: 'Normal', inbound: 'Normal', outbound: 'Normal' },
    latestEthereumBlockOnPolkadot: 5418510,
    latestEthereumBlock: 5437907,
    blockLatency: 19397,
    latencySeconds: 232764
  }
}
Asset Hub Channel: {
  toEthereum: { outbound: 8, inbound: 8 },
  toPolkadot: { operatingMode: { outbound: 'Normal' }, outbound: 30, inbound: 28 }
}
Primary Governance Channel: {
  toEthereum: { outbound: 1, inbound: 1 },
  toPolkadot: { operatingMode: { outbound: 'Normal' }, outbound: 0, inbound: 0 }
}
Secondary Governance Channel: {
  toEthereum: { outbound: 0, inbound: 0 },
  toPolkadot: { operatingMode: { outbound: 'Normal' }, outbound: 0, inbound: 0 }
}
Asset Hub Sovereign balance on bridgehub: 95088305295866n
Asset Hub Agent balance: 19976725459747494287n
Bridge Hub Agent balance: 4998205431258082952n
Relayers:
         44983302355375n : substrate balance -> beacon
         36273071477795524248n : ethereum balance -> beefy
         5001076910556582332n : ethereum balance -> parachain-primary-gov
         5000000000000000000n : ethereum balance -> parachain-secondary-gov
         54604255523507n : substrate balance -> execution-assethub
         4987531971873371681n : ethereum balance -> parachain-assethub
```

For local e2e:
```
npx ts-node src/monitor.ts
```

2. Round trip transfer script. (Only works locally for now, used to dev/test the api)
```
pnpx ts-node src/transfer_token.ts 
```

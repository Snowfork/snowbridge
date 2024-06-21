# Operations Scripts

Scripts for operating the bridge with metrics sent to cloudwatch and alarms integration with pagerduty.

## Env configuration

Config the `.env` follows `.env.example`, mainly for aws access key/secret and api keys for the infura/alchemy endpoint.

The `*SNS_TOPIC` in the config file should match the [aws sns topic](https://eu-central-1.console.aws.amazon.com/sns/v3/home?region=eu-central-1#/topics) and link to the [pagerduty service](https://snowfork.eu.pagerduty.com/service-directory), both already been created and configured. No need any change except you understand it.

## Initialize alarms

Currently there are only [a few alarms](https://github.com/Snowfork/snowbridge/pull/1196#issue-2288992655) supported. Mainly for checking the bridge stale and wallet insufficient. But we can add more later if necessary.

Before monitoring the bridge status, first step is to initialize the alarm rules with the command:

```
pnpm initialize
```

The alarm rules will be created in [cloudwatch page](https://eu-central-1.console.aws.amazon.com/cloudwatch/home?region=eu-central-1#alarmsV2:), check created as expected.

## Monitor bridge status

Monitor bridge status and important channels.

1. Monitoring script to send metrics to cloudwatch and trigger alarms if match the [threshold](https://github.com/Snowfork/snowbridge/blob/d36a91698f2acce132090e849c212b754fb023e5/web/packages/api/src/status.ts#L67-L79) configured.

For production:
```
$ NODE_ENV=polkadot_mainnet pnpm start
Bridge Status: {
  toEthereum: {
    operatingMode: { outbound: 'Normal' },
    latestPolkadotBlockOnEthereum: 21291707,
    latestPolkadotBlock: 21293770,
    blockLatency: 2056,
    latencySeconds: 12336,
    previousPolkadotBlockOnEthereum: 21291707
  },
  toPolkadot: {
    operatingMode: { beacon: 'Normal', inbound: 'Normal', outbound: 'Normal' },
    latestBeaconSlotOnPolkadot: 9335648,
    latestBeaconSlotAttested: 9335715,
    latestBeaconSlotFinalized: 9335648,
    blockLatency: 67,
    latencySeconds: 804,
    previousEthereumBlockOnPolkadot: 9332576
  }
}
Asset Hub Channel: {
  toEthereum: { outbound: 6, inbound: 6, previousOutbound: 4, previousInbound: 6 },
  toPolkadot: {
    operatingMode: { outbound: 'Normal' },
    outbound: 45,
    inbound: 45,
    previousOutbound: 45,
    previousInbound: 41
  },
  name: 'AssetHub'
}
Primary Governance Channel: {
  toEthereum: { outbound: 2, inbound: 2, previousOutbound: 2, previousInbound: 2 },
  toPolkadot: {
    operatingMode: { outbound: 'Normal' },
    outbound: 0,
    inbound: 0,
    previousOutbound: 0,
    previousInbound: 0
  },
  name: 'Primary'
}
Secondary Governance Channel: {
  toEthereum: { outbound: 0, inbound: 0, previousOutbound: 0, previousInbound: 0 },
  toPolkadot: {
    operatingMode: { outbound: 'Normal' },
    outbound: 0,
    inbound: 0,
    previousOutbound: 0,
    previousInbound: 0
  },
  name: 'Secondary'
}
Asset Hub Sovereign balance on bridgehub: 3865356600000n
Asset Hub Agent balance: 534464614671795711n
Bridge Hub Agent balance: 0n
Relayers:
         3651503609865n : substrate balance -> beacon
         625738383685108379n : ethereum balance -> beefy
         856828234768564376n : ethereum balance -> parachain-primary-gov
         856828234768564376n : ethereum balance -> parachain-secondary-gov
         3651503609865n : substrate balance -> execution-assethub
         856828234768564376n : ethereum balance -> parachain-assethub
```

For local e2e:
```
NODE_ENV=local_e2e pnpm start
```

There is also a cron script to run it periodically:
```
pnpm cron
```

2. Round trip transfer script. (Only works locally for now, used to dev/test the api)
```
pnpx ts-node src/transfer_token.ts
```

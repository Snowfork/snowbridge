# Monitoring service

Scripts in place to monitor the bridge, sending metrics to CloudWatch, with alarms integrated into PagerDuty for real-time notifications.

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

## Monitor bridge/channel status

### Run as a one-shot task

```
$ pnpm start
```

### Run periodically as a cron job

```
pnpm cron
```

### Install as daemon service with PM2

```
pm2 start ecosystem.config.js --only monitor --time
```

## Prometheus exporter

Long-running alternative to the CloudWatch one-shot. Serves `/metrics` and refreshes its
in-memory dataset every 15 minutes, running the same collection as `monitor` followed by a
fisherman scan. Any collection failure exits the process so the container restarts and the
orchestrator's container-down alarm fires.

```
pnpm monitorPrometheus
docker run -p 9000:9000 -v $(pwd)/config:/config snowbridge-monitor:latest prometheus
```

| Env var | Default | Meaning |
| --- | --- | --- |
| `METRICS_PORT` | `9000` | HTTP listen port |
| `METRICS_HOST` | `0.0.0.0` | HTTP bind address |
| `MONITOR_REFRESH_INTERVAL_SECONDS` | `900` | Seconds between refreshes |
| `MONITOR_TICK_TIMEOUT_SECONDS` | `1800` | A refresh exceeding this exits the process |
| `FISHERMAN_ENABLED` | `true` | Set `false` to collect bridge metrics only |
| `FISHERMAN_CHECKPOINT_PATH` | `checkpoint.json` | Where the fisherman checkpoint lives |

The exporter does not publish to CloudWatch, so it needs no AWS credentials.

**The checkpoint needs a persistent volume.** `FISHERMAN_CHECKPOINT_PATH` defaults to a
relative path, which inside a container lands in the ephemeral image layer. If it is lost
the fisherman rewinds to `FISHERMAN_START_BLOCK` and re-scans forward 5000 blocks per run,
which takes hours and re-alarms on everything in that range. Mount a volume and point
`FISHERMAN_CHECKPOINT_PATH` at it, and seed it from the previous checkpoint when migrating
an existing deployment.

Alerting is driven by `snowbridge_monitor_seconds_since_last_refresh` (`-1` until the first
successful refresh) and `snowbridge_collection_section_ok{section}`. The latter matters
because indexer and pool-liquidity failures are swallowed so they cannot take the rest of
the collection down — without it those series would simply disappear from `/metrics` and
their threshold alarms would stop evaluating silently.

# Transfers on Westend

We run the transfer on a daily basis to preemptively ensure that the bridge transfer won’t break.

```
pm2 start westend-ecosystem.config.js --only westend-transferToPolkadot,westend-transferToEthereum
```

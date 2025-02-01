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

## Monitor bridge/channel status

### Run as a one-shot task

```
$ pnpm start
```

### Run periodically as a cron job

```
pnpm cron
```

### Install as dameon service with PM2

```
pm2 start ecosystem.config.js --only monitor
```


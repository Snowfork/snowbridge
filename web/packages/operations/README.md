# Operational scripts

Ad-hoc scripts for operating and exercising the bridge: transfers, token registration,
governance submissions, and halt simulations. Run them with `pnpm <script>` — see the
`scripts` section of `package.json` for the full list.

Configure `.env` following `.env.example`.

> Bridge monitoring, CloudWatch alarms, the Prometheus exporter, and the Beefy fisherman
> now live in [Snowfork/monitor](https://github.com/snowfork/monitor).

# Transfers on Westend

We run the transfer on a daily basis to preemptively ensure that the bridge transfer won’t break.

```
pm2 start westend-ecosystem.config.js --only westend-transferToPolkadot,westend-transferToEthereum
```

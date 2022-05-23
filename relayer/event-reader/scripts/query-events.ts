#!/usr/bin/env node
import '@polkadot/api-augment'
import { ApiPromise, WsProvider, HttpProvider } from "@polkadot/api";
import yargs from "yargs"

import type { EventRecord } from '@polkadot/types/interfaces';

let main = async () => {
  const argv = yargs.options({
    "api": { type: "string", demandOption: true, describe: "API endpoint of parachain (HTTP)" },
    "block": { type: "string", demandOption: true, describe: "Block hash" },
  }).argv as any;

  let provider = new HttpProvider(argv.api);

  let api = await ApiPromise.create({
    provider,
  });

  const apiAt = await api.at(argv.block);
  const records = await apiAt.query.system.events<EventRecord[]>();
  let items = records
    .filter(({ event, phase }) => phase.isInitialization && (event.section == "basicOutboundChannel" || event.section == "incentivizedOutboundChannel") && event.method == "Committed" )
    .map(({event}) => {
      return {
        id: (event.section == "basicOutboundChannel") ? 0 : 1,
        hash: event.data[0].toHex(),
        data: event.data[1].toHex()
      }
    });
  console.log(JSON.stringify({items}))
  process.exit(0);
}


main().catch((error) => {
  console.error(error);
  process.exit(1);
});

import { ApiPromise, WsProvider } from "@polkadot/api";
import { createType } from "@polkadot/types";
import { Keyring } from "@polkadot/api";
import { bundle } from "@snowfork/snowbridge-types";
import yargs from "yargs"

import type { MultiLocation } from "@polkadot/types/interfaces/xcm/types";

const createTransferXcm = (
  api: ApiPromise,
  fromLocation: MultiLocation,
  toParaId: number,
  amount: number,
  toAddr: string
) => {
  return api.createType("Xcm", {
    WithdrawAsset: api.createType("XcmWithdrawAsset", {
      assets: [
        api.createType("MultiAsset", {
          ConcreteFungible: api.createType("MultiAssetConcreteFungible", {
            id: api.createType("MultiLocation", {
              X1: api.createType("Junction", "Parent"),
            }),
            amount: api.createType("Compact<U128>", 10_000_000),
          }),
        }),
        api.createType("MultiAsset", {
          ConcreteFungible: api.createType("MultiAssetConcreteFungible", {
            id: fromLocation,
            amount: api.createType("Compact<U128>", amount),
          }),
        }),
      ],
      effects: [
        api.createType("XcmOrder", {
          BuyExecution: api.createType("XcmOrderBuyExecution", {
            fees: api.createType("MultiAsset", "All"),
            weight: 0,
            debt: 3_000_000,
            haltOnError: false,
            xcm: [],
          }),
        }),
        api.createType("XcmOrder", {
          DepositReserveAsset: api.createType("XcmOrderDepositReserveAsset", {
            assets: [api.createType("MultiAsset", "All")],
            dest: api.createType("MultiLocation", {
              X2: [
                api.createType("Junction", "Parent"),
                api.createType("Junction", {
                  Parachain: api.createType("Compact<U32>", toParaId),
                }),
              ],
            }),
            effects: [
              api.createType("XcmOrder", {
                BuyExecution: api.createType("XcmOrderBuyExecution", {
                  fees: api.createType("MultiAsset", "All"),
                  weight: 0,
                  debt: 3_000_000,
                  haltOnError: false,
                  xcm: [],
                }),
              }),
              api.createType("XcmOrder", {
                DepositAsset: api.createType("XcmOrderDepositAsset", {
                  assets: [api.createType("MultiAsset", "All")],
                  dest: api.createType("MultiLocation", {
                    X1: api.createType("Junction", {
                      AccountId32: api.createType("AccountId32Junction", {
                        network: api.createType("NetworkId", "Any"),
                        id: toAddr,
                      }),
                    }),
                  }),
                }),
              }),
            ],
          }),
        }),
      ],
    }),
  });
};

let main = async () => {
  const argv = yargs.options({
    "key-uri": { type: "string", demandOption: true },
    "para-id": { type: "number", demandOption: true },
    recipient: { type: "string", demandOption: true },
    amount: { type: "number", demandOption: true },
  }).argv as any;

  let provider = new WsProvider("ws://127.0.0.1:11144");

  let api = await ApiPromise.create({
    provider,
    typesBundle: bundle as any,
  });

  const keyring = new Keyring({ type: "sr25519" });
  const alice = keyring.addFromUri(argv.keyUri);

  let assetId = api.createType("AssetId", "ETH");
  let location = api.createType("MultiLocation", {
    X1: api.createType("Junction", {
      GeneralKey: assetId.toHex(),
    }),
  });

  let xcm = createTransferXcm(
    api,
    location,
    argv.paraId,
    argv.amount,
    argv.recipient
  );

  let unsub = await api.tx.polkadotXcm
    .execute(xcm, 100000000)
    .signAndSend(alice, async (result) => {
      console.log(`Current status is ${result.status}`);

      if (result.status.isInBlock) {
        console.log(
          `Transaction included at blockHash ${result.status.asInBlock}`
        );
      } else if (result.status.isFinalized) {
        console.log(
          `Transaction finalized at blockHash ${result.status.asFinalized}`
        );
        unsub();
        await provider.disconnect();
      }
    });
};

main().catch((error) => {
  console.error(error);
  process.exit(1);
});

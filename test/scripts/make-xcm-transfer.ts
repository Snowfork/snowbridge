import { ApiPromise, WsProvider } from "@polkadot/api";
import { createType } from "@polkadot/types";
import { Keyring } from "@polkadot/api";
import { bundle } from "@snowfork/snowbridge-types";
import yargs from "yargs"

import type { XcmAssetId } from "@polkadot/types/interfaces/xcm/types";

const createTransferXcm = (
  api: ApiPromise,
  fromLocation: XcmAssetId,
  toParaId: number,
  amount: number,
  toAddr: string
) => {
  return api.createType("VersionedXcm", {
    V2: api.createType("XcmV2", [
      api.createType("InstructionV2", {
        BuyExecution: {
          fees: api.createType("MultiAssetV2", {
            id: api.createType("XcmAssetId", {
              Concrete: api.createType("MultiLocationV2", {
                parents: api.createType("u8", 1),
                interior: "Here"
              })
            }),
            fungibility: api.createType("FungibilityV2", {
              Fungible: api.createType("Compact<u128>", 3_000_000)
            })
          }),
          weightLimit: api.createType("WeightLimitV2", {
            Limited: api.createType("Compact<u64>", 3_000_000)
          }),
        }
      }),
      api.createType("InstructionV2", {
        WithdrawAsset: api.createType("MultiAssetsV2", [
          api.createType("MultiAssetV2", {
            id: fromLocation,
            fungibility: api.createType("FungibilityV2", {
              Fungible: api.createType("Compact<u128>", amount)
            })
          }),
        ])
      }),
      api.createType("InstructionV2", {
        DepositReserveAsset: {
          assets: api.createType("MultiAssetFilterV2", {
            Wild: api.createType("WildMultiAssetV2", "All")
          }),
          maxAssets: api.createType("u32", 1),
          dest: api.createType("MultiLocationV2", {
            parents: api.createType("u8", 1),
            interior: api.createType("JunctionsV2", {
              X1: api.createType("JunctionV2", {
                Parachain: api.createType("Compact<u32>", toParaId),
              }),
            }),
          }),
          xcm: api.createType("XcmV2", [
            api.createType("InstructionV2", {
              BuyExecution: {
                fees: api.createType("MultiAssetV2", {
                  id: api.createType("XcmAssetId", {
                    Concrete: api.createType("MultiLocationV2", {
                      parents: api.createType("u8", 1),
                      interior: api.createType("JunctionsV2", {
                        X1: api.createType("JunctionV2", {
                          Parachain: api.createType("Compact<u32>", toParaId),
                        }),
                      }),
                    })
                  }),
                  fungibility: api.createType("FungibilityV2", {
                    Fungible: api.createType("Compact<u128>", 3_000_000)
                  })
                }),
                weightLimit: api.createType("WeightLimitV2", {
                  Limited: api.createType("Compact<u64>", 3_000_000)
                }),
              }
            }),
            api.createType("InstructionV2", {
              DepositAsset: {
                assets: api.createType("MultiAssetFilterV2", {
                  Wild: api.createType("WildMultiAssetV2", "All")
                }),
                maxAssets: api.createType("u32", 1),
                beneficiary: api.createType("MultiLocationV2", {
                  parents: api.createType("u8", 0),
                  interior: api.createType("JunctionsV2", {
                    X1: api.createType("JunctionV2", {
                      AccountId32: {
                        network: api.createType("NetworkId", "Any"),
                        id: api.createType("AccountId", toAddr)
                      }
                    }),
                  }),
                }),
              }
            })
          ])
        }
      }),
    ])
  });
};

let main = async () => {
  const argv = yargs.options({
    "api": { type: "string", demandOption: true, describe: "API endpoint of source parachain" },
    "key-uri": { type: "string", demandOption: true, describe: "Account key for sending extrinsic" },
    "para-id": { type: "number", demandOption: true, describe: "Destination parachain" },
    recipient: { type: "string", demandOption: true, describe: "Destination account" },
    amount: { type: "number", demandOption: true, describe: "Amount to transfer" },
  }).argv as any;

  let provider = new WsProvider(argv.api);

  let api = await ApiPromise.create({
    provider,
    typesBundle: bundle as any,
  });

  const keyring = new Keyring({ type: "sr25519" });
  const alice = keyring.addFromUri(argv.keyUri);

  let assetId = api.createType("AssetId", "ETH");
  let location : XcmAssetId = api.createType('XcmAssetId', {
    Concrete: api.createType("MultiLocationV2", {
      parents: api.createType('u8', 0),
      interior: api.createType("JunctionsV2", {
        X1: api.createType("JunctionV2", {
          GeneralKey: assetId.toHex()
        })
      })
    })
  });

  let xcm = createTransferXcm(
    api,
    location,
    argv.paraId,
    argv.amount,
    argv.recipient
  );

  let unsub = await api.tx.polkadotXcm
    .execute(xcm, 100_000_000)
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

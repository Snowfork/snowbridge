import { ApiPromise, WsProvider, Keyring } from "@polkadot/api";
import { bundle } from "@snowfork/snowbridge-types";
import yargs from "yargs"

import type { MultiLocationV2 } from "@polkadot/types/interfaces/xcm/types";

const createTransferXcm = (
  api: ApiPromise,
  fromLocation: MultiLocationV2,
  toParaId: number,
  amount: number,
  toAddr: string
) => {
  return api.createType("VersionedXcm", {
    V2: [
      {
        WithdrawAsset: [
          {
            id: {
              Concrete: {
                parents: 1,
                interior: "Here"
              }
            },
            fungibility: {
              Fungible: 5_004_000_000
            }
          },
          {
            id: {
              Concrete: fromLocation
            },
            fungibility: {
              Fungible: amount
            }
          }
        ]
      },
      {
        BuyExecution: {
          fees: {
            id: {
              Concrete: {
                parents: 1,
                interior: "Here"
              }
            },
            fungibility: {
              Fungible: 4_000_000
            }
          },
          weightLimit: "Unlimited"
        }
      },
      {
        DepositReserveAsset: {
          assets: {
            Wild: "All"
          },
          maxAssets: 1,
          dest: {
            parents: 1,
            interior: {
              X1: {
                Parachain: toParaId
              }
            }
          },
          xcm: [
            {
              BuyExecution: {
                fees: {
                  id: {
                    Concrete: {
                      parents: 1,
                      interior: "Here"
                    }
                  },
                  fungibility: {
                    Fungible: 5_000_000_000
                  }
                },
                weightLimit: "Unlimited"
              }
            },
            {
              DepositAsset: {
                assets: {
                  Wild: "All"
                },
                maxAssets: 1,
                beneficiary: {
                  parents: 0,
                  interior: {
                    X1: {
                      AccountId32: {
                        network: "Any",
                        id: toAddr
                      }
                    }
                  }
                }
              }
            },
            "RefundSurplus"
          ]
        }
      },
      "RefundSurplus"
    ]
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
  let location : MultiLocationV2 = api.createType("MultiLocationV2", {
    parents: 0,
    interior: {
      X1: {
        GeneralKey: assetId.toHex()
      }
    }
  });

  let xcm = createTransferXcm(
    api,
    location,
    argv.paraId,
    argv.amount,
    argv.recipient
  );

  let call = api.tx.polkadotXcm.execute(xcm, 4_000_000);
  console.log("Encoded Xcm", xcm.toHex());
  console.log("Encoded Call", call.toHex());
  console.log("Human Call", JSON.stringify(call.toHuman(), null, 2));

  let unsub = await call.signAndSend(alice, async (result) => {
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

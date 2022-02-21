import { ApiPromise, Keyring, WsProvider } from "@polkadot/api";
import { ISubmittableResult } from "@polkadot/types/types";
import { assert } from "console";
import { exit } from "process";
import Web3 from "web3";
import yargs from "yargs";
import { createInterface } from "readline";

const rl = createInterface({ input: process.stdin, output: process.stdout });
const keyring = new Keyring({ type: "sr25519" });

const areYouSure = (prompt: string, yes: string): Promise<boolean> => {
  return new Promise<boolean>((ok) => {
    rl.question(prompt, (answer) => {
      ok(answer === yes);
    });
  });
};

const forceResetToFork = (
  api: ApiPromise,
  hash: string,
  user: string
): Promise<ISubmittableResult> => {
  const sudo = keyring.addFromUri(user);
  const call = api.tx.ethereumLightClient.forceResetToFork(
    api.createType("H256", hash)
  );

  return new Promise<ISubmittableResult>(async (ok, err) => {
    const unsub = await api.tx.sudo
      .sudo(call)
      .signAndSend(sudo, async (result) => {
        console.log(`Current status is ${result.status}`);

        if (result.isError) {
          err(result);
          unsub();
        } else if (result.status.isInBlock) {
          console.log(
            `Transaction included at blockHash ${result.status.asInBlock}`
          );
        } else if (result.status.isFinalized) {
          console.log(
            `Transaction finalized at blockHash ${result.status.asFinalized}`
          );
          ok(result);
          unsub();
        }
      });
  });
};

const fetchFinalized = async (api: ApiPromise) => {
  const key =
    "0xb76dae0be628ba37edd6eda726135ecc03675448006f828e6b077873c49b9733";
  const request: any = await api.rpc.state.getStorage(key);
  return api
    .createType("SnowbridgeEthereumHeaderHeaderId", request.toHex())
    .toJSON();
};

const fetchImported = async (api: ApiPromise, hash: string) => {
  const prefix =
    "0xb76dae0be628ba37edd6eda726135eccaf3385e35cc12fed4c74164ad01ecbbf";
  const key = prefix + hash.substring(2);
  const request: any = await api.rpc.state.getStorage(key);
  return api
    .createType(
      "Option<SnowbridgeEthereumLightClientStoredHeader>",
      request.toHex()
    )
    .toJSON();
};

const main = async () => {
  const argv = yargs.options({
    "eth-url": {
      type: "string",
      demandOption: true,
      describe: "Eth API endpoint",
    },
    "snowbridge-url": {
      type: "string",
      demandOption: true,
      describe: "API endpoint of source parachain",
    },
    blocks: {
      type: "number",
      demandOption: false,
      describe: "The amount of blocks to search.",
      default: 200,
    },
    "probe-from": {
      type: "string",
      demandOption: false,
      describe: "The ethereum block number or hash to start the search.",
      default: null,
    },
    "fix": {
      type: "string",
      demandOption: false,
      describe: "Fix the fork with the following user. e.g. '//Alice'",
      default: null,
    },
  }).argv as any;

  // intialize api clients
  const parachainApi = await ApiPromise.create({
    provider: new WsProvider(argv["snowbridge-url"]),
  });
  const ethApi = new Web3(
    new Web3.providers.WebsocketProvider(argv["eth-url"])
  );

  // get the current finalized block from the parachain
  const paraFinalized: any = await fetchFinalized(parachainApi);
  console.log("Parachain");
  console.log(`Number: ${paraFinalized.number}`);
  console.log(`Hash: ${paraFinalized.hash}.`);

  // get the block from ethereum
  const ethFinalized = await ethApi.eth.getBlock(paraFinalized.number, false);
  console.log("Ethereum");
  console.log(`Number: ${ethFinalized.number}`);
  console.log(`Hash: ${ethFinalized.hash}.`);

  // check if a fork exists
  assert(
    ethFinalized.number === paraFinalized.number,
    "Block numbers should always be the same."
  );
  if (ethFinalized.hash === paraFinalized.hash) {
    console.log("There is no fork.");
    process.exit(0);
  }

  // Walk backwards until we find a finalized block.
  console.log("Fork found");

  const startNumber = (
    await ethApi.eth.getBlock(
      argv["probe-from"] ?? ethFinalized.number - 1,
      false
    )
  ).number;
  const endNumber = startNumber - argv["blocks"];
  console.log(
    `Finding common ancestor between blocks ${endNumber} and ${startNumber}.`
  );

  for (let number = startNumber; endNumber < number; number--) {
    console.log(`Checking block number ${number}...`);
    const ethBlock = await ethApi.eth.getBlock(number, false);
    const paraBlock: any = await fetchImported(parachainApi, ethBlock.hash);
    if (paraBlock === null && !paraBlock.finalized) continue;

    assert(
      ethBlock.number === paraBlock.header.number,
      "Block numbers should always be the same."
    );
    console.log(
      `Common ancestor found at block number ${ethBlock.number} hash ${ethBlock.hash}`
    );
    console.log(`Parachain Finalized:        ${paraBlock.finalized}`);
    console.log(`Parachain Total Difficulty: ${paraBlock.totalDifficulty}`);
    console.log(`Ethereum Total Difficulty:  ${ethBlock.totalDifficulty}`);

    let fixWithUser: string = argv["fix"];
    if (fixWithUser !== null && fixWithUser !== "") {
      console.log(
        `Going to force reset to ${ethBlock.hash} with user ${fixWithUser}.`
      );
      if (!(await areYouSure("Are you sure? ", "yes"))) {
        exit(0);
      }

      await forceResetToFork(parachainApi, ethBlock.hash, fixWithUser);
    }

    process.exit(0);
  }

  console.log("No common ancestor found.");
  process.exit(1);
};

main().catch((error) => {
  console.error(error);
  process.exit(1);
});

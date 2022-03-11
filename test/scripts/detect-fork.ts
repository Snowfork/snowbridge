import { ApiPromise, Keyring, WsProvider } from "@polkadot/api";
import { ISubmittableResult } from "@polkadot/types/types";
import { assert } from "console";
import { exit } from "process";
import Web3 from "web3";
import yargs from "yargs";
import { createInterface } from "readline";
import * as https from "https";

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
  user: string,
  updates: any
): Promise<ISubmittableResult> => {
  const sudo = keyring.addFromUri(user);

  const calls = [
    api.tx.ethereumLightClient.forceResetToFork(api.createType("H256", hash)),
  ];

  if (updates.length > 0) {
    const items = [];
    for (const update of updates) {
      items.push([
        update.storageKey,
        api.createType("u64", update.nonce).toHex(true),
      ]);
    }
    calls.push(api.tx.system.setStorage(items));
  }

  const batch = api.tx.utility.batchAll(calls);

  return new Promise<ISubmittableResult>(async (ok, err) => {
    const unsub = await api.tx.sudo
      .sudo(batch)
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

const getContracts = (url: string): Promise<any> => {
  return new Promise<any>(function (ok, err) {
    const buffers: Array<Buffer> = [];
    https
      .get(url, (res) => {
        if (res.statusCode === 200) {
          res.on("data", (d) => {
            buffers.push(d);
          });
          res.on("end", () => {
            ok(JSON.parse(Buffer.concat(buffers).toString("utf8")));
          });
        } else err(res.statusMessage);
      })
      .on("error", (e) => {
        err(e);
      });
  });
};

const NONCES = [
  {
    name: "BasicInboundChannel",
    event: "MessageDispatched",
    storageKey:
      "0x684b82bef882079feeabe54a5bd7b94a718368a0ace36e2b1b8b6dbd7f8093c0",
  },
  {
    name: "IncentivizedInboundChannel",
    event: "MessageDispatched",
    storageKey:
      "0xf0f4d0b91e760c07da58bc0498033acb718368a0ace36e2b1b8b6dbd7f8093c0",
  },
  {
    name: "BasicOutboundChannel",
    event: "Message",
    storageKey:
      "0x664ff6e369f56e1c7deca5487e631a5c718368a0ace36e2b1b8b6dbd7f8093c0",
  },
  {
    name: "IncentivizedOutboundChannel",
    event: "Message",
    storageKey:
      "0x557df379daaf1cd514a7452dcbf6fccc718368a0ace36e2b1b8b6dbd7f8093c0",
  },
];

const fetchEthNonces = async (
  contractsConfig: any,
  ethApi: Web3,
  commonAnscestorBlockNumber: number,
  descendantsUntilFinalized: number
): Promise<any> => {
  const pastEvents = {};
  const nonces = {};
  for (const nonce of NONCES) {
    var contract = new ethApi.eth.Contract(
      contractsConfig[nonce.name].abi,
      contractsConfig[nonce.name].address
    );
    // get all nonce changing events that happened after the new finalized
    pastEvents[nonce.name] = contract.getPastEvents(nonce.event, {
      fromBlock: commonAnscestorBlockNumber - descendantsUntilFinalized,
      toBlock: "latest",
    });
    nonces[nonce.name] = contract.methods.nonce().call();
  }

  await Promise.all(Object.values(pastEvents).concat(Object.values(nonces)));

  const result = {};
  for (const nonce of NONCES) {
    const events = await pastEvents[nonce.name];
    // take the first event if there are any else get take the current nonce.
    if (events.length > 0) {
      result[nonce.name] = Number(events[0].returnValues["nonce"]) - 1;
    } else {
      result[nonce.name] = Number(await nonces[nonce.name]);
    }
  }
  return result;
};

const fetchParachainNonces = async (parachainApi: ApiPromise): Promise<any> => {
  const promises = {};
  for (const nonce of NONCES) {
    promises[nonce.name] =
      parachainApi.query[
        `${nonce.name[0].toLowerCase()}${nonce.name.substring(1)}`
      ].nonce();
  }

  await Promise.all(Object.values(promises));

  const result = {};
  for (const nonce of NONCES) {
    result[nonce.name] = (await promises[nonce.name]).toNumber();
  }
  return result;
};

const generateUpdates = (ethNonces, parachainNonces) => {
  const result = [];
  if (parachainNonces.BasicInboundChannel !== ethNonces.BasicOutboundChannel) {
    result.push({
      name: NONCES[0].name,
      storageKey: NONCES[0].storageKey,
      nonce: ethNonces.BasicOutboundChannel,
    });
  }
  if (
    parachainNonces.IncentivizedInboundChannel !==
    ethNonces.IncentivizedOutboundChannel
  ) {
    result.push({
      name: NONCES[1].name,
      storageKey: NONCES[1].storageKey,
      nonce: ethNonces.IncentivizedOutboundChannel,
    });
  }
  return result;
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
    "contracts-url": {
      type: "string",
      demandOption: true,
      describe: "url to contracts.json file.",
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
    fix: {
      type: "string",
      demandOption: false,
      describe: "Fix the fork with the following user. e.g. '//Alice'",
      default: null,
    },
    "descendants-until-finalized": {
      type: "number",
      demandOption: false,
      describe:
        "The number of descendants until a block is considered finalized.",
      default: 16,
    },
  }).argv as any;

  console.log("Fetching contracts.");
  const contractsConfig = await getContracts(argv["contracts-url"]);
  console.log("Fetch complete.");

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
    if (paraBlock === null || !paraBlock.finalized) continue;

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

    console.log("Checking nonces.");
    const parachainNonces = await fetchParachainNonces(parachainApi);
    const ethNonces = await fetchEthNonces(
      contractsConfig.contracts,
      ethApi,
      ethBlock.number,
      argv["descendants-until-finalized"]
    );

    console.log("Nonces                Parachain -> ETH");
    console.log(
      `Basic Channel:        ${parachainNonces.BasicOutboundChannel} -> ${ethNonces.BasicInboundChannel}`
    );
    console.log(
      `Incentivized Channel: ${parachainNonces.IncentivizedOutboundChannel} -> ${ethNonces.IncentivizedInboundChannel}`
    );

    console.log("Nonces                Parachain <- ETH");
    console.log(
      `Basic Channel:        ${parachainNonces.BasicInboundChannel} <- ${ethNonces.BasicOutboundChannel}`
    );
    console.log(
      `Incentivized Channel: ${parachainNonces.IncentivizedInboundChannel} <- ${ethNonces.IncentivizedOutboundChannel}`
    );

    let fixWithUser: string = argv["fix"];
    if (fixWithUser !== null && fixWithUser !== "") {
      console.log(`Fixing fork`);
      const updates = generateUpdates(ethNonces, parachainNonces);
      console.log(
        `Going to force reset to number ${ethBlock.number} hash ${ethBlock.hash} with user ${fixWithUser}.`
      );
      for (const update of updates) {
        console.log(
          `Going to reset ${update.name}'s nonce to ${update.nonce}.`
        );
      }
      if (!(await areYouSure("Are you sure? ", "yes"))) {
        exit(0);
      }
      await forceResetToFork(parachainApi, ethBlock.hash, fixWithUser, updates);
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

import fs from "node:fs";
import net from "node:net";
import { spawn } from "node:child_process";
import { Context } from "mocha";
import { AbortController, AbortSignal } from "@chainsafe/abort-controller";
import childProcess from "node:child_process";
import { fromHexString } from "@chainsafe/ssz";
import { WinstonLogger, LogLevel, TransportType, TransportOpts, TimestampFormat, fromTransportOpts, sleep, TimestampFormatCode } from "@chainsafe/lodestar-utils";
import { SLOTS_PER_EPOCH } from "@chainsafe/lodestar-params";
import { Epoch } from "@chainsafe/lodestar-types";
import { bellatrix } from "@chainsafe/lodestar-beacon-state-transition";
import { ChainEvent } from "@chainsafe/lodestar/lib/chain/emitter";
import { ILogger, RecursivePartial } from "@chainsafe/lodestar-utils";
import deepmerge from "deepmerge";
import tmp from "tmp";
import PeerId from "peer-id";
import { createEnr } from "@chainsafe/lodestar-cli/lib/config";
import { config as minimalConfig } from "@chainsafe/lodestar-config/default";
import { createIBeaconConfig, createIChainForkConfig, IChainConfig } from "@chainsafe/lodestar-config";
import { LevelDbController } from "@chainsafe/lodestar-db";
import { phase0 } from "@chainsafe/lodestar-types";
import { BeaconStateAllForks } from "@chainsafe/lodestar-beacon-state-transition";
import { BeaconNode } from "@chainsafe/lodestar/lib/node";
import { createNodeJsLibp2p } from "@chainsafe/lodestar/lib/network/nodejs";
import { createPeerId } from "@chainsafe/lodestar/lib/network";
import { defaultNetworkOptions } from "@chainsafe/lodestar/lib/network/options";
import { initDevState } from "@chainsafe/lodestar/lib/node/utils/state";
import { IBeaconNodeOptions } from "@chainsafe/lodestar/lib/node/options";
import { defaultOptions } from "@chainsafe/lodestar/lib/node/options";
import { BeaconDb } from "@chainsafe/lodestar/lib/db";
import { InteropStateOpts } from "@chainsafe/lodestar/lib/node/utils/interop/state";
import { isPlainObject } from "@chainsafe/lodestar-utils";
import { RestApiOptions } from "@chainsafe/lodestar/lib/api";
import { interopSecretKey } from "@chainsafe/lodestar-beacon-state-transition";
import {
  SlashingProtection,
  Validator,
  Signer,
  SignerType,
} from "@chainsafe/lodestar-validator";
import { SecretKey } from "@chainsafe/bls";
import { Eth1Provider } from "@chainsafe/lodestar/lib";
import {
  computeEpochAtSlot,
  computeStartSlotAtEpoch,
  allForks,
  CachedBeaconStateAllForks,
  beforeProcessEpoch,
} from "@chainsafe/lodestar-beacon-state-transition";
import { IBeaconConfig } from "@chainsafe/lodestar-config";
import { IProtoBlock } from "@chainsafe/lodestar-fork-choice";
import { SLOTS_PER_HISTORICAL_ROOT } from "@chainsafe/lodestar-params";
import { Slot } from "@chainsafe/lodestar-types";
import { Checkpoint } from "@chainsafe/lodestar-types/phase0";
import { mapValues } from "@chainsafe/lodestar-utils";
import { toHexString } from "@chainsafe/ssz";
import { linspace } from "@chainsafe/lodestar/lib/util/numpy";
import { RegenCaller } from "@chainsafe/lodestar/lib/chain/regen";
import { ZERO_HASH } from "@chainsafe/lodestar/lib/constants";


/* eslint-disable no-console, @typescript-eslint/naming-convention, quotes */

// BELLATRIX_EPOCH will happen at 2 sec * 8 slots = 16 sec
// 10 ttd / 2 difficulty per block = 5 blocks * 5 sec = 25 sec
const terminalTotalDifficultyPreMerge = 10;
const TX_SCENARIOS = process.env.TX_SCENARIOS?.split(",") || [];
export const logFilesDir = "test-logs";

const jwtSecretHex = "0xdc6457099f127cf0bac78de8b297df04951281909db4f58b43def7c7151e765d";

const fromAccount = "0x89b4AB1eF20763630df9743ACF155865600daFF2"
const toAccount = "0xbe68fc2d8249eb60bfcf0e71d5a0d2f2e292c4ed"

var jsonRpcUrl = "";
var engineApiUrl = "";

async function startBeaconNode() {
  this.timeout("30min");

  const jsonRpcPort = process.env.ETH_PORT;
  const enginePort = process.env.ENGINE_PORT;

  jsonRpcUrl = `http://localhost:${jsonRpcPort}`;
  engineApiUrl = `http://localhost:${enginePort}`;

  runPostMerge();
}

// Ref: https://notes.ethereum.org/@9AeMAlpyQYaAAyuj47BzRw/rkwW3ceVY
// Build geth from source at branch https://github.com/ethereum/go-ethereum/pull/23607
// $ ./go-ethereum/build/bin/geth --catalyst --datadir "~/ethereum/taunus" init genesis.json
// $ ./build/bin/geth --catalyst --http --ws -http.api "engine" --datadir "~/ethereum/taunus" console
async function runEL(elScript: string, ttd: number): Promise<{ genesisBlockHash: string }> {
  if (!process.env.EL_BINARY_DIR || !process.env.EL_SCRIPT_DIR || !process.env.ENGINE_PORT || !process.env.ETH_PORT) {
    throw Error(
      `EL ENV must be provided, EL_BINARY_DIR: ${process.env.EL_BINARY_DIR}, EL_SCRIPT_DIR: ${process.env.EL_SCRIPT_DIR}, ENGINE_PORT: ${process.env.ENGINE_PORT}, ETH_PORT: ${process.env.ETH_PORT}`
    );
  }

  // Wait for Geth to be online
  const controller = new AbortController();
  await waitForELOnline(jsonRpcUrl, controller.signal);

  // Fetch genesis block hash
  const genesisBlockHash = await getGenesisBlockHash({ providerUrl: engineApiUrl, jwtSecretHex }, controller.signal);
  return { genesisBlockHash };
}

async function runPostMerge() {
  console.log("\n\nPost-merge, run for a few blocks\n\n");
  const { genesisBlockHash } = await runEL("post-merge.sh", 0);
  await runNodeWithEL({
    genesisBlockHash,
    bellatrixEpoch: 0,
    ttd: BigInt(0),
    testName: "post-merge",
  });
}

async function runNodeWithEL(
  {
    genesisBlockHash,
    bellatrixEpoch,
    ttd,
    testName,
  }: { genesisBlockHash: string; bellatrixEpoch: Epoch; ttd: bigint; testName: string }
): Promise<void> {
  const validatorClientCount = 1;
  const validatorsPerClient = 32;
  const event = ChainEvent.finalized;

  const testParams: Pick<IChainConfig, "SECONDS_PER_SLOT"> = {
    SECONDS_PER_SLOT: 2,
  };

  // Should reach justification in 6 epochs max.
  // Merge block happens at epoch 2 slot 4. Then 4 epochs to finalize
  const expectedEpochsToFinish = 6;
  // 1 epoch of margin of error
  const epochsOfMargin = 1;
  const timeoutSetupMargin = 5 * 1000; // Give extra 5 seconds of margin

  // delay a bit so regular sync sees it's up to date and sync is completed from the beginning
  const genesisSlotsDelay = 3;

  const timeout =
    ((epochsOfMargin + expectedEpochsToFinish) * SLOTS_PER_EPOCH + genesisSlotsDelay) *
    testParams.SECONDS_PER_SLOT *
    1000;

  this.timeout(timeout + 2 * timeoutSetupMargin);

  const genesisTime = Math.floor(Date.now() / 1000) + genesisSlotsDelay * testParams.SECONDS_PER_SLOT;

  const testLoggerOpts: TestLoggerOpts = {
    logLevel: LogLevel.info,
    logFile: `${logFilesDir}/merge-interop-${testName}.log`,
    timestampFormat: {
      format: TimestampFormatCode.EpochSlot,
      genesisTime,
      slotsPerEpoch: SLOTS_PER_EPOCH,
      secondsPerSlot: testParams.SECONDS_PER_SLOT,
    },
  };
  const loggerNodeA = testLogger("Node-A", testLoggerOpts);

  const bn = await getDevBeaconNode({
    params: {
      ...testParams,
      ALTAIR_FORK_EPOCH: 0,
      BELLATRIX_FORK_EPOCH: bellatrixEpoch,
      TERMINAL_TOTAL_DIFFICULTY: ttd,
    },
    options: {
      api: { rest: { enabled: true } as RestApiOptions },
      sync: { isSingleNode: true },
      network: { allowPublishToZeroPeers: true, discv5: null },
      // Now eth deposit/merge tracker methods directly available on engine endpoints
      eth1: { enabled: true, providerUrls: [engineApiUrl] },
      executionEngine: { urls: [engineApiUrl], jwtSecretHex },
    },
    validatorCount: validatorClientCount * validatorsPerClient,
    logger: loggerNodeA,
    genesisTime,
    eth1BlockHash: fromHexString(genesisBlockHash),
  });

  // afterEachCallbacks.push(async function () {
  // await bn.close();
  //  await sleep(1000);
  //});

  const stopInfoTracker = simTestInfoTracker(bn, loggerNodeA);

  const { validators } = await getAndInitDevValidators({
    node: bn,
    validatorsPerClient,
    validatorClientCount,
    startIndex: 0,
    // At least one sim test must use the REST API for beacon <-> validator comms
    useRestApi: true,
    testLoggerOpts,
    // TODO test merge-interop with remote;
  });

  //afterEachCallbacks.push(async function () {
  // await Promise.all(validators.map((v) => v.stop()));
  //});

  // await Promise.all(validators.map((v) => v.start()));

  if (TX_SCENARIOS.includes("simple")) {
    const balance = await getBalance(jsonRpcUrl, toAccount);
    console.log("balance before is " + balance);
    console.log("===========================================");

    console.log("fromAccount");
    console.log(fromAccount);
    console.log("toAccount");
    console.log(toAccount);
    // If bellatrixEpoch > 0, this is the case of pre-merge transaction submission on EL pow
    await sendTransaction(jsonRpcUrl, {
      from: fromAccount,
      to: toAccount,
      gas: "0x76c0",
      gasPrice: "0x9184e72a000",
      value: "0x9184e72a",
    });
  }

  await new Promise<void>((resolve, reject) => {
    // Play TX_SCENARIOS
    bn.chain.emitter.on(ChainEvent.clockSlot, async (slot) => {
      if (slot < 2) return;
      switch (slot) {
        // If bellatrixEpoch > 0, this is the case of pre-merge transaction confirmation on EL pow
        case 2:
          if (TX_SCENARIOS.includes("simple")) {
            const balance = await getBalance(jsonRpcUrl, toAccount);
            console.log("balance after is " + balance);
            console.log("===========================================");
            console.log("expected balance after is 0x9184e72a");
            console.log("===========================================");

            const balancefromAccount = await getBalance(jsonRpcUrl, fromAccount);
            console.log("balance after is " + balancefromAccount);
            console.log("===========================================");
            // if (balance !== "0x9184e72a") reject("Invalid Balance");
          }
          break;

        // By this slot, ttd should be reached and merge complete
        case Number(ttd) + 3: {
          const headState = bn.chain.getHeadState();
          const isMergeTransitionComplete =
            bellatrix.isBellatrixStateType(headState) && bellatrix.isMergeTransitionComplete(headState);
          if (!isMergeTransitionComplete) reject("Merge not completed");

          // Send another tx post-merge, total amount in destination account should be double after this is included in chain
          if (TX_SCENARIOS.includes("simple")) {
            await sendTransaction(jsonRpcUrl, {
              from: fromAccount,
              to: toAccount,
              gas: "0x76c0",
              gasPrice: "0x9184e72a000",
              value: "0x9184e72a",
            });
          }
          break;
        }

        default:
      }
    });

    bn.chain.emitter.on(ChainEvent.finalized, (checkpoint) => {
      // Resolve only if the finalized checkpoint includes execution payload
      const finalizedBlock = bn.chain.forkChoice.getBlock(checkpoint.root);
      if (finalizedBlock?.executionPayloadBlockHash !== null) {
        console.log(`\nGot event ${event}, stopping validators and nodes\n`);
        //resolve();
      }
    });
  });

  // Stop chain and un-subscribe events so the execution engine won't update it's head
  // Allow some time to broadcast finalized events and complete the importBlock routine
  await Promise.all(validators.map((v) => v.stop()));
  await bn.close();
  await sleep(500);

  // Assertions to make sure the end state is good
  // 1. The proper head is set
  const rpc = new Eth1Provider({ DEPOSIT_CONTRACT_ADDRESS: ZERO_HASH }, { providerUrls: [engineApiUrl] });
  const consensusHead = bn.chain.forkChoice.getHead();
  const executionHeadBlockNumber = await rpc.getBlockNumber();
  const executionHeadBlock = await rpc.getBlockByNumber(executionHeadBlockNumber);
  if (!executionHeadBlock) throw Error("Execution has not head block");
  if (consensusHead.executionPayloadBlockHash !== executionHeadBlock.hash) {
    throw Error(
      "Consensus head not equal to execution head: " +
      JSON.stringify({
        executionHeadBlockNumber,
        executionHeadBlockHash: executionHeadBlock.hash,
        consensusHeadExecutionPayloadBlockHash: consensusHead.executionPayloadBlockHash,
        consensusHeadSlot: consensusHead.slot,
      })
    );
  }

  if (TX_SCENARIOS.includes("simple")) {
    const balance = await getBalance(jsonRpcUrl, "0xafa3f8684e54059998bc3a7b0d2b0da075154d66");
    // 0x12309ce54 = 2 * 0x9184e72a
    if (balance !== "0x12309ce54") throw Error("Invalid Balance");
  }

  // wait for 1 slot to print current epoch stats
  await sleep(1 * bn.config.SECONDS_PER_SLOT * 1000);
  stopInfoTracker();
  console.log("\n\nDone\n\n");
}


async function waitForELOnline(url: string, signal: AbortSignal): Promise<void> {
  for (let i = 0; i < 60; i++) {
    try {
      console.log("Waiting for EL online...");
      await shell(
        `curl -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"net_version","params":[],"id":67}' ${url}`
      );

      console.log("Waiting for few seconds for EL to fully setup, for e.g. unlock the account...");
      await sleep(5000, signal);
      return; // Done
    } catch (e) {
      await sleep(1000, signal);
    }
  }
  throw Error("EL not online in 60 seconds");
}

async function waitForELOffline(): Promise<void> {
  const port = 30303;

  for (let i = 0; i < 60; i++) {
    console.log("Waiting for EL offline...");
    const isInUse = await isPortInUse(port);
    if (!isInUse) {
      return;
    }
    await sleep(1000);
  }
  throw Error("EL not offline in 60 seconds");
}

async function isPortInUse(port: number): Promise<boolean> {
  return await new Promise<boolean>((resolve, reject) => {
    const server = net.createServer();
    server.once("error", function (err) {
      if (((err as unknown) as { code: string }).code === "EADDRINUSE") {
        resolve(true);
      } else {
        reject(err);
      }
    });

    server.once("listening", function () {
      // close the server if listening doesn't fail
      server.close(() => {
        resolve(false);
      });
    });

    server.listen(port);
  });
}

async function getGenesisBlockHash(
  { providerUrl, jwtSecretHex }: { providerUrl: string; jwtSecretHex?: string },
  signal: AbortSignal
): Promise<string> {
  const eth1Provider = new Eth1Provider(
    ({ DEPOSIT_CONTRACT_ADDRESS: ZERO_HASH } as Partial<IChainConfig>) as IChainConfig,
    { providerUrls: [providerUrl] },
    signal
  );

  const genesisBlock = await eth1Provider.getBlockByNumber(0);
  if (!genesisBlock) {
    throw Error("No genesis block available");
  }

  return genesisBlock.hash;
}

async function sendTransaction(url: string, transaction: Record<string, unknown>): Promise<void> {
  const response: string = await shell(
    `curl -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"eth_sendTransaction","params":[${JSON.stringify(
      transaction
    )}],"id":67}' ${url}`
  );
  console.log("RESPONSE sendTransaction***********");
  console.log(response);
}

async function getBalance(url: string, account: string): Promise<string> {
  const response: string = await shell(
    `curl -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"eth_getBalance","params":["${account}","latest"],"id":67}' ${url}`
  );
  console.log("RESPONSE getBalance***********");
  console.log(response);
  const { result } = (JSON.parse(response) as unknown) as Record<string, string>;
  return result;
}

/**
 * If timeout is greater than 0, the parent will send the signal
 * identified by the killSignal property (the default is 'SIGTERM')
 * if the child runs longer than timeout milliseconds.
 */
const defaultTimeout = 15 * 60 * 1000; // ms

/**
 * Run arbitrary commands in a shell
 * If the child process exits with code > 0, rejects
 */
async function shell(
  cmd: string | string[],
  options?: { timeout?: number; maxBuffer?: number; signal?: AbortSignal; pipeToProcess?: boolean }
): Promise<string> {
  const timeout = options?.timeout ?? defaultTimeout;
  const maxBuffer = options?.maxBuffer;
  const cmdStr = Array.isArray(cmd) ? cmd.join(" ") : cmd;

  return new Promise((resolve, reject) => {
    const proc = childProcess.exec(cmdStr, { timeout, maxBuffer }, (err, stdout) => {
      if (err) {
        reject(err);
      } else {
        resolve(stdout.trim());
      }
    });

    if (options?.pipeToProcess) {
      proc.stdout?.pipe(process.stdout);
      proc.stderr?.pipe(process.stderr);
    }

    if (options?.signal) {
      options.signal.addEventListener(
        "abort",
        () => {
          proc.kill("SIGKILL");
        },
        { once: true }
      );
    }
  });
}

export async function getDevBeaconNode(
  opts: {
    params: Partial<IChainConfig>;
    options?: RecursivePartial<IBeaconNodeOptions>;
    validatorCount?: number;
    logger?: ILogger;
    peerId?: PeerId;
    peerStoreDir?: string;
    anchorState?: BeaconStateAllForks;
    wsCheckpoint?: phase0.Checkpoint;
  } & InteropStateOpts
): Promise<BeaconNode> {
  const { params, validatorCount = 8, peerStoreDir } = opts;
  let { options = {}, logger, peerId } = opts;

  if (!peerId) peerId = await createPeerId();
  const tmpDir = tmp.dirSync({ unsafeCleanup: true });
  const config = createIChainForkConfig({ ...minimalConfig, ...params });
  logger = logger ?? testLogger();

  const db = new BeaconDb({ config, controller: new LevelDbController({ name: tmpDir.name }, { logger }) });
  await db.start();

  const libp2p = await createNodeJsLibp2p(
    peerId,
    {
      discv5: {
        enabled: false,
        enr: createEnr(peerId),
        bindAddr: options.network?.discv5?.bindAddr || "/ip4/127.0.0.1/udp/0",
        bootEnrs: [],
      },
      localMultiaddrs: options.network?.localMultiaddrs || ["/ip4/127.0.0.1/tcp/0"],
      targetPeers: defaultNetworkOptions.targetPeers,
      maxPeers: defaultNetworkOptions.maxPeers,
    },
    { disablePeerDiscovery: true, peerStoreDir }
  );

  options = deepmerge(
    // This deepmerge should NOT merge the array with the defaults but overwrite them
    defaultOptions,
    deepmerge(
      // This deepmerge should merge all the array elements of the api options with the
      // dev defaults that we wish, especially for the api options
      {
        db: { name: tmpDir.name },
        eth1: { enabled: false },
        api: { rest: { api: ["beacon", "config", "events", "node", "validator"], port: 19596 } },
        metrics: { enabled: false },
        network: { discv5: null },
      } as Partial<IBeaconNodeOptions>,
      options
    ),
    {
      arrayMerge: overwriteTargetArrayIfItems,
      isMergeableObject: isPlainObject,
    }
  );

  const state = opts.anchorState || (await initDevState(config, db, validatorCount, opts));
  const beaconConfig = createIBeaconConfig(config, state.genesisValidatorsRoot);
  return await BeaconNode.init({
    opts: options as IBeaconNodeOptions,
    config: beaconConfig,
    db,
    logger,
    libp2p,
    anchorState: state,
    wsCheckpoint: opts.wsCheckpoint,
  });
}

function overwriteTargetArrayIfItems(target: unknown[], source: unknown[]): unknown[] {
  if (source.length === 0) {
    return target;
  }
  return source;
}



export type TestLoggerOpts = {
  logLevel?: LogLevel;
  logFile?: string;
  timestampFormat?: TimestampFormat;
};

/**
 * Run the test with ENVs to control log level:
 * ```
 * LOG_LEVEL=debug mocha .ts
 * DEBUG=1 mocha .ts
 * VERBOSE=1 mocha .ts
 * ```
 */
export function testLogger(module?: string, opts?: TestLoggerOpts): WinstonLogger {
  const transports: TransportOpts[] = [
    { type: TransportType.console, level: getLogLevelFromEnvs() || opts?.logLevel || LogLevel.error },
  ];
  if (opts?.logFile) {
    transports.push({ type: TransportType.file, filename: opts.logFile, level: LogLevel.debug });
  }

  return new WinstonLogger({ module, ...opts }, transports);
}

function getLogLevelFromEnvs(): LogLevel | null {
  if (process.env["LOG_LEVEL"]) return process.env["LOG_LEVEL"] as LogLevel;
  if (process.env["DEBUG"]) return LogLevel.debug;
  if (process.env["VERBOSE"]) return LogLevel.verbose;
  return null;
}


export async function getAndInitDevValidators({
  node,
  validatorsPerClient = 8,
  validatorClientCount = 1,
  startIndex = 0,
  useRestApi,
  testLoggerOpts,
  externalSignerUrl,
}: {
  node: BeaconNode;
  validatorsPerClient: number;
  validatorClientCount: number;
  startIndex: number;
  useRestApi?: boolean;
  testLoggerOpts?: TestLoggerOpts;
  externalSignerUrl?: string;
}): Promise<{ validators: Validator[]; secretKeys: SecretKey[] }> {
  const validators: Promise<Validator>[] = [];
  const secretKeys: SecretKey[] = [];

  for (let clientIndex = 0; clientIndex < validatorClientCount; clientIndex++) {
    const startIndexVc = startIndex + clientIndex * validatorsPerClient;
    const endIndex = startIndexVc + validatorsPerClient - 1;
    const logger = testLogger(`Vali ${startIndexVc}-${endIndex}`, testLoggerOpts);
    const tmpDir = tmp.dirSync({ unsafeCleanup: true });
    const dbOps = {
      config: node.config,
      controller: new LevelDbController({ name: tmpDir.name }, { logger }),
    };
    const slashingProtection = new SlashingProtection(dbOps);

    const secretKeysValidator = Array.from({ length: validatorsPerClient }, (_, i) => interopSecretKey(i + startIndexVc));
    secretKeys.push(...secretKeysValidator);

    const signers = externalSignerUrl
      ? secretKeysValidator.map(
        (secretKey): Signer => ({
          type: SignerType.Remote,
          externalSignerUrl,
          pubkeyHex: secretKey.toPublicKey().toHex(),
        })
      )
      : secretKeysValidator.map(
        (secretKey): Signer => ({
          type: SignerType.Local,
          secretKey,
        })
      );

    validators.push(
      Validator.initializeFromBeaconNode({
        dbOps,
        api: useRestApi ? getNodeApiUrl(node) : node.api,
        slashingProtection,
        logger,
        signers,
      })
    );
  }

  return {
    validators: await Promise.all(validators),
    // Return secretKeys to start the externalSigner
    secretKeys,
  };
}

function getNodeApiUrl(node: BeaconNode): string {
  const host = node.opts.api.rest.host || "127.0.0.1";
  const port = node.opts.api.rest.port || 19596;
  return `http://${host}:${port}`;
}


export function simTestInfoTracker(bn: BeaconNode, logger: ILogger): () => void {
  let lastSeenEpoch = 0;

  const attestationsPerBlock = new Map<Slot, number>();
  const inclusionDelayPerBlock = new Map<Slot, number>();
  const prevParticipationPerEpoch = new Map<Epoch, number>();
  const currParticipationPerEpoch = new Map<Epoch, number>();

  async function onHead(head: IProtoBlock): Promise<void> {
    const slot = head.slot;

    // For each block
    // Check if there was a proposed block and how many attestations it includes
    const block = await bn.chain.getCanonicalBlockAtSlot(head.slot);
    if (block) {
      const bits = sumAttestationBits(block.message);
      const inclDelay = avgInclusionDelay(block.message);
      attestationsPerBlock.set(slot, bits);
      inclusionDelayPerBlock.set(slot, inclDelay);
      logger.info("> Block attestations", { slot, bits, inclDelay });
    }
  }

  function logParticipation(state: CachedBeaconStateAllForks): void {
    // Compute participation (takes 5ms with 64 validators)
    // Need a CachedBeaconStateAllForks where (state.slot + 1) % SLOTS_EPOCH == 0
    const epochProcess = beforeProcessEpoch(state);
    const epoch = computeEpochAtSlot(state.slot);

    const prevParticipation =
      epochProcess.prevEpochUnslashedStake.targetStakeByIncrement / epochProcess.totalActiveStakeByIncrement;
    const currParticipation =
      epochProcess.currEpochUnslashedTargetStakeByIncrement / epochProcess.totalActiveStakeByIncrement;
    prevParticipationPerEpoch.set(epoch - 1, prevParticipation);
    currParticipationPerEpoch.set(epoch, currParticipation);
    logger.info("> Participation", {
      slot: `${state.slot}/${computeEpochAtSlot(state.slot)}`,
      prev: prevParticipation,
      curr: currParticipation,
    });
  }

  async function onCheckpoint(checkpoint: Checkpoint): Promise<void> {
    // Skip epochs on duplicated checkpoint events
    if (checkpoint.epoch <= lastSeenEpoch) return;
    lastSeenEpoch = checkpoint.epoch;

    // Recover the pre-epoch transition state, use any random caller for regen
    const checkpointState = await bn.chain.regen.getCheckpointState(checkpoint, RegenCaller.onForkChoiceFinalized);
    const lastSlot = computeStartSlotAtEpoch(checkpoint.epoch) - 1;
    const lastStateRoot = checkpointState.stateRoots.get(lastSlot % SLOTS_PER_HISTORICAL_ROOT);
    const lastState = await bn.chain.regen.getState(toHexString(lastStateRoot), RegenCaller.onForkChoiceFinalized);
    logParticipation(lastState);
  }

  bn.chain.emitter.on(ChainEvent.forkChoiceHead, onHead);
  bn.chain.emitter.on(ChainEvent.checkpoint, onCheckpoint);

  return function stop() {
    bn.chain.emitter.off(ChainEvent.forkChoiceHead, onHead);
    bn.chain.emitter.off(ChainEvent.checkpoint, onCheckpoint);

    // Write report
    console.log("\nEnd of sim test report\n");
    printEpochSlotGrid(attestationsPerBlock, bn.config, "Attestations per block");
    printEpochSlotGrid(inclusionDelayPerBlock, bn.config, "Inclusion delay per block");
    printEpochGrid({ curr: currParticipationPerEpoch, prev: prevParticipationPerEpoch }, "Participation per epoch");
  };
}

function sumAttestationBits(block: allForks.BeaconBlock): number {
  return Array.from(block.body.attestations).reduce(
    (total, att) => total + att.aggregationBits.getTrueBitIndexes().length,
    0
  );
}

function avgInclusionDelay(block: allForks.BeaconBlock): number {
  const inclDelay = Array.from(block.body.attestations).map((att) => block.slot - att.data.slot);
  return avg(inclDelay);
}

function avg(arr: number[]): number {
  return arr.length === 0 ? 0 : arr.reduce((p, c) => p + c, 0) / arr.length;
}

/**
 * Print a table grid of (Y) epoch / (X) slot_per_epoch
 */
function printEpochSlotGrid<T>(map: Map<Slot, T>, config: IBeaconConfig, title: string): void {
  const lastSlot = Array.from(map.keys())[map.size - 1];
  const lastEpoch = computeEpochAtSlot(lastSlot);
  const rowsByEpochBySlot = linspace(0, lastEpoch).map((epoch) => {
    const slots = linspace(epoch * SLOTS_PER_EPOCH, (epoch + 1) * SLOTS_PER_EPOCH - 1);
    return slots.map((slot) => formatValue(map.get(slot)));
  });
  console.log(renderTitle(title));
  console.table(rowsByEpochBySlot);
}

/**
 * Print a table grid of (Y) maps object key / (X) epoch
 */
function printEpochGrid(maps: Record<string, Map<Epoch, number>>, title: string): void {
  const lastEpoch = Object.values(maps).reduce((max, map) => {
    const epoch = Array.from(map.keys())[map.size - 1];
    return epoch > max ? epoch : max;
  }, 0);
  const epochGrid = linspace(0, lastEpoch).map((epoch) =>
    mapValues(maps, (val, key) => formatValue(maps[key].get(epoch)))
  );
  console.log(renderTitle(title));
  console.table(epochGrid);
}

function renderTitle(title: string): string {
  return `${title}\n${"=".repeat(title.length)}`;
}

/** Represent undefined values as "-" to make the tables shorter. The word "undefined" is too wide */
function formatValue<T>(val: T | undefined): T | string {
  return val === undefined ? "-" : val;
}

const main = async () => {
  startBeaconNode();
};

main().catch((error) => {
  console.error(error);
  process.exit(1);
}).finally(async () => {
  console.log("Done");
  await shell(`rm -rf lodestar-test-merge-interop`);
});


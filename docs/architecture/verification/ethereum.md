# Ethereum

We have implemented a Proof-of-Stake (PoS) light client for the Beacon chain. This  client deprecates the older PoW light client we developed in 2020.

The beacon client tracks the beacon chain, the new Ethereum chain that will replace Ethereum's Proof-of-Work consensus method around mid-September, called the Merge. The work we have done consists of the following parts:

* Beacon Client pallet
  * Initial chain snapshot (forms part of the Genesis Config)
  * Sync committee updates
  * Finalized beacon header updates
  * Execution header updates
  * Message verification
* Beacon Relayer
  * Sends data from a beacon node to the beacon client

## Concepts

### Before the Merge: Execution Layer

Before the Merge, the Ethereum chain as we know it existed in isolation in the sense that consensus was determined by the same chain, using Proof-of-Work (POW).

<figure><img src="../../.gitbook/assets/Screenshot 2022-10-19 at 16.09.41.png" alt=""><figcaption><p>Ethereum Chain before the Merge</p></figcaption></figure>

### After the Merge: Consensus Layer

After the Merge, the Beacon chain became the sole manner in which consensus is tracked on Ethereum. The Beacon chain is a separate chain that was launched on 1 December 2020 and has been running independently since then. On 15 September 2022, the original Ethereum chain's POW consensus method was disabled and the chain switched over to the Beacon chain for consensus. The original Ethereum chain is now often referred to as the Execution Layer and the Beacon chain as the Consensus Layer.

<figure><img src="../../.gitbook/assets/Screenshot 2022-10-19 at 16.07.23.png" alt=""><figcaption><p>Ethereum Chains after the Merge</p></figcaption></figure>

### **Snowbridge Beacon Client**

#### **Beacon Headers & Execution Headers**

The Snowbridge light client to track Ethereum consensus is implemented as an on-chain Beacon client, on the parachain. It is implemented as a Substrate pallet and the code can be found on Github under the [`ethereum-beacon-client` pallet](../../../parachain/pallets/ethereum-beacon-client/src/lib.rs).

The beacon client tracks finalized beacon blocks. The Beacon chain introduced finality to the chain (more on this later). Since it is vital that transfer messages are included in the canonical chain (and not in blocks that go through a re-org), the beacon client only tracks blocks that are ancestors of finalized beacon blocks.

In the diagram below, the purple blocks are examples of those stored in the beacon client. Only finalized beacon blocks are stored as checkpoints. Not all finalized beacon blocks need to be stored and skipping a finalized block is allowed, since these finalized blocks are merely used as checkpoints to indicate that all ancestors of such a block will be seen as finalized as well.

Beacon blocks and execution headers are linked through the `ExecutionPayload` field in a Beacon block. To verify messages, we are particularly interested in the `receiptsRoot` hash, which is used to verify the Ethereum message receipt containing the details about the transfer. For this reason, we store all the execution headers that are ancestors of a finalized beacon header.&#x20;

<figure><img src="../../.gitbook/assets/Screenshot 2022-10-19 at 16.12.09.png" alt=""><figcaption><p>Snowbridge storage (items in purple are stored on-chain)</p></figcaption></figure>

#### Sync Committees

Additionally, the beacon client also syncs sync committees. Sync committees are a subset of randomly chosen validators to sign blocks for a sync committee period (256 epochs, around 27 hours).

<figure><img src="../../.gitbook/assets/Screenshot 2022-10-19 at 16.15.49.png" alt=""><figcaption></figcaption></figure>



### Proofs

The Beacon client checks the following proofs before storing beacon headers and execution headers:

* Merkle proof of the beacon state root to verify if the supposedly finalized header is finalized
* BLS signature verification to assert that the sync committee signed the block attesting to the finalized header

Additionally, the sync committee and next sync committee is also verified using Merkle proofs, to verify if those sync committees are part of the beacon state.

## Beacon Client Operations

### **Initial Snapshot**

The beacon light client expects an initial snapshot of the beacon chain, in the form of a finalized header and the current sync committee (the current elected subset of validators that signs blocks and is used for light client tracking). This initial config is used for the genesis config of the parachain.

### **Sync committee updates**

After the initial snapshot has been validated, the beacon relayer periodically sends sync committee updates. These updates contain the next sync committee. The sync committee subset of validators change every \~27 hours. The sync committee is verified using a Merkle proof and then stored in storage.

Storage will always contain:

* The current sync committee
* The next sync committee

### **Finalized beacon header updates**

After the initial snapshot, the relayer sends finalized beacon header updates to the beacon client. The beacon client verifies the finalized beacon header in the following ways:

* Checks that the beacon state confirms the finalized header using a merkle proof
* Checks that the header attesting to the finalized header was signed by the sync committee

The finalized beacon header is stored in storage.

### **Execution header updates**

Once there are more than 2 beacon finalized headers, all the execution headers between the two finalized beacon headers are backfilled. The execution header lives on the Ethereum execution layer (historically just the Ethereum chain). The execution header looks almost the same as it used to in the Ethereum PoW world. Each beacon header contains an ExecutionPayload header which is on the execution layer. The execution header is also stored in storage.

### **Message verification**

The light client is also responsible for verifying incoming Ethereum events. It does so using transaction receipt proofs which prove that a particular transaction to a particular Ethereum smart contract was in fact valid, was included in the chain, and did emit some event. It accepts and processes a proof, verifies it and then returns the set of Ethereum events that were emitted by the proven transaction receipt.

## Implementation

Pallets:

* [ethereum-beacon-client](https://github.com/Snowfork/snowbridge/tree/main/parachain/pallets/ethereum-beacon-client)

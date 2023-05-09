# Polkadot

We use Polkadot’s[ BEEFY](https://github.com/paritytech/grandpa-bridge-gadget/blob/master/docs/walkthrough.md) gadget to implement an efficient light client that only needs to verify a very small subset of relay chain validator signatures. BEEFY is live on Rococo, and is awaiting deployment on Kusama and Polkadot.

Fundamentally, the BEEFY light client allows the bridge to prove that a specified parachain header was finalized by the relay chain.

We want a bridge design that is light enough to deploy on Ethereum. It will be too expensive to verify signatures from say 1000 validators of the Polkadot relay chain on Ethereum, so we basically have two choices: verify all signatures in succinct proofs or only verify a few signatures. We settled for a design that tries to make the latter cryptoeconomically secure.

The ideal security to aim for is for an attack to be as expensive as the smaller market cap of DOT and ETH. Unfortunately, we can only slash the bond of the few validators whose signatures are verified, so any attack attempt is necessarily much cheaper than the whole market cap. However, we can aim to make an attack very expensive in expectation by making sure that an attack succeeds with low probability and that failed attacks still cost the attackers.

## Update Protocol

The light client needs to be frequently updated with new BEEFY commitments by an untrusted permissionless set of relayers.&#x20;

BEEFY commitments are signed by relay chain validators. The light client needs to verify these signatures before accepting commitments.

In collaboration with W3F, we have designed a protocol where the light client needs to only verify $$\lceil log_2{(3N)}\rceil$$ signatures from randomly-chosen validators​, where $$N$$ is the size of the current validator set.

In the EVM there is no cryptographically-secure source of randomness. Instead we make our update protocol crypto-economically secure through an [Interactive Update Protocol](interactive-update-protocol.md). In this protocol, a candidate commitment is verified over 2 transactions. At a high-level it works like this:

1. In the [first transaction](https://github.com/Snowfork/snowbridge/blob/54b62c92445635164d1414af742e26b56a097003/core/packages/contracts/src/BeefyClient.sol#L199), the relayer submits the commitment, and an initial bitfield claiming which validators have signed the commitment.
2. The relayer must then wait [MAX\_SEED\_LOOKAHEAD](https://eth2book.info/bellatrix/part3/config/preset/#max\_seed\_lookahead) blocks.
3. The relayer submits a [second transaction](https://github.com/Snowfork/snowbridge/blob/54b62c92445635164d1414af742e26b56a097003/core/packages/contracts/src/BeefyClient.sol#L227) to reveal and commit to a random seed, derived from Ethereum's [RANDAO](https://eips.ethereum.org/EIPS/eip-4399).
4. The relayer [requests](https://github.com/Snowfork/snowbridge/blob/54b62c92445635164d1414af742e26b56a097003/core/packages/contracts/src/BeefyClient.sol#L524) from the light client a bitfield with $$\lceil log_2{(3N)}\rceil$$randomly-chosen validators sampled from the initial bitfield.​
5. The relayer sends a [third transaction](https://github.com/Snowfork/snowbridge/blob/54b62c92445635164d1414af742e26b56a097003/core/packages/contracts/src/BeefyClient.sol#L255) with signatures for all the validators specified in the final bitfield
6. The light client verifies all validator signatures in the third transaction to ensure:
   1. &#x20;The provided validators are in the current validator set
   2. The provided validators are in the final bitfield
   3. The provided validator have signed the beefy commitment
7. If the third transaction succeeds then the payload inside the BEEFY commitment is applied

**Note**: The constants and parameters in this protocol will need to be reviewed by us and W3F before launch.

## Message Verification

On our parachain, outbound channels periodically emit message commitment hashes which are inserted into the parachain header as a digest item. These commitment hashes are produced by hashing a set of messages submitted by end users.

To verify these commitment hashes, the light client side needs the following information&#x20;

1. The full message bundle
2. Partial parachain header
3. A merkle leaf proof for the parachain header containing the commitment hash for (1)
4. An MMR leaf proof for the MMR leaf containing the merkle root for the merkle tree in (2)

Working backwards, if the BEEFY light client successfully verifies a parachain header, then the commitment hash within that header is also valid, and the messages mapping to that commitment hash can be safely dispatched.

## Implementation

Solidity Contracts:&#x20;

* [BeefyClient.sol](../../../../core/packages/contracts/src/BeefyClient.sol)
* [ParachainClient.sol](../../../../core/packages/contracts/src/ParachainClient.sol)

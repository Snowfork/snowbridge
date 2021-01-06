---
layout: default
title: Polkadot Relay Chain Interactive Update Protocol
nav_order: 2
permalink: /concepts/polkadot-light-client-verifier/interactive-protocol
parent: Polkadot Light Client Verifier
grand_parent: Concepts and Architecture
---
# Polkadot Relay Chain Interactive Update Protocol

We want a bridge design that is light enough to deploy on Ethereum 1.x. It will be too expensive to verify signatures from say 1000 validators of the Polkadot relay chain on Ethereum, so we basically have two choices: verify all signatures in succinct proofs or only verify a few signatures. We settled for a design that tries to make the latter cryptoeconomically secure. The ideal security to aim for is for an attack to be as expensive as the smaller market cap of DOT and ETH. Unfortunately, we can only slash the bond of the few validators whose signatures are verified, so any attack attempt is necessarily much cheaper than the whole market cap. However, we can aim to make an attack very expensive in expectation by making sure that an attack succeeds with low probability and that failed attacks still cost the attackers.

To convince the chain that at least 1/3 of validators voted for something, we will need to sample validators who are claimed to vote for this at random. We will use a block hash as the source of randomness, which means we need to deal with the issue that this is influenceable. We counter that by using a block hash that comes `n` blocks after the proposal. Since we are working with the assumption that at least 2/3 of validators are honest and up to almost 1/3 might be dishonest, we can only expect 2/3 of validators to vote for something. If only exactly $$(n-1)/3 + 1$$ validators vote for something, then we cannot prove that at least $$(n-1)/3 + 1$$ did by random sampling. Instead we will need a vote where all honest validators vote for something and it is enough to show that a single honest validator voted for it, for which it suffices to argue that over 1/2 of these 2/3 claimed votes are for it. This means that the validators must sign something after they know it is already final, which is an extra signature/vote beyond just that for Byzantine agreement.

We will describe this as a light client that uses an interactive protocol and then discuss how to implement it in Ethereum.

## The interactive protocol

A prover wants to convince a light client that at least $$1/3$$ of validators signed a statement, which they claim that a specific set of at least $$2/3$$ of validators do. We assume that the light client already has a Merkle root $$r_{val}$$ of validator public keys.

1. The prover sends to the light client the statement $$S$$, a bitfield $$b$$ of validators claimed to sign it (which claims that more than $$2/3$$ of validators signed $$S$$), one signatures $$sig$$ on $$S$$ from an arbitrary validator together with Merkle proofs from $$r_{val}$$ of their public key.

2. The light client checks that the bitfield claims at least 2/3 of validators signed. It then verifies the backing signature and its proof and asks for $$k_{approval}$$ validators at random among those that $$b$$ claims signed $$S$$

3. The prover sends the signatures $$sig_i$$, Merkle proofs of public keys from $$r_{val,i}$$ and Merkle proofs of signatures from $$r_{sig,i}$$ corresponding to the validators the light client asked for.

4. If all these signatures are also correct then the light client accepts the block.

Analysis: If at least $$2/3$$ of validators are honest but no honest validator signed $$S$$, then at least $$1/2$$ of validators the prover claimed to sign $$S$$ did not. Therefore the proof fails with probability at least $$2^{-k_{approval}}$$.

Furthermore, if signing an incorrect statement $$S$$ is slashable and we slash by at least $$minsupport$$, then if the light client reports the initial claim, then at least $$minsupport$$ stake can be slashed for an incorrect inital statement. Now if at least $$2/3$$ of validators are honest, then the proof fails with probability $$2^{-k_{approval}}$$ and so there is an expected cost of $$2^{k_{approval}} minsupport$$.

## Implementing this on an Ethereum PoW Chain

In this case the light client is a smart contract. We can use a block hash as a source of randomness, although this can be manipulated.

In order to have an adversary with access to sufficient hashpower on Ethereum still undertake an unknown risk when submitting backing signatures on an invalid statement to the light client, we use the block hash of the block exactly 100 blocks later than the block in which the original claim transaction was included as a source of randomness.

Now, if we e.g. assume that over $$2/3$$ of the hashpower of Ethereum is owned by honest miners and that over $$1/2$$ of the claimed signers are honest validators who didn't sign $$S$$, then we can analyse the maximum probability that the 100th block after the first transaction was included has a blockhash that results in the test succeeding against any strategy by the adversarial miners. This will involve building a Markov chain model of the worst case and proving that the bad guys can't do better. A back of the envelope calculation gave me that this would be something like $$7p/2$$ chance of success vs $$p$$ for a random number.

Now we'd rather argue about rational miners than honest ones. In this case, producing a block with a hash that fails the test, which happens with probability $$1-p$$, would gain the miner some block reward $$R$$ if they released it. It would cost them in expectation $$(1-p)/p$$ block rewards. With $$p=2^{-k_{approval}}$$, this is $$(2^{k_{approval}}-1)R$$. With $$R=5$$ ETH and $$k_{approval} = 25$$, this would be 167,772,155 ETH which is more than the 112,421,804 ETH currently in existence. Something like this would be secure enough for rational miners to be honest even if there was only one mining pool for Ethereum.

### The protocol

1. First a transaction including the data as in 1. above:
   _the statement $$S$$, a bitfield $$b$$ of validators claimed to sign it (which claims that more than $$2/3$$ of validators signed $$S$$), one signature $$sig$$ on $$S$$ from an arbitrary validator together with Merkle proofs from $$r_{val}$$ of their public key._

   is placed on the Ethereum chain. The smart contract validates the signature and Merkle proof from the $$r_{val}$$ stored on chain. If this passes, it records $$S$$, $$b$$, the block number $$n$$ where this transaction is included and maybe another id or counter $$id$$ for disambiguation.

2. Nothing happens until the block number is at least $$n+k+1$$. At this point, a designated[^designation-motivation] relayer (probably the same as sent the first transaction), can send a second transaction. The blockhash of block $$n+k$$ is used as a pseudorandom seed to generate $$k_{approval}$$ random validators from the $$b$$ validators who signed. The relayer generates a second transaction containing $$S$$, $$id$$, and these signatures.
3. The smart contract processes this transaction. It generates the pseudorandom validators to check from the blockhash of the $$n+k$$th block. It then checks whether these signatures were included, whether they are correct and whether the Merkle proofs from $$r_{val}$$ are correct. If so, it accepts $$S$$ as having happened on Polkadot.

Assuming the relayer used the correct blockhash and has all the signatures they claimed, this will succeed. Probably the pseudorandomness is generated by repeatedly hashing the blockhash.

### Relayer designation procedure

<!---(temporary working name for 1st & 2nd transactions: initialization & finalization transactions)--->

The second transaction (finalization transaction) is expensive compared to the first (initialization transaction). Thus, we need a mutual exclusion protocol that ensures that the finalization transaction is only submitted once by one of the protocol-abiding relayers.

<!--This can be solved by designating a single relayer only to submit this transaction, within some timeout period.-->

The most apparent choice of this relayer is the author of the first transaction. Since the bridge can be attacked by intentionally timing out on the finalization transaction, we require collateral to be locked by the designated relayer to secure the mutex lock for them. In this case, the designated relayer should already be chosen before the initialization transaction.

Since the light client will be unaware of the designation choice, it can nonetheless be blocked with illegitimate initialization transactions since they are comparatively cheap. A possible solution for this is to have the light client store a bounded number of these initialization transactions concurrently, but to require them to lock collateral with the light client as well.

If we use the author of the initialization transaction, we still need to cater for the possibility that they -- intentionally or not -- time out on the second transaction.

### Optimistic scheme

A protocol with competing initialization transactions for a given statement $$S_n$$ is only required whenever:

1. the designated relayer for $$S_n$$ times out on the initialization transaction or submits a malicious statement $$S'_n$$
2. the designated relayer for $$S_n$$ times submits a valid initialization transaction, but times out on the finalization transaction

Assuming that these costly situations are rare, we can take an optimistic approach where on the Polkadot side, we already commit to the relayer for $$S_n$$ much earlier and include their identity in the statement $$S_{n-k}$$, where $$k$$ is a tradeoff between the minimum distance between Polkadot blocks that we relay and the number of backup relayers, and the range of statements for which we must pre-commit to designated relayers.

This would then allocate a mutex for $$S_n$$ for a block range within which the smart contract will only accept initialization transactions from the pre-elected relayer. If and once the relayer chosen in $$S_{n-k}$$ times out on the initialisation transaction for $$S_n$$, the relayer for statement $$S_{n-k+1}$$ (who was elected for relaying statement $$S_{n+1}$$) attains the mutex for $$S_n$$ on the initialization transaction and submits this initialization transaction instead.

This scheme can be iterated for up to $$k-1$$ failures, at which point we must revert to a protocol with competing initialization transactions. As such, $$k$$ increases the number of backup relayers we can have to remain within the optimistic scheme, but thus also increases the impact a sequence of colluding designated relayers can have.

[^designation-motivation]: The motivation for having a designation procedure is that the second transaction will be very expensive, yet we will only refund one single relayer for sending it

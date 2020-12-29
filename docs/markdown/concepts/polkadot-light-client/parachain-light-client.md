---
layout: default
title: Snowbridge Parachain Light Client
nav_order: 4
permalink: /concepts/polkadot-light-client-verifier/parachain-light-client
parent: Polkadot Light Client Verifier
grand_parent: Concepts and Architecture
---

# Snowbridge Parachain Light Client

The parachain light client receives our latest parachain block headers from the MMR Light Client. With them, it can extract our parachain commitments.

We can use these commitments to verify and process every message that has been committed to by the bridge.

More details coming soon...

<!-- TODO: again, storing them is very expensive. Just go up in the proof to the MMR root is much cheaper.  -->
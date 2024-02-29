---
description: >-
  Generate a beacon checkpoint to sync the Ethereum client from. This is done on
  bridge initialization or forced sync reset.
---

# Initialize Ethereum light client with Forced Checkpoint

## Generate Checkpoint Data

On the server where the relayer is deployed, run the following command:

```
relayer/build/snowbridge-relay generate-beacon-checkpoint --url http://127.0.0.1:9596
```

The command will output the beacon checkpoint data in hex form. Prepend  `0x5200` to the resulting hex (the [Ethereum client pallet index](https://github.com/Snowfork/polkadot-sdk/blob/snowbridge/cumulus/parachains/runtimes/bridge-hubs/bridge-hub-rococo/src/lib.rs#L730) and the `force_checkpoint` [call index](https://github.com/Snowfork/polkadot-sdk/blob/snowbridge/bridges/snowbridge/pallets/ethereum-client/src/lib.rs#L216) combined).

### Call Force Checkpoint from Relay Chain

Use the resulting call data to make the Transact call to set the beacon checkpoint.

* call hash: `0xff00630003000100d50f03082f00000602070008d6e82982ee3600a5025200821017d7c8b03a2a182824cfe569187a28faa718368a0ace36e2b1b8b6dbd7f8093c0594aa8a9c557dabac173fac324158e77fb5840738a1a541f633cbec8884c6a601c567d2b376a05392000000000000000002101851a76c1adff357d59b36327d02cfb7f718368a0ace36e2b1b8b6dbd7f8093c0594aa8a9c557dabac173fac324158e77fb5840738a1a541f633cbec8884c6a601c567d2b376a05392000000000000000000`
* link to pre-populate on Rococo: [https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Frococo-rpc.polkadot.io#/extrinsics/decode/0xff00630003000100d50f03082f00000602070008d6e82982ee3600a5025200821017d7c8b03a2a182824cfe569187a28faa718368a0ace36e2b1b8b6dbd7f8093c0594aa8a9c557dabac173fac324158e77fb5840738a1a541f633cbec8884c6a601c567d2b376a05392000000000000000002101851a76c1adff357d59b36327d02cfb7f718368a0ace36e2b1b8b6dbd7f8093c0594aa8a9c557dabac173fac324158e77fb5840738a1a541f633cbec8884c6a601c567d2b376a05392000000000000000000](https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Frococo-rpc.polkadot.io#/extrinsics/decode/0xff00630003000100d50f03082f00000602070008d6e82982ee3600a5025200821017d7c8b03a2a182824cfe569187a28faa718368a0ace36e2b1b8b6dbd7f8093c0594aa8a9c557dabac173fac324158e77fb5840738a1a541f633cbec8884c6a601c567d2b376a05392000000000000000002101851a76c1adff357d59b36327d02cfb7f718368a0ace36e2b1b8b6dbd7f8093c0594aa8a9c557dabac173fac324158e77fb5840738a1a541f633cbec8884c6a601c567d2b376a05392000000000000000000)

Replace the encoded call bytes highlighted in the screenshot below with the checkpoint data generated in the previous step:

<figure><img src="../.gitbook/assets/Screenshot 2024-02-16 at 11.21.53.png" alt=""><figcaption><p>Replace encoded call bytes with hex generated in the first step.</p></figcaption></figure>






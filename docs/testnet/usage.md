---
description: Using the Rococo-Goerli Bridge.
---

# Usage

## Disclaimer

* The bridge is still in development and the testnet is liable to break at any time.
* Fees and prices of transfers are not yet finalized.
* For support please reach out [here](https://github.com/Snowfork/snowbridge/discussions).

## Goerli → Rococo

Interacting with the bridge through various App contracts on Ethereum using the contracts tab in Etherscan.

* A single transaction through the bridge will cost a fee of `0.1 wDOT`.
* Transfers can take between 15-30 minutes.

### Acquiring wDOT (Fees)

Token Address: [0x954C4f2F26eC46e9c3D07a89a7571e2E75b31675](https://goerli.etherscan.io/token/0x954C4f2F26eC46e9c3D07a89a7571e2E75b31675)

You can get `wDOT` (also named `SnowDOT`) by adding the token to MetaMask and swapping some ETH for it on [Uniswap](https://app.uniswap.org/#/swap?inputCurrency=ETH\&outputCurrency=0x954C4f2F26eC46e9c3D07a89a7571e2E75b31675\&exactAmount=1\&exactField=output) (connect Metamask so that the Goerli network will be selected).

<figure><img src="../.gitbook/assets/Untitled" alt=""><figcaption></figcaption></figure>

### Ether

Etherscan: [0xf1becfdca540605451553b4d5f80acf17c7a490a](https://goerli.etherscan.io/address/0xf1becfdca540605451553b4d5f80acf17c7a490a)

To initiate a transfer you can use the `lock` method on the ETHApp contract.

| Parameter   | Description                                                                                                                                                                                                      |
| ----------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| \_amount    | The amount you want to send.                                                                                                                                                                                     |
| \_recipient | <p>The polkadot address that will receive the funds encoded in hex format.<br>Address conversion from SS58 address can be done using the <code>subkey</code> cli.<br><code>subkey inspect SS58_ADDRESS</code></p> |
| \_channelId | The channel to use. It must be set to 1 to use the incentivized channel which requires users to pay fees.                                                                                                        |
| \_paraId    | Destination parachain id. Must be 0.                                                                                                                                                                             |
| \_fee       | Destination parachain fees. Must be 0.                                                                                                                                                                           |

<figure><img src="../.gitbook/assets/Untitled 1" alt=""><figcaption></figcaption></figure>

To check the balance you can use the [PolkadotJS](https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Frococo-rpc.snowbridge.network#/explorer) explorer and use the chain state screen to query the `assets-pallet` for the `_recipient` address above. ETH is stored as asset id `0`.

| Parameter | Description                                                                                                          |
| --------- | -------------------------------------------------------------------------------------------------------------------- |
| 0         | The asset id to query. 0 for ETH                                                                                     |
| 1         | The address holding the asset. The address must be added to your wallet or you can provide a SS58 formatted address. |

<figure><img src="../.gitbook/assets/Untitled 2" alt=""><figcaption></figcaption></figure>

### DOT

Etherscan: [0x7f138fd13e80d8141daf9ca1d3808e3164974655](https://goerli.etherscan.io/address/0x7f138fd13e80d8141daf9ca1d3808e3164974655)

To transfer `SnowDOT` you can use the `burn` method on the DOTApp contract. This will burn the `SnowDOT` ERC20 token on the Ethereum side and mint `DOT` on the Snowbridge side.

| Parameter   | Description                                                                                                                                                                                                       |
| ----------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| \_recipient | <p>The polkadot address that will receive the funds encoded in hex format.<br>Address conversion from SS58 address can be done using the <code>subkey</code> cli.<br><code>subkey inspect SS58_ADDRESS</code></p> |
| \_amount    | The amount of SnowDOT formatted with 18 decimals.                                                                                                                                                                 |
| \_channelId | The channel to use. It must be set to 1 to use the incentivized channel which requires users to pay fees.                                                                                                         |

<figure><img src="../.gitbook/assets/Untitled 3" alt=""><figcaption></figcaption></figure>

To check the balance you can use the [PolkadotJS](https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Frococo-rpc.snowbridge.network#/explorer) explorer and use the chain state screen to query the `system-pallet` for the `_recipient` address above.

| Parameter | Description                                                                                                          |
| --------- | -------------------------------------------------------------------------------------------------------------------- |
| 0         | The address holding the asset. The address must be added to your wallet or you can provide a SS58 formatted address. |

<figure><img src="../.gitbook/assets/Untitled 4" alt=""><figcaption></figcaption></figure>

### ERC20App - ERC20 Tokens

Etherscan: [0x6d61e454d0f1227e285e8d89021a579c036f0bf7](https://goerli.etherscan.io/address/0x6d61e454d0f1227e285e8d89021a579c036f0bf7)

To transfer an ERC20 token you can use the `lock` method on the ERC20App contract.

| Parameter   | Description                                                                                                                                                                                                       |
| ----------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| \_token     | The ERC20 Tokens contract address.                                                                                                                                                                                |
| \_recipient | <p>The polkadot address that will receive the funds encoded in hex format.<br>Address conversion from SS58 address can be done using the <code>subkey</code> cli.<br><code>subkey inspect SS58_ADDRESS</code></p> |
| \_amount    | The amount of SnowDOT formatted with 18 decimals.                                                                                                                                                                 |
| \_channelId | The channel to use. It must be set to 1 to use the incentivized channel which requires users to pay fees.                                                                                                         |
| \_paraId    | Destination parachain id. Must be 0.                                                                                                                                                                              |
| \_fee       | Destination parachain fees. Must be 0.                                                                                                                                                                            |

<figure><img src="../.gitbook/assets/Untitled 5" alt=""><figcaption></figcaption></figure>

To check the balance you can use the [PolkadotJS](https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Frococo-rpc.snowbridge.network#/explorer) explorer and use the chain state screen to query the `assets-pallet` for the `_recipient` address above. ERC20 tokens are stored in the Assets pallet. AssetId’s are assigned incrementally and created on demand.

To find the asset id for a token by querying `assetId` on the `erc20app`.

<figure><img src="../.gitbook/assets/Untitled 6" alt=""><figcaption></figcaption></figure>

You can now view the balance using the `assets-pallet`.

| Parameter | Description                                                                                                          |
| --------- | -------------------------------------------------------------------------------------------------------------------- |
| 0         | The asset id to query. 1 in this example for UNI (Uniswap).                                                          |
| 1         | The address holding the asset. The address must be added to your wallet or you can provide a SS58 formatted address. |

<figure><img src="../.gitbook/assets/Untitled 7" alt=""><figcaption></figcaption></figure>

## Rococo → Goerli

This involves users interacting with our various App pallets on the Snowbridge parachain using the [PolkadotJS](https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Frococo-rpc.snowbridge.network#/explorer) user interface.

* A single transaction through the bridge will cost a fee of `0.01 wETH`.
* Transfers can take between 5-10 minutes.

### Acquiring wETH for fees

You can acquire `wETH` using ETHApp contract in the step specified [here](https://www.notion.so/Using-the-Rococo-Goerli-Bridge-3ad88090091b4b7cb6d62f63c6b4fcf5).

### Ether

To transfer `wETH` across the bridge to submit a `burn` extrinsic to the `ethApp` pallet.

| Parameter | Description                                                                          |
| --------- | ------------------------------------------------------------------------------------ |
| channeld  | The channel to use. It must be set to Incentivized which requires users to pay fees. |
| recipient | The Ethereum address that will receive the funds.                                    |
| amount    | The amount to send.                                                                  |

<figure><img src="../.gitbook/assets/Untitled 8" alt=""><figcaption></figcaption></figure>

This will eventually become `ETH` on the Ethereum side of the bridge. You will be able to see you ETH balance increase in your Wallet or on Etherscan.

### DOT

To transfer `DOT` across the bridge to submit a `lock` extrinsic to the `dotApp` pallet.

| Parameter | Description                                                                          |
| --------- | ------------------------------------------------------------------------------------ |
| channeld  | The channel to use. It must be set to Incentivized which requires users to pay fees. |
| recipient | The Ethereum address that will receive the funds.                                    |
| amount    | The amount to send.                                                                  |

<figure><img src="../.gitbook/assets/Untitled 9" alt=""><figcaption></figcaption></figure>

This will eventually become `SnowDOT` on the Ethereum side of the bridge. You will be able to see you `SnowDOT` balance increase in your Wallet or on Etherscan.

### ERC20 Tokens

To transfer token across the bridge to submit a `burn` extrinsic to the `erc20App` pallet.

| Parameter | Description                                                                          |
| --------- | ------------------------------------------------------------------------------------ |
| channeld  | The channel to use. It must be set to Incentivized which requires users to pay fees. |
| token     | The token contract address on the Ethereum.                                          |
| recipient | The Ethereum address that will receive the funds.                                    |
| amount    | The amount to send.                                                                  |

<figure><img src="../.gitbook/assets/Untitled 10" alt=""><figcaption></figcaption></figure>

You will be able to see your token balance increase in your Wallet or on Etherscan.

# Running the bridge locally and testing it end to end manually from UI yourself

## Local extension Setup:
 - The tests setup a ganache server with a preset mnemonic. Add the first key from this mnemonic to your Metamask as an extra account to use for testing. You can find this first key here: ARTEMIS_ETHEREUM_KEY in /polkadot-ethereum/test/docker-compose.yml
 - Add Localhost:8545 to your Metamask and use it
 - Setup your polkadot-js extension in your browser and create a test Play account

## Local Server Setup:
 - Run all test docker containers as per the test README
 - Make sure you have pulled and setup our substrate-ui fork in a seperate folder to this repo (https://github.com/Snowfork/substrate-ui)
 - Look at the deployed smart contract addresses in the test build config which should have been generated here: /test/build/address.json
 - Update the bridge react package in your local substrate-ui fork's config with the deployed ETHApp and ERC20App address (In /substrate-ui/packages/page-ethereum-bridge/src/config.tsx)
 - Add the TestToken address from /test/build/address.json to your metamask wallet assets so you can see your TestTokens. Some should have been automatically minted to your test metamask account
 - Run our substrate-ui fork

## Using the substrate-ui fork
 - Go to localhost:3000
 - Send some unit currency from ALICE to your Play account via polkadot-js browser extension
 
## Sending eth accross:
 - Copy your PLAY account address from your polkadot-js extension or http://localhost:3000/#/accounts
 - Work out your public key in hex format *(sorry, our dapp only supports hex format, this will be fixed)*: In a terminal run ```subkey inspect <the address you just copied>```
 - Copy the Public key in hex without the 0x prefix *(eg: 0x2c8829e0ca67b23ed41c44c21b98c5ce916aab0e5a2a01f4576c9a6bf8331e4e -> **2c8829e0ca67b23ed41c44c21b98c5ce916aab0e5a2a01f4576c9a6bf8331e4e**)*
 - Go to the bridge app http://localhost:3000/#/app-ethereum-bridge
 - Send 1 ETH to the unprefixed hex address (Make sure it is sent from your new test Metamask account to the localhost:8545 server)
 - See the metamask transaction succeed
 - See your balance update in Metamask
 - Go to the explorer and see the new asset has been minted: http://localhost:3000/#/explorer

## Query your PolkaETH Balance
 - Go to query: http://localhost:3000/#/explorer
 - Query the asset pallet for asset id 0x and your Play account
 - You should see 1 PolkaETH (18 decimal places)

## Burning PolkaETH to send it back
- Go to submit an extrinsic: http://localhost:3000/#/extrinsics
- Submit an eth burn transaction to your original metamask wallet address for 1 ETH
- Sign and send with Polkadot-JS extension
- Go to network explorer and see the new events
- A few seconds later, see your balance update in metamask

## Sending/Querying/Burning PolkaERC20
 - Go to the bridge app, add the TestToken address in the contract address field (from /test/build/address.json)
 - Add the TestToken address to your metamask wallet assets
 - Approve 1 token
 - Send a token
 - See successful recent events again in the explorer
 - ...etc
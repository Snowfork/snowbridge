Options for dynamic XCM in Ethereum -> Polkadot direction
----------------------------------------------------------------

Option A: Encode Destination Parachain and Parachain Pallet in RLP on Ethereum. Consuming Parachain needs to have ethereum-xcm-adapter to be able to interpret the message and route it to the right pallet.

Ethereum ETHApp:
 -> emits plain Ethereum event(target_parachain_index, target_pallet_index, payload)

Parachain A:
accepts Ethereum event (target_parachain_index, target_pallet_index, payload)
 -> converts to handle(target_pallet_index, payload)
 -> xcm via transact to handle(target_pallet_index, payload) on second Parachain

Parachain B:
receives transact with Call::handle(target_pallet_index, payload)
 -> xcm-executes the transact, sends to xcm-adapter pallet
 -> xcm-adapter pallet processes handle(target_pallet_index, payload)
   -> statically maps target_pallet_index to another dispatchable Call (eg: mint on ETHApp)
   -> calls dispatchable ETHApp.mint(payload)
      -> ETHApp receives dispatchable, rlp decodes payload, and processes it
---------
+: easiest implementation, cheaper on ethereum encoding side as just rlp encoding, parachain developer can control/update routing easily, explicit coupling between pallet and smart contract
-: depends on parachain adding the xcm-adapter pallet, requires static mapping from source

----------------------------------------------------------------------------------------------------------------------------------------------------------------------------

Option B: Encode Destination Parachain and Parachain Pallet in SCALE on Ethereum. Consuming Parachain can accept xcm via transact directly and routing to the right pallet happens automatically via xcm-executor.

Ethereum ETHApp:
 -> SCALE encode routing information, ie SCALE(target_parachain_index, target_pallet_index, payload)
 -> emits Ethereum event(SCALE(target_parachain_index, target_pallet_index, payload))

Parachain A:
accepts Ethereum event (SCALE(target_parachain_index, target_pallet_index, payload))
 -> xcm via transact(SCALE(target_parachain_index, target_pallet_index, payload)) on second Parachain

Parachain B:
receives transact with Call::handle(target_pallet_index, payload)
 -> xcm-executes the transact, sends to ETHApp pallet, mint function
 -> xcm-execute calls dispatchable ETHApp.mint(payload)
      -> ETHApp receives dispatchable, rlp decodes payload, and processes it
-------------
+: no static mapping/adapter pallet needed, any ethereum contract can interface with parachain B pallets without permission, though can only interface with parachain B pallets that understand RLP.
-: need to implement (int,int,byte) full SCALE Call encoding on ethereum, and pay cost to do encoding on ethereum, implicit coupling between parachain and smart contract, so any updates to parachain or its pallets can break the ethereum side
----------------------------------------------------------------------------------------------------------------------------------------------------------------------------

Option C: Encode Destination Parachain and Parachain Pallet AND payload data in SCALE on Ethereum. Consuming Parachain can accept xcm via transact directly and routing to the right pallet happens automatically via xcm-executor.

Ethereum ETHApp:
 -> SCALE encode routing information, ie SCALE(target_parachain_index, target_pallet_index, amount, receiver)
 -> emits Ethereum event(SCALE(target_parachain_index, target_pallet_index, amount, receiver))

Parachain A:
accepts Ethereum event (SCALE(target_parachain_index, target_pallet_index, amount, receiver))
 -> xcm via transact(SCALE(target_parachain_index, target_pallet_index, amount, receiver)) on second Parachain

Parachain B:
receives transact with Call::handle(target_pallet_index, amount, receiver)
 -> xcm-executes the transact, sends to ETHApp pallet, mint function
 -> xcm-execute calls dispatchable ETHApp.mint(amount, receiver)
      -> ETHApp receives dispatchable and processes it
-------------
+: no static mapping/adapter pallet needed, any ethereum contract can interface with parachain B pallets whatsover without permission, even if those pallets know nothing about ethereum or the smart contract. allows potentially completely permissionless cross-chain calls.
-: need to implement full SCALE Call encoding on ethereum and basically full SCALE encoder for any data type on Etheruem, and possibly pay even higher cost to do encoding on ethereum, completely implicit decoupling between parachain and smart contract, so any updates to parachain or its pallets can break the ethereum side
# Smoketests

# Bindings

## Parachains

BridgeHub
```shell
subxt codegen --url http://localhost:11144 | rustfmt --edition 2021 --emit=stdout > src/parachains/bridgehub/mod.rs
```

## Ethereum Contracts

```shell
forge bind --module --overwrite --bindings-path src/contracts --root ../core/packages/contracts
```

# Smoketests

Before running deposit funds into Statemint's sovereign account on BridgeHub:
```
5Ec4AhPZk8STuex8Wsi9TwDtJQxKqzPJRCH7348Xtcs9vZLJ
```

This is necessary to cover rewards for "execution message" relayers. Messages will be rejected if the account is empty.

# Bindings

Generate bindings:

```shell
./make-bindings.sh
```

# Run Tests

```
cargo test --test lock_tokens -- --nocapture
```
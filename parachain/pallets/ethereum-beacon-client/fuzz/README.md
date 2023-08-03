# Beacon client fuzz tests

This crate contains fuzz tests for the three beacon client extrinsics.

# Installation

```
cargo install cargo-fuzz
```

# Run tests

- Force Checkpoint: `cargo fuzz run fuzz_force_checkpoint -- -max_len=34359738368`
- Submit: `cargo fuzz run fuzz_submit -- -max_len=34359738368`
- Submit Execution Header: `cargo fuzz run fuzz_submit_execution_header -- -max_len=34359738368`

Note: `max-len` is necessary because the max input length is 4096 bytes. Some of our inputs are larger than this
default value. When running the tests without an increased max len parameter, no fuzz data will be generated.

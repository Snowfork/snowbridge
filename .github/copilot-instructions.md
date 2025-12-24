## Quick Orientation for AI coding agents

This repository implements Snowbridge: a multi-language bridge stack with Ethereum contracts, Go relayer services, and several Rust crates/runtimes. The goal of these notes is to help an AI code assistant become productive quickly by pointing to the project's structure, build/test workflows, conventions, and integration touchpoints.

- **High level components**:

  - Contracts: [contracts/](contracts/) — Foundry-managed Solidity contracts and tests. See [contracts/foundry.toml](contracts/foundry.toml).
  - Relayer: [relayer/](relayer/) — Go services that relay between chains; main entrypoint: [relayer/main.go](relayer/main.go#L1-L200). Contains `docker-compose` for local runs: [relayer/docker-compose.yml](relayer/docker-compose.yml).
  - Supporting tools:
    - [control/](control/) (Rust) — control tool helps to build a governance call.
    - [gas-estimator/](gas-estimator/) (Rust) — gas estimator for parachain transactions and etherum interactions.
    - [web/](web/) (JavaScript/TypeScript) — web frontend and related packages.
    - [smoketest/](smoketest/) — end-to-end smoke tests.
    - [scripts/](scripts/) — various helper scripts.

- **Developer environment (must-follow)**:

  - This project uses Nix for a reproducible dev shell. Open the repo with the Nix shell active: `nix develop` (root `flake.nix`). See [README.md](README.md#L1-L120) for `scripts/init.sh` and Nix notes.
  - Run `scripts/init.sh` once after cloning to initialise toolchains/submodules.
  - For editor correctness, open VS Code from the nix shell: `nix develop && code .`.

- **Build & test snippets (concrete examples)**:

  - Contracts (Foundry):
    - `cd contracts && forge build` — compiles contracts using `solc` configured in [contracts/foundry.toml](contracts/foundry.toml).
    - `cd contracts && forge test` — runs Solidity unit tests.
  - Go relayer:
    - `cd relayer && go build` — builds the relayer binary (project uses `go.work`).
    - `cd relayer && go test ./...` — run relayer tests.
  - Rust services / runtimes:
    - `cd control && cargo build --workspace` — build the parachain/runtimes workspace defined in [control/Cargo.toml](control/Cargo.toml).
    - `cd gas-estimator && cargo build` — builds the gas estimator tool.

- **Project conventions & patterns**:

  - Multiple language borders are explicit and isolated by folder (Solidity in `contracts/`, Go in `relayer/`, Rust in `control/` and `gas-estimator/`). Prefer changes scoped to the component directory unless cross-cutting changes are required.
  - `go.work` is used — run Go commands inside `relayer/` or with `go work` aware environment (`nix` shell ensures right toolchain).
  - Foundry config is authoritative for solidity tooling (see `solc_version` and `profile.*` in [contracts/foundry.toml](contracts/foundry.toml)). Use those settings when suggesting upgrades or compilation flags.
  - Rust crates use explicit feature flags for different runtime targets. See `features` in [gas-estimator/Cargo.toml](gas-estimator/Cargo.toml) and `control/Cargo.toml` workspace layout.

- **Integration touchpoints to check when making changes**:

  - Contracts <> Relayer: relayer queries or submits to contract ABIs. Look for code-generation scripts and bindings (search for `make-bindings.sh` and `generate.go`).
  - Relayer <> Parachain: relayer communicates with parachain runtimes (see `runtimes/` under `control/` and `gas-estimator`'s `subxt` usage).
  - Local testnet & smoke tests under `web/packages/test/` and `smoketest/` orchestrate end-to-end runs — update these if behavior across components changes.

- **When editing code, be conservative and reproducible**:

  - Prefer making changes that build and test inside the component directory first (e.g., `cd contracts && forge test`).
  - Run the developer shell (`nix develop`) before suggesting tool installs or running commands. The repo expects specific toolchain versions.
  - If recommending dependency upgrades (Rust/Cargo, Go, Foundry), reference the related manifest file (e.g., `Cargo.toml`, `go.mod`, `contracts/foundry.toml`) and update changelogs/tests.

- **Files to consult for context (quick links)**:
  - Root README and developer notes: [README.md](README.md)
  - Nix setup / init script: [scripts/init.sh](scripts/init.sh)
  - Relayer main: [relayer/main.go](relayer/main.go#L1-L200)
  - Relayer compose: [relayer/docker-compose.yml](relayer/docker-compose.yml)
  - Contracts config: [contracts/foundry.toml](contracts/foundry.toml)
  - Parachain workspace: [control/Cargo.toml](control/Cargo.toml)
  - Gas estimator: [gas-estimator/Cargo.toml](gas-estimator/Cargo.toml)

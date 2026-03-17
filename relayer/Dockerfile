# Stage 1: Build Go relayer
FROM golang:1.23 AS go-builder
WORKDIR /opt/relayer
# Cache Go modules when only application code changes
COPY relayer/go.mod relayer/go.sum ./
RUN go mod download
COPY relayer/ .
# CGO required for go-sqlite3
ENV CGO_ENABLED=1
RUN go build -v -o build/snowbridge-relay main.go

# Stage 2: Build Rust gas estimator
# Build args: GAS_ESTIMATOR_NETWORK (polkadot|paseo|westend), plus RPC URLs for that network.
FROM rust:1.85 AS rust-builder
ARG GAS_ESTIMATOR_NETWORK=polkadot
ARG GAS_ESTIMATOR_ASSET_HUB_URL=wss://asset-hub-polkadot-rpc.n.dwellir.com
ARG GAS_ESTIMATOR_BRIDGE_HUB_URL=wss://bridge-hub-polkadot-rpc.n.dwellir.com
WORKDIR /opt/gas-estimator
RUN apt-get update && apt-get install -y build-essential && rm -rf /var/lib/apt/lists/*
COPY gas-estimator/ .
# Runtime metadata (.bin) is gitignored; generate with subxt before building
RUN cargo install subxt-cli --version 0.50.0
RUN subxt metadata -f bytes --url "${GAS_ESTIMATOR_ASSET_HUB_URL}" > runtimes/asset-hub-${GAS_ESTIMATOR_NETWORK}/asset-hub-metadata.bin && \
    subxt metadata -f bytes --url "${GAS_ESTIMATOR_BRIDGE_HUB_URL}" > runtimes/bridge-hub-${GAS_ESTIMATOR_NETWORK}/bridge-hub-metadata.bin
RUN cargo build --release --features ${GAS_ESTIMATOR_NETWORK}

# Stage 3: Final image
FROM ubuntu:22.04
RUN apt-get update && apt-get install -y gettext-base curl && rm -rf /var/lib/apt/lists/*
COPY --from=go-builder /opt/relayer/build/snowbridge-relay /usr/local/bin/
COPY --from=rust-builder /opt/gas-estimator/target/release/snowbridge-gas-estimator /usr/local/bin/
COPY relayer/config/docker/ /config/
COPY relayer/scripts/docker-entrypoint.sh /usr/local/bin/docker-entrypoint.sh
RUN chmod +x /usr/local/bin/docker-entrypoint.sh
ENTRYPOINT ["/usr/local/bin/docker-entrypoint.sh"]

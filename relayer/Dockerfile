# Build argument for gas estimator network (polkadot, paseo, westend)
ARG GAS_ESTIMATOR_NETWORK=polkadot

# Stage 1: Build Go relayer
FROM golang:1.23 AS go-builder
WORKDIR /opt/relayer
COPY relayer/ .
RUN go build -v -o build/snowbridge-relay main.go

# Stage 2: Build Rust gas estimator
FROM rust:1.85 AS rust-builder
ARG GAS_ESTIMATOR_NETWORK
WORKDIR /opt/gas-estimator
RUN apt-get update && apt-get install -y build-essential && rm -rf /var/lib/apt/lists/*
COPY gas-estimator/ .
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

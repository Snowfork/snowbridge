FROM golang:1.23
WORKDIR /opt/relayer
ADD . .
RUN go build -v -o build/snowbridge-relay main.go

FROM ubuntu:22.04
RUN apt-get update && apt-get install -y gettext-base curl && rm -rf /var/lib/apt/lists/*
COPY --from=0 /opt/relayer/build/snowbridge-relay /usr/local/bin/
COPY scripts/docker-entrypoint.sh /usr/local/bin/docker-entrypoint.sh
RUN chmod +x /usr/local/bin/docker-entrypoint.sh
VOLUME ["/config"]
ENTRYPOINT ["/usr/local/bin/docker-entrypoint.sh"]

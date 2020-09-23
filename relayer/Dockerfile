FROM golang:1.14
WORKDIR /opt/relayer
ADD . .
RUN CGO_ENABLED=0 GOOS=linux go build -v -o build/artemis-relay main.go

FROM parity/subkey:2.0.0-rc4
COPY --from=0 /opt/relayer/build/artemis-relay /usr/local/bin/
ENTRYPOINT ["/usr/local/bin/artemis-relay"]

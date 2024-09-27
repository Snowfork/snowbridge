FROM golang:1.21
WORKDIR /opt/relayer
ADD . .
RUN go build -v -o build/snowbridge-relay main.go

FROM ubuntu:22.04
COPY --from=0 /opt/relayer/build/snowbridge-relay /usr/local/bin/
VOLUME ["/config"]
ENTRYPOINT ["/usr/local/bin/snowbridge-relay"]

FROM golang:1.20 as builder
WORKDIR /app
ADD . .

RUN go build -v -o build/snowbridge-relay main.go

# =============
FROM ubuntu:22.04
COPY --from=builder /app/build/snowbridge-relay /usr/local/bin

VOLUME ["/config"]
ENTRYPOINT ["/usr/local/bin/snowbridge-relay"]

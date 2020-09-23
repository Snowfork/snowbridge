FROM golang:1.14
WORKDIR /opt/relayer
ADD . .
RUN go build -v -o build/artemis-relay main.go

ENTRYPOINT ["/opt/relayer/build/artemis-relay"]
CMD ["run"]

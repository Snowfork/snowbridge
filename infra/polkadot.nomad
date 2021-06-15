job "polkadot" {
  datacenters = ["dc1"]
  group "node-0" {

    volume "volume0" {
      type      = "host"
      read_only = false
      source    = "volume0"
    }

    network {
      port "p2p" {
        to = 30333
      }
      port "rpc" {
        to = 9944
      }
    }

    task "polkadot" {
      driver = "docker"

      volume_mount {
        volume      = "volume0"
        destination = "/data"
        read_only   = false
      }

      service {
        name = "polkadot-alice"
        port = "p2p"
        tags = [
            "alice",
            "traefik.enable=true",
            "traefik.http.routers.polkadot.rule=Host(`polkadot-rpc.snowbridge.network`)",
            "traefik.http.routers.polkadot.entrypoints=websecure",
            "traefik.http.services.polkadot.loadbalancer.server.port=${NOMAD_HOST_PORT_rpc}"
        ]
      }
      config {
        image = "parity/polkadot:v0.9.3"
        args = [
          "--chain", "rococo-local",
          "--alice",
          "--rpc-cors", "all",
          "--ws-external",
          "--rpc-external",
          "--node-key", "ac86396c43f8083b41c02fcbcfde161baf42e56e4ff7bc5bb38144825860fe50"
        ]
        ports = ["p2p", "rpc"]
      }
      resources {
        cpu = 1001
        memory = 1024
      }
    }
  }
  group "node-1" {

    volume "volume1" {
      type      = "host"
      read_only = false
      source    = "volume1"
    }

    network {
      port "p2p" {
        to = 30333
      }
      port "rpc" {
        to = 9944
      }
    }
    task "polkadot" {
      driver = "docker"

      volume_mount {
        volume      = "volume1"
        destination = "/data"
        read_only   = false
      }

    template {
        data = <<EOF
#!/bin/sh
{{ with service "polkadot-alice" }}{{ with index . 0 }}
exec /usr/bin/polkadot --chain rococo-local --bob --rpc-cors all --bootnodes /ip4/{{ .Address }}/tcp/{{ .Port }}/p2p/12D3KooWGbgscGKWfHgGXZU42e1BNkCiBHqobhBptWXceuHsL8VL
{{ end }}{{ end }}
EOF
        change_mode = "restart"
        destination = "local/init.sh"
        perms = 755
      }
      service {
        name = "polkadot-bob"
        tags = ["bob"]
        port = "p2p"
        meta {
          rpc_port = "${NOMAD_PORT_rpc}"
        }
      }
      config {
        image = "parity/polkadot:v0.9.3"
        entrypoint = ["/local/init.sh"]
        ports = ["p2p", "rpc"]
      }
      resources {
        cpu = 1001
        memory = 1024
      }
    }
  }
}
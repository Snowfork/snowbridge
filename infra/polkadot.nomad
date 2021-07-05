job "polkadot" {
  datacenters = ["dc1"]
  group "alice" {

    volume "storage" {
      type = "csi"
      source = "polkadot-node-0"
      attachment_mode = "file-system"
      access_mode = "single-node-writer"
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
        volume      = "storage"
        destination = "/var/lib/polkadot"
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

      user = "root"

      config {
        image = "parity/polkadot:v0.9.5"
        args = [
          "--base-path", "/var/lib/polkadot",
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
        cpu = 1024
        memory = 8192
      }
    }
  }

  group "bob" {

    volume "storage" {
      type = "csi"
      source = "polkadot-node-1"
      attachment_mode = "file-system"
      access_mode = "single-node-writer"
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
        volume      = "storage"
        destination = "/var/lib/polkadot"
        read_only   = false
      }

      template {
        data = <<EOF
#!/bin/sh
{{ with service "polkadot-alice" }}{{ with index . 0 }}
bootnode=/ip4/{{ .Address }}/tcp/{{ .Port }}/p2p/12D3KooWGbgscGKWfHgGXZU42e1BNkCiBHqobhBptWXceuHsL8VL
{{ end }}{{ end }}
exec /usr/bin/polkadot \
  --base-path /var/lib/polkadot \
  --chain rococo-local \
  --bob \
  --rpc-cors all \
  --bootnodes $bootnode
EOF
        change_mode = "restart"
        destination = "local/run.sh"
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

      user = "root"

      config {
        image = "parity/polkadot:v0.9.5"
        entrypoint = ["/local/run.sh"]
        ports = ["p2p", "rpc"]
      }

      resources {
        cpu = 1024
        memory = 8192
      }
    }
  }
}
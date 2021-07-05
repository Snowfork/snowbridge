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
      port "p2p" {}
      port "ws_rpc" {}
      port "http_rpc" {}
      port "prometheus" {}
    }

    task "polkadot" {
      driver = "docker"

      volume_mount {
        volume      = "storage"
        destination = "/var/lib/polkadot"
      }

      service {
        name = "polkadot-alice"
        port = "p2p"
        tags = [
            "alice",
            "traefik.enable=true",
            "traefik.http.routers.polkadot.rule=Host(`polkadot-rpc.snowbridge.network`)",
            "traefik.http.routers.polkadot.entrypoints=websecure",
            "traefik.http.services.polkadot.loadbalancer.server.port=${NOMAD_PORT_ws_rpc}"
        ]
        check {
          type     = "tcp"
          port     = "p2p"
          interval = "10s"
          timeout  = "2s"
        }
        meta {
          ws_rpc_port = "${NOMAD_PORT_ws_rpc}"
          http_rpc_port = "${NOMAD_PORT_http_rpc}"
          prometheus_port = "${NOMAD_PORT_prometheus}"
        }
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
          "--port", "${NOMAD_PORT_p2p}",
          "--ws-port", "${NOMAD_PORT_ws_rpc}",
          "--rpc-port", "${NOMAD_PORT_http_rpc}",
          "--prometheus-port", "${NOMAD_PORT_prometheus}",
          "--node-key", "ac86396c43f8083b41c02fcbcfde161baf42e56e4ff7bc5bb38144825860fe50"
        ]
        ports = ["p2p", "ws_rpc", "http_rpc", "prometheus"]
        network_mode = "host"
      }
      resources {
        cpu = 8000
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
      port "p2p" {}
      port "ws_rpc" {}
      port "http_rpc" {}
      port "prometheus" {}
    }

    task "await-bootnode" {
      driver = "docker"

      config {
        image        = "busybox:1.28"
        command      = "sh"
        args         = ["-c", "until nslookup polkadot-alice.service.dc1.consul 2>&1 >/dev/null; do echo '.'; sleep 5; done"]
        network_mode = "host"
      }

      resources {
        cpu    = 200
        memory = 128
      }

      lifecycle {
        hook    = "prestart"
        sidecar = false
      }
    }

    task "polkadot" {
      driver = "docker"

      volume_mount {
        volume      = "storage"
        destination = "/var/lib/polkadot"
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
  --port {{ env "NOMAD_PORT_p2p" }} \
  --ws-port {{ env "NOMAD_PORT_ws_rpc" }} \
  --rpc-port {{ env "NOMAD_PORT_http_rpc" }} \
  --prometheus-port {{ env "NOMAD_PORT_prometheus" }} \
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
        check {
          type     = "tcp"
          port     = "p2p"
          interval = "10s"
          timeout  = "2s"
        }
        meta {
          ws_rpc_port = "${NOMAD_PORT_ws_rpc}"
          http_rpc_port = "${NOMAD_PORT_http_rpc}"
          prometheus_port = "${NOMAD_PORT_prometheus}"
        }
      }

      user = "root"

      config {
        image = "parity/polkadot:v0.9.5"
        entrypoint = ["/local/run.sh"]
        ports = ["p2p", "ws_rpc", "http_rpc", "prometheus"]
        network_mode = "host"
      }

      resources {
        cpu = 8000
        memory = 8192
      }
    }
  }
}
job "polkadot" {
  datacenters = ["dc1"]
  group "node-0" {

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

    task "validator" {
      driver = "docker"

      volume_mount {
        volume      = "storage"
        destination = "/var/lib/polkadot"
      }

      service {
        name = "polkadot-p2p-0"
        port = "p2p"
        check {
          type     = "tcp"
          port     = "p2p"
          interval = "10s"
          timeout  = "2s"
        }
      }

      service {
        name = "polkadot-rpc"
        port = "ws_rpc"
        tags = [
            "traefik.enable=true",
            "traefik.http.routers.polkadot.rule=Host(`polkadot-rpc.snowbridge.network`)",
            "traefik.http.routers.polkadot.entrypoints=websecure",
            "traefik.http.services.polkadot.loadbalancer.server.port=${NOMAD_PORT_ws_rpc}",
            "traefik.http.services.polkadot.loadbalancer.sticky=true",
            "traefik.http.services.polkadot.loadbalancer.sticky.cookie.name=polkadot",
            "traefik.http.services.polkadot.loadbalancer.sticky.cookie.secure=true",
            "traefik.http.services.polkadot.loadbalancer.sticky.cookie.httpOnly=true"
        ]
        check {
          type     = "tcp"
          port     = "ws_rpc"
          interval = "10s"
          timeout  = "2s"
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
          "--prometheus-external",
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
        cpu = 4000
        memory = 4096
      }
    }
  }

  group "node-1" {

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
        args         = [
          "-c",
          "until nslookup polkadot-p2p-0.service.dc1.consul; do sleep 5; done"
        ]
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

    task "validator" {
      driver = "docker"

      volume_mount {
        volume      = "storage"
        destination = "/var/lib/polkadot"
      }

      template {
        data = <<EOF
{{ with service "polkadot-p2p-0" -}}
  {{ with index . 0 -}}
BOOTNODE=/ip4/{{ .Address }}/tcp/{{ .Port }}/p2p/12D3KooWGbgscGKWfHgGXZU42e1BNkCiBHqobhBptWXceuHsL8VL
  {{- end -}}
{{- end }}
EOF
        env = true
        change_mode = "restart"
        destination = "local/bootnodes.env"
      }

      service {
        name = "polkadot-rpc"
        port = "ws_rpc"
        tags = [
            "traefik.enable=true",
            "traefik.http.routers.polkadot.rule=Host(`polkadot-rpc.snowbridge.network`)",
            "traefik.http.routers.polkadot.entrypoints=websecure",
            "traefik.http.services.polkadot.loadbalancer.server.port=${NOMAD_PORT_ws_rpc}",
            "traefik.http.services.polkadot.loadbalancer.sticky=true",
            "traefik.http.services.polkadot.loadbalancer.sticky.cookie.name=polkadot",
            "traefik.http.services.polkadot.loadbalancer.sticky.cookie.secure=true",
            "traefik.http.services.polkadot.loadbalancer.sticky.cookie.httpOnly=true"
        ]
        check {
          type     = "tcp"
          port     = "ws_rpc"
          interval = "10s"
          timeout  = "2s"
        }
      }

      user = "root"

      config {
        image = "parity/polkadot:v0.9.5"
        args = [
          "--base-path", "/var/lib/polkadot",
          "--chain", "rococo-local",
          "--bob",
          "--rpc-cors", "all",
          "--ws-external",
          "--rpc-external",
          "--prometheus-external",
          "--port", "${NOMAD_PORT_p2p}",
          "--ws-port", "${NOMAD_PORT_ws_rpc}",
          "--rpc-port", "${NOMAD_PORT_http_rpc}",
          "--prometheus-port", "${NOMAD_PORT_prometheus}",
          "--bootnodes", "${BOOTNODE}"
        ]
        ports = ["p2p", "ws_rpc", "http_rpc", "prometheus"]
        network_mode = "host"
      }

      resources {
        cpu = 4000
        memory = 4096
      }
    }
  }
}
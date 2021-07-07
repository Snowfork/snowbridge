job "snowbridge-collator" {
  datacenters = ["dc1"]
  group "node-0" {

    volume "storage" {
      type = "csi"
      source = "snowbridge-node-0"
      attachment_mode = "file-system"
      access_mode = "single-node-writer"
    }

    network {
      port "p2p" {}
      port "ws_rpc" {}
      port "http_rpc" {}
      port "prometheus" {}
    }

    task "collator" {
      driver = "docker"

      volume_mount {
        volume      = "storage"
        destination = "/var/lib/snowbridge"
        read_only   = false
      }

      service {
        name = "snowbridge-p2p-0"
        port = "p2p"
        check {
          type     = "tcp"
          port     = "p2p"
          interval = "10s"
          timeout  = "2s"
        }
      }

      service {
        name = "snowbridge-rpc"
        port = "ws_rpc"
        tags = [
            "traefik.enable=true",
            "traefik.http.routers.collator.rule=Host(`parachain-rpc.snowbridge.network`)",
            "traefik.http.routers.collator.entrypoints=websecure",
            "traefik.http.services.collator.loadbalancer.server.port=${NOMAD_PORT_ws_rpc}",
            "traefik.http.services.collator.loadbalancer.sticky=true",
            "traefik.http.services.collator.loadbalancer.sticky.cookie.name=snowbridge",
            "traefik.http.services.collator.loadbalancer.sticky.cookie.secure=true",
            "traefik.http.services.collator.loadbalancer.sticky.cookie.httpOnly=true"
        ]
        check {
          type     = "tcp"
          port     = "ws_rpc"
          interval = "10s"
          timeout  = "2s"
        }
      }

      artifact {
        source = "s3::https://snowbridge-artifacts.s3.eu-central-1.amazonaws.com/spec.json"
      }

      artifact {
        source = "s3::https://snowbridge-artifacts.s3.eu-central-1.amazonaws.com/rococo-local.json"
      }

      template {
        data = <<EOF
#!/bin/sh
{{ with service "polkadot-alice" }}{{ with index . 0 }}
relay_bootnode=/ip4/{{ .Address }}/tcp/{{ .Port }}/p2p/12D3KooWGbgscGKWfHgGXZU42e1BNkCiBHqobhBptWXceuHsL8VL
{{ end }}{{ end }}

exec /usr/local/bin/snowbridge \
  --base-path /var/lib/snowbridge \
  --alice \
  --node-key f390b6c880d57f2a73b928dc13ddcb86fa595f92ecd6f09bf40160335c6ec459 \
  --chain /local/spec.json \
  --parachain-id 200 \
  --rpc-cors=all \
  --ws-external \
  --rpc-external \
  --port {{ env "NOMAD_PORT_p2p" }} \
  --ws-port {{ env "NOMAD_PORT_ws_rpc" }} \
  --rpc-port {{ env "NOMAD_PORT_http_rpc" }} \
  --prometheus-port {{ env "NOMAD_PORT_prometheus" }} \
  --rpc-methods=Safe \
  --offchain-worker=Always \
  --enable-offchain-indexing=true \
  --execution=native \
  -lruntime=debug \
  -- \
  --chain /local/rococo-local.json \
  --execution=wasm \
  --bootnodes=$relay_bootnode
EOF
        change_mode = "restart"
        destination = "local/run.sh"
        perms = 755
      }

      config {
        image = "ghcr.io/snowfork/snowbridge-collator:0.3.2"
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

  group "node-1" {

    volume "storage" {
      type = "csi"
      source = "snowbridge-node-1"
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
          "until nslookup snowbridge-p2p-0.service.dc1.consul; do sleep 5; done"
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

    task "collator" {
      driver = "docker"

      volume_mount {
        volume      = "storage"
        destination = "/var/lib/snowbridge"
        read_only   = false
      }

      service {
        name = "snowbridge-rpc"
        port = "ws_rpc"
        tags = [
            "traefik.enable=true",
            "traefik.http.routers.collator.rule=Host(`parachain-rpc.snowbridge.network`)",
            "traefik.http.routers.collator.entrypoints=websecure",
            "traefik.http.services.collator.loadbalancer.server.port=${NOMAD_PORT_ws_rpc}",
            "traefik.http.services.collator.loadbalancer.sticky=true",
            "traefik.http.services.collator.loadbalancer.sticky.cookie.name=snowbridge",
            "traefik.http.services.collator.loadbalancer.sticky.cookie.secure=true",
            "traefik.http.services.collator.loadbalancer.sticky.cookie.httpOnly=true"
        ]
        check {
          type     = "tcp"
          port     = "ws_rpc"
          interval = "10s"
          timeout  = "2s"
        }
      }

      artifact {
        source = "s3::https://snowbridge-artifacts.s3.eu-central-1.amazonaws.com/spec.json"
      }

      artifact {
        source = "s3::https://snowbridge-artifacts.s3.eu-central-1.amazonaws.com/rococo-local.json"
      }

      template {
        data = <<EOF
#!/bin/sh
{{ with service "polkadot-p2p-0" }}{{ with index . 0 }}
relay_bootnode=/ip4/{{ .Address }}/tcp/{{ .Port }}/p2p/12D3KooWGbgscGKWfHgGXZU42e1BNkCiBHqobhBptWXceuHsL8VL
{{ end }}{{ end }}

{{ with service "snowbridge-p2p-0" }}{{ with index . 0 }}
para_bootnode=/ip4/{{ .Address }}/tcp/{{ .Port }}/p2p/12D3KooWJxpA4svH4YipQ7Vc8sNfaakBjzfHMUWTtQ2baVx6rtTX
{{ end }}{{ end }}

exec /usr/local/bin/snowbridge \
  --base-path /var/lib/snowbridge \
  --bob \
  --chain /local/spec.json \
  --parachain-id 200 \
  --rpc-cors=all \
  --ws-external \
  --rpc-external \
  --port {{ env "NOMAD_PORT_p2p" }} \
  --ws-port {{ env "NOMAD_PORT_ws_rpc" }} \
  --rpc-port {{ env "NOMAD_PORT_http_rpc" }} \
  --prometheus-port {{ env "NOMAD_PORT_prometheus" }} \
  --rpc-methods=Safe \
  --offchain-worker=Always \
  --enable-offchain-indexing=true \
  --execution=native \
  --bootnodes=$para_bootnode \
  -lruntime=debug \
  -- \
  --chain /local/rococo-local.json \
  --execution=wasm \
  --bootnodes=$relay_bootnode
EOF
        change_mode = "restart"
        destination = "local/run.sh"
        perms = 755
      }

      config {
        image = "ghcr.io/snowfork/snowbridge-collator:0.3.2"
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
job "snowbridge-relay" {
  datacenters = ["dc1"]

  group "main" {
    count = 1

    task "relay" {
      driver = "docker"

      service {
        name = "snowbridge-relay"
      }

      artifact {
        source = "s3::https://snowbridge-artifacts.s3.eu-central-1.amazonaws.com/relay-config.toml.tpl"
      }

      template {
        source = "local/relay-config.toml.tpl"
        destination = "local/config.toml"
      }

      vault {
        policies = ["snowbridge"]
      }

      template {
        data = <<EOF
{{with secret "secret/data/relay" -}}
SNOWBRIDGE_BEEFY_KEY="{{.Data.data.beefy_key}}"
SNOWBRIDGE_MESSAGE_KEY="{{.Data.data.message_key}}"
SNOWBRIDGE_PARACHAIN_KEY="{{.Data.data.parachain_key}}"
SNOWBRIDGE_RELAYCHAIN_KEY="{{.Data.data.relaychain_key}}"
{{end -}}
EOF
        env = true
        change_mode = "restart"
        destination = "secrets/keys.env"
      }

      config {
        image = "ghcr.io/snowfork/snowbridge-relay:0.3.2"
        args = [
          "run",
          "--data-dir", "/var/lib/snowbridge-relay",
          "--config", "local/config.toml"
        ]
      }

      resources {
        cpu = 2000
        memory = 2048
      }

      kill_timeout = "60s"
    }
  }
}

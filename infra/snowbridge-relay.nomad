job "snowbridge-relay" {
  datacenters = ["dc1"]

  group "main" {
    count = 1

    task "plugin" {
      driver = "docker"

      artifact {
        source = "s3::https://snowbridge-artifacts.s3.eu-central-1.amazonaws.com/relay-config.toml.tpl"
        destination = "local/config.toml.tpl"
      }

      template {
        source = "local/config.toml.tpl"
        destination = "local/config.toml"
      }

      vault {
        policies = ["snowbridge"]
      }

      template {
        data = <<EOF
KEY="{{with secret "secret/data/relay"}}{{.Data.data.foo}}{{end}}"
BEEFY_RELAYER_ETHEREUM_KEY="<foo>"
PARACHAIN_COMMITMENT_RELAYER_ETHEREUM_KEY="<bar>"
ARTEMIS_PARACHAIN_KEY="//Relay"
ARTEMIS_RELAYCHAIN_KEY="//Alice"

EOF
        env = true
        change_mode = "restart"
        destination = "secrets/keys.env"
      }

      config {
        image = "ghcr.io/snowfork/snowbridge-relay:0.3.2"
        entrypoint = "/bin/sh"
        args = [
          "run", "--config", "local/config.toml"
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
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

      config {
        image = "ghcr.io/snowfork/snowbridge-relay:0.3.2"
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
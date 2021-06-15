job "traefik" {
  region      = "global"
  datacenters = ["dc1"]
  type        = "service"

  group "traefik" {
    count = 1

    volume "certs" {
      type      = "host"
      read_only = false
      source    = "certs"
    }

    network {
      port "http" {
        static = 80
      }
      port "api" {
        static = 8080
      }
    }

    service {
      name = "traefik"

      check {
        name     = "alive"
        type     = "tcp"
        port     = "http"
        interval = "10s"
        timeout  = "2s"
      }

      tags = [
        "traefik.enable=true",
        "traefik.http.routers.dashboard.rule=Host(`traefik.snowbridge.network`)",
        "traefik.http.routers.dashboard.entrypoints=websecure",
        "traefik.http.routers.dashboard.service=api@internal",
        "traefik.http.routers.dashboard.middlewares=auth@file",
      ]
    }

    task "traefik" {
      driver = "docker"

      volume_mount {
        volume      = "certs"
        destination = "/certs"
        read_only   = false
      }

      config {
        image        = "traefik:v2.4.8"
        network_mode = "host"

        volumes = [
          "local/traefik.toml:/etc/traefik/traefik.toml",
          "local/dynamic-config.toml:/etc/traefik/dynamic-config.toml",
        ]
      }

      template {
        data = <<EOF
[providers]
  [providers.file]
    filename = "/etc/traefik/dynamic-config.toml"

[entryPoints]
    [entryPoints.web]
        address = ":80"
        [entryPoints.web.http.redirections.entryPoint]
            to = "websecure"
            scheme = "https"
    [entryPoints.websecure]
        address = ":443"
    [entryPoints.websecure.http.tls]
        certResolver = "myresolver"

[api]
    dashboard = true

# Enable Consul Catalog configuration backend.
[providers.consulCatalog]
    prefix           = "traefik"
    exposedByDefault = false
    [providers.consulCatalog.endpoint]
        address = "127.0.0.1:8500"
        scheme  = "http"

[certificatesResolvers.myresolver.acme]
    email = "vincent.geddes@hey.com"
    storage = "/certs/acme.json"
    [certificatesResolvers.myresolver.acme.tlsChallenge]
EOF

        destination = "local/traefik.toml"
      }

     template {
        data = <<EOF
[http.middlewares]
  [http.middlewares.auth.basicAuth]
    users = ["admin:$apr1$WJKOCk0U$SKZGoH6xn0Tdr4yvmJcBT."]

[http.routers]
  [http.routers.consul]
    rule = "Host(`consul.snowbridge.network`)"
    entryPoints = ["websecure"]
    service = "consul"
    middlewares = ["auth"]
  [http.routers.nomad]
    rule = "Host(`nomad.snowbridge.network`)"
    entryPoints = ["websecure"]
    service = "nomad"

[http.services]
  [http.services.consul.loadBalancer]
    [[http.services.consul.loadBalancer.servers]]
      url = "http://172.31.44.209:8500/"
  [http.services.nomad.loadBalancer]
    [[http.services.nomad.loadBalancer.servers]]
      url = "http://172.31.37.41:4646/"
EOF

        destination = "local/dynamic-config.toml"
      }

      resources {
        cpu    = 500
        memory = 512
      }
    }
  }
}
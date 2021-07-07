job "traefik" {
  region      = "global"
  datacenters = ["dc1"]
  type        = "service"

  group "traefik" {
    count = 1

    constraint {
      attribute = "${node.unique.id}"
      value     = "cb240190-d0bb-f889-547e-2cd59822248e"
    }

    volume "certs" {
      type = "csi"
      source = "traefik-certs"
      attachment_mode = "file-system"
      access_mode = "single-node-writer"
    }

    network {
      port "http" {
        static = 80
      }
      port "https" {
        static = 443
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

        args = [
          "--configFile=local/traefik.toml"
        ]
      }

      template {
        data = <<EOF
[providers]
  [providers.file]
    filename = "/local/dynamic-config.toml"

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
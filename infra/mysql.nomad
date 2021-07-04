job "mysql-server" {
  datacenters = ["dc1"]
  type        = "service"

  group "mysql-server" {
    count = 1

    volume "mysql" {
      type      = "csi"
      source    = "testvol"
      attachment_mode = "file-system"
      access_mode     = "single-node-writer"
    }

    network {
      port "db" {
        static = 3306
      }
    }

    restart {
      attempts = 10
      interval = "5m"
      delay    = "25s"
      mode     = "delay"
    }

    task "mysql-server" {
      driver = "docker"

      volume_mount {
        volume      = "mysql"
        destination = "/srv"
        read_only   = false
      }

      env {
        MYSQL_ROOT_PASSWORD = "password"
      }

      config {
        image = "hashicorp/mysql-portworx-demo:latest"
        args  = ["--datadir", "/srv/mysql"]
        ports = ["db"]
      }

      resources {
        cpu    = 500
        memory = 1024
      }

      service {
        name = "mysql-server"
        port = "db"

        check {
          type     = "tcp"
          interval = "10s"
          timeout  = "2s"
        }
      }
    }
  }
}
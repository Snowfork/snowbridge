variable "csi_volume_id" {
  type = string
}

job "clear-volume" {
  datacenters = ["dc1"]

  type = "batch"

  group "main" {
    count = 1

    volume "source" {
      type = "csi"
      source = var.csi_volume_id
      attachment_mode = "file-system"
      access_mode = "single-node-writer"
    }

    task "wiper-script" {

      driver = "docker"

      volume_mount {
        volume      = "source"
        destination = "/srv"
      }

      config {
        image = "ubuntu:20.04"
        command = "rm"
        args = ["-rfv", "/srv/chains"]
      }

      resources {
        cpu    = 100
        memory = 256
      }
    }
  }
}
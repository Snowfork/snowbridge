job "clear-polkadot-volume" {
  datacenters = ["dc1"]

  type = "batch"

  group "main" {
    count = 1

    volume "source0" {
      type = "csi"
      source = "polkadot-node-0"
      attachment_mode = "file-system"
      access_mode = "single-node-writer"
    }

    volume "source1" {
      type = "csi"
      source = "polkadot-node-1"
      attachment_mode = "file-system"
      access_mode = "single-node-writer"
    }

    task "wiper-script" {

      driver = "docker"

      volume_mount {
        volume      = "source0"
        destination = "/mnt/vol0"
      }

      volume_mount {
        volume      = "source1"
        destination = "/mnt/vol1"
      }

      config {
        image = "ubuntu:20.04"
        command = "rm"
        args = ["-rfv", "/mnt/vol0/chains", "/mnt/vol1/chains"]
      }

      resources {
        cpu    = 100
        memory = 256
      }
    }
  }
}
job "plugin-aws-ebs-controller" {
  datacenters = ["dc1"]

  group "controller" {
    count = 2

    constraint {
      distinct_hosts = true
    }

    task "plugin" {
      driver = "docker"

      config {
        image = "amazon/aws-ebs-csi-driver:v1.1.0"

        args = [
          "controller",
          "--endpoint=unix://csi/csi.sock",
          "--logtostderr",
          "--v=5",
        ]
      }

      csi_plugin {
        id        = "aws-ebs0"
        type      = "controller"
        mount_dir = "/csi"
      }

      resources {
        memory = 256
      }

      kill_timeout = "60s"
    }
  }
}

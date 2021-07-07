job "plugin-aws-ebs-nodes" {
  datacenters = ["dc1"]

  type = "system"

  group "nodes" {
    task "plugin" {
      driver = "docker"

      config {
        image = "amazon/aws-ebs-csi-driver:v1.1.0"

        args = [
          "node",
          "--endpoint=unix://csi/csi.sock",
          "--logtostderr",
          "--v=5",
        ]

        privileged = true
      }

      csi_plugin {
        id        = "aws-ebs0"
        type      = "node"
        mount_dir = "/csi"
      }

      resources {
        memory = 256
      }

      kill_timeout = "60s"
    }
  }
}

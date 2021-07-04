job "plugin-aws-ebs-nodes" {
  datacenters = ["dc1"]

  # you can run node plugins as service jobs as well, but this ensures
  # that all nodes in the DC have a copy.
  type = "system"

  group "nodes" {
    task "plugin" {
      driver = "docker"

      config {
        image = "amazon/aws-ebs-csi-driver:v0.10.1"

        args = [
          "node",
          "--endpoint=unix://csi/csi.sock",
          "--logtostderr",
          "--v=5",
        ]

        # node plugins must run as privileged jobs because they
        # mount disks to the host
        privileged = true
      }

      csi_plugin {
        id        = "aws-ebs0"
        type      = "node"
        mount_dir = "/csi"
      }

      resources {
        cpu    = 500
        memory = 256
      }
    }
  }
}
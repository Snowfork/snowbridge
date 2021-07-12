id = "relay"
name = "relay"
type = "csi"
external_id = "vol-0d8de0c99cb1c6f68"
plugin_id = "aws-ebs0"

capability {
  access_mode     = "single-node-writer"
  attachment_mode = "file-system"
}

mount_options {
  fs_type     = "ext4"
}

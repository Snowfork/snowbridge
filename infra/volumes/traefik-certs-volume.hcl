id = "traefik-certs"
name = "traefik-certs"
type = "csi"
external_id = "vol-09c374888524b20ee"
plugin_id = "aws-ebs0"

capability {
  access_mode     = "single-node-writer"
  attachment_mode = "file-system"
}

mount_options {
  fs_type     = "ext4"
}
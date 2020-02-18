// Terraform plugin for creating random ids
resource "random_id" "instance_id" {
  byte_length = 8
}

// A single Google Cloud Engine instance
resource "digitalocean_droplet" "default" {
  name     = "kusama-${random_id.instance_id.hex}"
  size     = var.machine_type
  region   = var.zone_name
  image    = var.image_name
  monitoring = true
  ipv6     = true
  private_networking = true

   ssh_keys = [
     var.ssh_fingerprint
     ]

  provisioner "file" {
    source      = var.script_path
    destination = "/tmp/setup.sh"


    connection {
      type        = "ssh"
      user        = var.username
      private_key = file(var.private_key_path)
      timeout     = "2m"
      host = digitalocean_droplet.default.ipv4_address

    }
  }

  provisioner "remote-exec" {
    inline = [
      "chmod +x /tmp/setup.sh",
      "/tmp/setup.sh"
    ]
    connection {
      type        = "ssh"
      user        = var.username
      private_key = file(var.private_key_path)
      timeout     = "4m"
      host = digitalocean_droplet.default.ipv4_address

    }
  }

}

// A variable for extracting the external ip of the instance
output "ip" {
  value     = digitalocean_droplet.default.ipv4_address
}

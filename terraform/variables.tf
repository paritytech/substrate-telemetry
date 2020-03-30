variable "project_name" {
  type        = string
  description = "The name of the project to instantiate the instance at. Make sure this matches the project ID on GCP"
}

variable "zone_name" {
  type        = string
  description = "the zone that this terraform configuration will instantiate at."
  default     = "ams3"
}

variable "machine_type" {
  type        = string
  description = "The machine type that this instance will be"
  default     = "s-4vcpu-8gb"
}

variable "image_name" {
  type        = string
  default     = "docker-18-04"
  description = "The image type that the instance runs"
}

variable "image_size" {
  type        = string
  description = "The disk size the image uses"
  default     = "100"
}

variable "script_path" {
  type        = string
  description = "Where is the path to the script locally on the machine"
}

variable "public_key_path" {
  type        = string
  description = "The path to the public ssh key used to connect to the instance"
}

variable "private_key_path" {
  type        = string
  description = "The path to the private key used to connect to the instance"
}

variable "do_token" {
  type        = string
  description = "DigitalOcean API token"
}

variable "ssh_fingerprint" {
  type        = string
  description = "Fingerprint of the SSH key"
}

variable "username" {
  type        = string
  description = "The name of the user that will be used to remote exec the script"
}

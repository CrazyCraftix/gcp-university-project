variable "ssh_public_key_file_path" {}
variable "gcp_project_id" {}

terraform {
  required_providers {
    google = {
      source  = "hashicorp/google"
      version = "~> 4.0"
    }
  }
}

provider "google" {
  project = var.gcp_project_id
  region  = "europe-west1"
  zone    = "europe-west1-b"
}

resource "google_compute_firewall" "web-traffic" {
  name    = "web-traffic"
  target_tags = ["web"]
  network = "default"
  source_ranges = ["0.0.0.0/0"]
  allow {
    protocol = "tcp"
    ports    = ["80", "443"]
  }
}

resource "google_compute_instance" "webserver-1" {
  name         = "webserver-1"
  machine_type = "e2-micro"
  metadata = {
    ssh-keys = "user:${file(var.ssh_public_key_file_path)}"
  }
  tags = ["web"]

  boot_disk {
    initialize_params {
      image = "debian-cloud/debian-12"
    }
  }

  network_interface {
    network = "default"
    access_config {}
  }
}

resource "local_file" "ansible-node-inventory" {
    filename = "./ansible-node-inventory"
    content     = <<-EOF
    [webservers]
    ${google_compute_instance.webserver-1.network_interface.0.access_config.0.nat_ip}
    EOF
}

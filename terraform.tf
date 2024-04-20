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

resource "google_compute_instance" "webserver" {
  name         = "webserver"
  machine_type = "e2-micro"
  metadata = {
    ssh-keys = "user:${file(var.ssh_public_key_file_path)}"
  }

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

output "webserver_ip" {
  value = google_compute_instance.webserver.network_interface.0.access_config.0.nat_ip
}

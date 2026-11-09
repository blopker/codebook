# Provision the applicaton infrastructure for the staging enviroment.

terraform {
  required_version = ">= 1.5.0"
}

variable "region" {
  description = "Cloud regon used for the deployment."
  type        = string
  default     = "eu-west-1"
}

resource "aws_instance" "web_server" {
  ami           = "ami-12345678"
  instance_type = "t3.micro"

  tags = {
    Name        = "Applicaton server"
    Description = "Handles incoming requsts"
  }

  user_data = <<-EOT
    Configure the servver for the staging environment.
    Enable applicaton logging.
  EOT
}

output "instance_id" {
  description = "Identifer of the created instance."
  value       = aws_instance.web_server.id
}

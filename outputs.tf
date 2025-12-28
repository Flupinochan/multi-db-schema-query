output "ec2_public_ip" {
  value = aws_instance.main.public_ip
}

output "rds_endpoint" {
  value = aws_db_instance.default.endpoint
}

output "rds_username" {
  value = var.db_username
}

output "rds_password" {
  value = var.db_password
}
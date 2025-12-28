variable "region" {
  type    = string
  default = "us-east-1"
}

variable "instance_type" {
  type    = string
  default = "t3.micro"
}

variable "key_pair_name" {
  description = "Create a key pair in advance via the EC2 console screen and download it"
  type        = string
  default     = "multi-db-schema-query"
}

variable "db_instance_class" {
  type    = string
  default = "db.t3.micro"
}

variable "db_username" {
  type    = string
  default = "admin"
}

variable "db_password" {
  type    = string
  default = "password1!"
}

variable "db_name" {
  type    = string
  default = "mydb"
}
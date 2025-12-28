terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.92"
    }
  }

  required_version = ">= 1.2"
}

provider "aws" {
  region = var.region
}

# Most Recent AMI ID
data "aws_ami" "amazon_linux" {
  most_recent = true
  owners      = ["amazon"]

  filter {
    name   = "name"
    values = ["al2023-ami-*-x86_64"]
  }
}

# EC2 Security Group
resource "aws_security_group" "ssh" {
  name        = "allow-ssh"
  description = "Allow SSH inbound traffic"

  tags = {
    Name = "allow-ssh"
  }
}

resource "aws_security_group_rule" "ssh_ingress" {
  type              = "ingress"
  from_port         = 22
  to_port           = 22
  protocol          = "tcp"
  cidr_blocks       = ["0.0.0.0/0"]
  security_group_id = aws_security_group.ssh.id
}

resource "aws_security_group_rule" "ssh_egress" {
  type              = "egress"
  from_port         = 0
  to_port           = 0
  protocol          = "-1"
  cidr_blocks       = ["0.0.0.0/0"]
  security_group_id = aws_security_group.ssh.id
}

# EC2
resource "aws_instance" "main" {
  ami                    = data.aws_ami.amazon_linux.id
  instance_type          = var.instance_type
  key_name               = var.key_pair_name
  vpc_security_group_ids = [aws_security_group.ssh.id]
  user_data              = <<-EOF
              #!/bin/bash
              sudo dnf install -y mariadb105
              EOF

  tags = {
    Name = "multi-db-schema-query"
  }

  timeouts {
    create = "5m"
  }
}

# RDS Security Group
resource "aws_security_group" "rds" {
  name        = "allow-mysql-from-ec2"
  description = "Allow MySQL from EC2"

  tags = {
    Name = "allow-mysql-from-ec2"
  }
}

resource "aws_security_group_rule" "rds_ingress" {
  type                     = "ingress"
  from_port                = 3306
  to_port                  = 3306
  protocol                 = "tcp"
  security_group_id        = aws_security_group.rds.id
  source_security_group_id = aws_security_group.ssh.id
}

resource "aws_security_group_rule" "rds_egress" {
  type              = "egress"
  from_port         = 0
  to_port           = 0
  protocol          = "-1"
  cidr_blocks       = ["0.0.0.0/0"]
  security_group_id = aws_security_group.rds.id
}

# RDS
resource "aws_db_instance" "default" {
  allocated_storage      = 10
  db_name                = var.db_name
  engine                 = "mysql"
  instance_class         = var.db_instance_class
  username               = var.db_username
  password               = var.db_password
  skip_final_snapshot    = true
  deletion_protection    = false
  vpc_security_group_ids = [aws_security_group.rds.id]
  publicly_accessible    = false

  timeouts {
    create = "10m"
    delete = "30m"
  }
}
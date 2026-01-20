# Database Module - RDS PostgreSQL (t3.micro for cost optimization)

variable "project" {}
variable "environment" {}
variable "vpc_id" {}
variable "subnet_ids" { type = list(string) }
variable "db_password" { sensitive = true }
variable "instance_class" { default = "db.t3.micro" }

resource "aws_security_group" "db" {
  name_prefix = "${var.project}-${var.environment}-db-"
  vpc_id      = var.vpc_id

  ingress {
    from_port   = 5432
    to_port     = 5432
    protocol    = "tcp"
    cidr_blocks = ["10.0.0.0/16"]
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  lifecycle { create_before_destroy = true }
}

resource "aws_db_subnet_group" "main" {
  name       = "${lower(var.project)}-${var.environment}"
  subnet_ids = var.subnet_ids
}

resource "aws_db_instance" "main" {
  identifier     = "${lower(var.project)}-${var.environment}"
  engine         = "postgres"
  engine_version = "15"
  instance_class = var.instance_class

  allocated_storage = 20
  storage_type      = "gp2"

  db_name  = "coffee_qm"
  username = "cqm_admin"
  password = var.db_password

  db_subnet_group_name   = aws_db_subnet_group.main.name
  vpc_security_group_ids = [aws_security_group.db.id]

  publicly_accessible    = false
  skip_final_snapshot    = true
  backup_retention_period = 1
  deletion_protection    = false

  tags = { Component = "database" }
}

output "endpoint" {
  value = aws_db_instance.main.endpoint
}

output "connection_string" {
  value     = "postgres://cqm_admin:${var.db_password}@${aws_db_instance.main.endpoint}/coffee_qm"
  sensitive = true
}

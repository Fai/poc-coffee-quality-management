terraform {
  required_version = ">= 1.5.0"

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }

  backend "s3" {
    bucket         = "coffee-qm-terraform-state"
    key            = "test/terraform.tfstate"
    region         = "ap-southeast-1"
    encrypt        = true
    dynamodb_table = "coffee-qm-terraform-locks"
  }
}

provider "aws" {
  region = var.aws_region

  default_tags {
    tags = {
      Project     = var.project_name
      Environment = var.environment
      CostCenter  = var.cost_center
      ManagedBy   = "terraform"
    }
  }
}

# Variables
variable "aws_region" {
  default = "ap-southeast-1"
}

variable "project_name" {
  default = "CoffeeQualityManagement"
}

variable "environment" {
  default = "test"
}

variable "cost_center" {
  default = "CC-COFFEE-001"
}

variable "db_password" {
  sensitive = true
}

# Modules
module "vpc" {
  source      = "../../modules/vpc"
  project     = var.project_name
  environment = var.environment
}

module "database" {
  source        = "../../modules/database"
  project       = var.project_name
  environment   = var.environment
  vpc_id        = module.vpc.vpc_id
  subnet_ids    = module.vpc.public_subnet_ids
  db_password   = var.db_password
  instance_class = "db.t3.micro"
}

module "ecs" {
  source           = "../../modules/ecs"
  project          = var.project_name
  environment      = var.environment
  vpc_id           = module.vpc.vpc_id
  subnet_ids       = module.vpc.public_subnet_ids
  database_url     = module.database.connection_string
  use_fargate_spot = true
}

module "ai" {
  source      = "../../modules/ai"
  project     = var.project_name
  environment = var.environment
}

# Cost tracking - AWS Budget
resource "aws_budgets_budget" "project_budget" {
  name         = "${var.project_name}-${var.environment}-monthly"
  budget_type  = "COST"
  limit_amount = "50"
  limit_unit   = "USD"
  time_unit    = "MONTHLY"

  cost_filter {
    name   = "TagKeyValue"
    values = ["user:Project$${var.project_name}"]
  }

  notification {
    comparison_operator       = "GREATER_THAN"
    threshold                 = 80
    threshold_type           = "PERCENTAGE"
    notification_type        = "ACTUAL"
    subscriber_sns_topic_arns = []
  }
}

# Cost and Usage Report (for detailed tracking)
resource "aws_cur_report_definition" "project_report" {
  count = var.environment == "test" ? 1 : 0

  report_name                = "${var.project_name}-cost-report"
  time_unit                 = "DAILY"
  format                    = "Parquet"
  compression               = "Parquet"
  additional_schema_elements = ["RESOURCES", "SPLIT_COST_ALLOCATION_DATA"]
  s3_bucket                 = aws_s3_bucket.cost_reports[0].id
  s3_prefix                 = "reports"
  s3_region                 = var.aws_region
  report_versioning         = "OVERWRITE_REPORT"
}

resource "aws_s3_bucket" "cost_reports" {
  count  = var.environment == "test" ? 1 : 0
  bucket = "${lower(var.project_name)}-${var.environment}-cost-reports-${data.aws_caller_identity.current.account_id}"
}

resource "aws_s3_bucket_policy" "cost_reports" {
  count  = var.environment == "test" ? 1 : 0
  bucket = aws_s3_bucket.cost_reports[0].id
  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Principal = { Service = "billingreports.amazonaws.com" }
        Action   = ["s3:GetBucketAcl", "s3:GetBucketPolicy"]
        Resource = aws_s3_bucket.cost_reports[0].arn
      },
      {
        Effect = "Allow"
        Principal = { Service = "billingreports.amazonaws.com" }
        Action   = "s3:PutObject"
        Resource = "${aws_s3_bucket.cost_reports[0].arn}/*"
      }
    ]
  })
}

data "aws_caller_identity" "current" {}

# Outputs
output "api_url" {
  value = module.ecs.api_url
}

output "ai_endpoint" {
  value = module.ai.api_endpoint
}

output "database_endpoint" {
  value     = module.database.endpoint
  sensitive = true
}

output "cost_tracking" {
  value = {
    budget_name    = aws_budgets_budget.project_budget.name
    cost_filter    = "Project=${var.project_name}"
    monthly_limit  = "$50 USD"
  }
}

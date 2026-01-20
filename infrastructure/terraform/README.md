# Terraform Infrastructure

## Structure

```
terraform/
├── bootstrap/          # Run once to create state backend
├── modules/
│   ├── vpc/           # Network (no NAT for cost savings)
│   ├── database/      # RDS PostgreSQL t3.micro
│   ├── ecs/           # Fargate Spot cluster
│   └── ai/            # Lambda + API Gateway
└── environments/
    ├── test/          # Test environment config
    └── prod/          # Production config (future)
```

## Quick Start

### 1. Bootstrap (one-time)

```bash
cd infrastructure/terraform/bootstrap
terraform init
terraform apply
```

This creates:
- S3 bucket for state: `coffee-qm-terraform-state`
- DynamoDB table for locking: `coffee-qm-terraform-locks`

### 2. Deploy Test Environment

```bash
cd infrastructure/terraform/environments/test

# Initialize
terraform init

# Preview changes
terraform plan -var="db_password=YourSecurePassword123"

# Deploy
terraform apply -var="db_password=YourSecurePassword123"
```

### 3. View Costs

```bash
# Show resources and their tags
terraform state list

# Show specific resource
terraform state show module.database.aws_db_instance.main
```

## Cost Tracking

All resources are tagged with:
- `Project = CoffeeQualityManagement`
- `Environment = test`
- `CostCenter = CC-COFFEE-001`
- `ManagedBy = terraform`

The deployment creates:
- AWS Budget alert at $50/month
- Cost and Usage Report (daily, to S3)

### View Costs in AWS Console

1. Go to **AWS Cost Explorer**
2. Filter by tag: `Project = CoffeeQualityManagement`
3. Group by: `Service` or `Environment`

### CLI Cost Query

```bash
aws ce get-cost-and-usage \
  --time-period Start=2024-01-01,End=2024-01-31 \
  --granularity MONTHLY \
  --metrics UnblendedCost \
  --filter '{"Tags":{"Key":"Project","Values":["CoffeeQualityManagement"]}}' \
  --group-by Type=DIMENSION,Key=SERVICE
```

## Destroy (Clean Up)

```bash
cd infrastructure/terraform/environments/test

# Preview what will be deleted
terraform plan -destroy -var="db_password=x"

# Destroy all resources
terraform destroy -var="db_password=x"
```

This removes ALL resources and stops ALL costs for this project.

## Version History

Terraform state tracks all changes. View history:

```bash
# List state versions (in S3)
aws s3api list-object-versions \
  --bucket coffee-qm-terraform-state \
  --prefix test/terraform.tfstate

# Restore previous version if needed
terraform state pull > backup.tfstate
```

## Estimated Costs

| Resource | Monthly Cost |
|----------|-------------|
| RDS t3.micro | $12 |
| ECS Fargate Spot | $5 |
| ALB | $16 |
| Lambda AI | $3 |
| S3 + Logs | $2 |
| **Total** | **~$38/month** |

#!/bin/bash
# Run database migrations
set -e

PROJECT_NAME="CoffeeQualityManagement"
ENVIRONMENT="${1:-test}"
AWS_REGION="${AWS_REGION:-ap-southeast-1}"
STACK_NAME="${PROJECT_NAME}-${ENVIRONMENT}"

echo "Running database migrations for ${ENVIRONMENT}..."

# Get database endpoint
DB_HOST=$(aws cloudformation describe-stacks \
    --stack-name "$STACK_NAME" \
    --query "Stacks[0].Outputs[?OutputKey=='DatabaseEndpoint'].OutputValue" \
    --output text \
    --region "$AWS_REGION")

# Get password from secrets manager
DB_PASSWORD=$(aws secretsmanager get-secret-value \
    --secret-id "${PROJECT_NAME}/${ENVIRONMENT}/db-password" \
    --query "SecretString" \
    --output text \
    --region "$AWS_REGION")

export DATABASE_URL="postgres://cqm_admin:${DB_PASSWORD}@${DB_HOST}:5432/coffee_qm"

echo "Running migrations..."
cd backend
sqlx migrate run --source migrations

echo "=========================================="
echo "Migrations complete!"
echo "=========================================="

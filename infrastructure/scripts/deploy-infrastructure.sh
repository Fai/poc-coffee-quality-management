#!/bin/bash
# Deploy cost-optimized test environment
set -e

PROJECT_NAME="CoffeeQualityManagement"
ENVIRONMENT="${1:-test}"
AWS_REGION="${AWS_REGION:-ap-southeast-1}"
STACK_NAME="${PROJECT_NAME}-${ENVIRONMENT}"

echo "=========================================="
echo "Deploying ${PROJECT_NAME} - ${ENVIRONMENT} (Cost-Optimized)"
echo "Estimated cost: ~$36/month"
echo "=========================================="

command -v aws >/dev/null 2>&1 || { echo "AWS CLI required"; exit 1; }

if [ -z "$DB_PASSWORD" ]; then
    read -sp "Enter database password (min 8 chars): " DB_PASSWORD
    echo
fi

# Deploy optimized stack
echo "Deploying CloudFormation stack..."
aws cloudformation deploy \
    --template-file infrastructure/cloudformation/test-environment-optimized.yaml \
    --stack-name "$STACK_NAME" \
    --parameter-overrides \
        Environment="$ENVIRONMENT" \
        ProjectName="$PROJECT_NAME" \
        DatabasePassword="$DB_PASSWORD" \
    --capabilities CAPABILITY_IAM \
    --region "$AWS_REGION" \
    --tags \
        Project=$PROJECT_NAME \
        Environment=$ENVIRONMENT \
        CostCenter=CC-COFFEE-001

# Get outputs
API_URL=$(aws cloudformation describe-stacks \
    --stack-name "$STACK_NAME" \
    --query "Stacks[0].Outputs[?OutputKey=='APIURL'].OutputValue" \
    --output text --region "$AWS_REGION")

ECR_URI=$(aws cloudformation describe-stacks \
    --stack-name "$STACK_NAME" \
    --query "Stacks[0].Outputs[?OutputKey=='ECRRepository'].OutputValue" \
    --output text --region "$AWS_REGION")

echo "=========================================="
echo "Deployment complete!"
echo "API: $API_URL"
echo "ECR: $ECR_URI"
echo ""
echo "Next: ./build-and-push.sh $ENVIRONMENT"
echo "=========================================="

#!/bin/bash
# Build and push Docker image to ECR
set -e

PROJECT_NAME="CoffeeQualityManagement"
ENVIRONMENT="${1:-test}"
AWS_REGION="${AWS_REGION:-ap-southeast-1}"
STACK_NAME="${PROJECT_NAME}-${ENVIRONMENT}"

echo "Building and pushing backend image..."

# Get ECR repository URI
ECR_URI=$(aws cloudformation describe-stacks \
    --stack-name "$STACK_NAME" \
    --query "Stacks[0].Outputs[?OutputKey=='ECRRepositoryUri'].OutputValue" \
    --output text \
    --region "$AWS_REGION")

if [ -z "$ECR_URI" ]; then
    echo "Error: Could not get ECR URI. Is the stack deployed?"
    exit 1
fi

# Login to ECR
aws ecr get-login-password --region "$AWS_REGION" | \
    docker login --username AWS --password-stdin "${ECR_URI%%/*}"

# Build image
echo "Building Docker image..."
docker build -t "${PROJECT_NAME}-backend:latest" -f backend/Dockerfile .

# Tag and push
IMAGE_TAG="${ECR_URI}:latest"
docker tag "${PROJECT_NAME}-backend:latest" "$IMAGE_TAG"

echo "Pushing to ECR..."
docker push "$IMAGE_TAG"

# Also tag with git commit hash if available
if command -v git &> /dev/null && git rev-parse HEAD &> /dev/null; then
    GIT_HASH=$(git rev-parse --short HEAD)
    docker tag "${PROJECT_NAME}-backend:latest" "${ECR_URI}:${GIT_HASH}"
    docker push "${ECR_URI}:${GIT_HASH}"
    echo "Also pushed with tag: ${GIT_HASH}"
fi

# Update ECS service to use new image
echo "Updating ECS service..."
CLUSTER_NAME=$(aws cloudformation describe-stacks \
    --stack-name "$STACK_NAME" \
    --query "Stacks[0].Outputs[?OutputKey=='ECSClusterName'].OutputValue" \
    --output text \
    --region "$AWS_REGION")

SERVICE_NAME=$(aws cloudformation describe-stacks \
    --stack-name "$STACK_NAME" \
    --query "Stacks[0].Outputs[?OutputKey=='ECSServiceName'].OutputValue" \
    --output text \
    --region "$AWS_REGION")

aws ecs update-service \
    --cluster "$CLUSTER_NAME" \
    --service "$SERVICE_NAME" \
    --force-new-deployment \
    --region "$AWS_REGION" \
    --no-cli-pager

echo "=========================================="
echo "Backend deployment initiated!"
echo "Monitor progress: aws ecs describe-services --cluster $CLUSTER_NAME --services $SERVICE_NAME"
echo "=========================================="

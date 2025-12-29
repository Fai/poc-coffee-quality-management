#!/bin/bash
# Deploy to shared infrastructure

PROJECT_NAME="cqm"
ENVIRONMENT="sandbox"

echo "🚀 Deploying ${PROJECT_NAME} to shared infrastructure..."

# Build and push Docker image
docker build -t ${PROJECT_NAME}-shared .
aws ecr get-login-password --region us-east-1 | docker login --username AWS --password-stdin ${AWS_ACCOUNT_ID}.dkr.ecr.us-east-1.amazonaws.com
docker tag ${PROJECT_NAME}-shared:latest ${AWS_ACCOUNT_ID}.dkr.ecr.us-east-1.amazonaws.com/${PROJECT_NAME}-shared:latest
docker push ${AWS_ACCOUNT_ID}.dkr.ecr.us-east-1.amazonaws.com/${PROJECT_NAME}-shared:latest

# Update ECS service
aws ecs update-service \
  --cluster ${ENVIRONMENT}-shared-cluster \
  --service ${PROJECT_NAME}-backend \
  --task-definition ${PROJECT_NAME}-shared:latest \
  --force-new-deployment

echo "✅ ${PROJECT_NAME} deployed to shared infrastructure"

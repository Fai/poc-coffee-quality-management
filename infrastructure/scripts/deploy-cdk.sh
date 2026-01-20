#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
CDK_DIR="${SCRIPT_DIR}/../cdk"
FRONTEND_DIR="${SCRIPT_DIR}/../../frontend"
REGION="${AWS_REGION:-ap-southeast-1}"

echo "=== Coffee QM CDK Deployment (No Local Docker) ==="

# Step 1: Install CDK dependencies
echo ">>> Installing CDK dependencies..."
cd "${CDK_DIR}"
npm install --silent

# Step 2: Bootstrap CDK (if needed)
echo ">>> Bootstrapping CDK..."
npx cdk bootstrap --quiet 2>/dev/null || true

# Step 3: Deploy CDK stack
echo ">>> Deploying CDK stack..."
npx cdk deploy CoffeeQM-AI --require-approval never --outputs-file /tmp/cdk-outputs.json

# Step 4: Get outputs
AI_API_URL=$(cat /tmp/cdk-outputs.json | grep -o '"AIApiURL": "[^"]*"' | cut -d'"' -f4)
FRONTEND_BUCKET=$(cat /tmp/cdk-outputs.json | grep -o '"FrontendBucketName": "[^"]*"' | cut -d'"' -f4)
FRONTEND_URL=$(cat /tmp/cdk-outputs.json | grep -o '"FrontendURL": "[^"]*"' | cut -d'"' -f4)
CODEBUILD_PROJECT=$(cat /tmp/cdk-outputs.json | grep -o '"CodeBuildProject": "[^"]*"' | cut -d'"' -f4)

# Step 5: Trigger CodeBuild to build Docker image in AWS
echo ">>> Building Docker image in AWS CodeBuild..."
BUILD_ID=$(aws codebuild start-build --project-name ${CODEBUILD_PROJECT} --region ${REGION} --query 'build.id' --output text)
echo "    Build ID: ${BUILD_ID}"
echo "    Waiting for build to complete (this takes ~10-15 minutes for first build)..."

# Wait for build
while true; do
  STATUS=$(aws codebuild batch-get-builds --ids ${BUILD_ID} --region ${REGION} --query 'builds[0].buildStatus' --output text)
  if [ "$STATUS" = "SUCCEEDED" ]; then
    echo "    ✅ Build succeeded!"
    break
  elif [ "$STATUS" = "FAILED" ] || [ "$STATUS" = "FAULT" ] || [ "$STATUS" = "STOPPED" ]; then
    echo "    ❌ Build failed with status: ${STATUS}"
    echo "    Check logs: aws codebuild batch-get-builds --ids ${BUILD_ID}"
    exit 1
  fi
  echo "    Status: ${STATUS}... waiting"
  sleep 30
done

# Step 6: Update Lambda to use new image
echo ">>> Updating Lambda function..."
ECR_REPO=$(cat /tmp/cdk-outputs.json | grep -o '"ECRRepository": "[^"]*"' | cut -d'"' -f4)
aws lambda update-function-code \
  --function-name coffee-qm-ai-detect \
  --image-uri ${ECR_REPO}:latest \
  --region ${REGION} > /dev/null

# Wait for Lambda update
aws lambda wait function-updated --function-name coffee-qm-ai-detect --region ${REGION}
echo "    ✅ Lambda updated!"

# Step 7: Build and deploy frontend
echo ">>> Building frontend..."
cd "${FRONTEND_DIR}"
echo "VITE_AI_API_URL=${AI_API_URL}" > .env.production
npm install --silent
npm run build

echo ">>> Uploading frontend to S3..."
aws s3 sync dist/ s3://${FRONTEND_BUCKET}/ --delete

echo ""
echo "=========================================="
echo "=== Deployment Complete ==="
echo "=========================================="
echo ""
echo "Frontend URL: ${FRONTEND_URL}"
echo "AI API URL: ${AI_API_URL}"
echo ""
echo "🤖 Using REAL HuggingFace model!"
echo "   Model: everycoffee/autotrain-coffee-bean-quality-97496146930"
echo ""
echo "📱 Test on mobile: Open ${FRONTEND_URL}"

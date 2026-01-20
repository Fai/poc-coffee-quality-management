#!/bin/bash
# Deploy frontend to S3 and invalidate CloudFront
set -e

PROJECT_NAME="CoffeeQualityManagement"
ENVIRONMENT="${1:-test}"
AWS_REGION="${AWS_REGION:-ap-southeast-1}"
STACK_NAME="${PROJECT_NAME}-${ENVIRONMENT}"

echo "Deploying frontend for ${ENVIRONMENT}..."

# Get S3 bucket name
BUCKET_NAME=$(aws cloudformation describe-stacks \
    --stack-name "$STACK_NAME" \
    --query "Stacks[0].Outputs[?OutputKey=='FrontendBucketName'].OutputValue" \
    --output text \
    --region "$AWS_REGION")

# Get API endpoint for frontend config
ALB_DNS=$(aws cloudformation describe-stacks \
    --stack-name "$STACK_NAME" \
    --query "Stacks[0].Outputs[?OutputKey=='ALBDnsName'].OutputValue" \
    --output text \
    --region "$AWS_REGION")

# Build frontend
echo "Building frontend..."
cd frontend

# Create environment file
cat > .env.production << EOF
VITE_API_URL=http://${ALB_DNS}/api
VITE_ENVIRONMENT=${ENVIRONMENT}
EOF

npm ci
npm run build

# Upload to S3
echo "Uploading to S3..."
aws s3 sync dist/ "s3://${BUCKET_NAME}/" \
    --delete \
    --cache-control "max-age=31536000" \
    --region "$AWS_REGION"

# Set correct cache headers for index.html
aws s3 cp "s3://${BUCKET_NAME}/index.html" "s3://${BUCKET_NAME}/index.html" \
    --cache-control "no-cache, no-store, must-revalidate" \
    --content-type "text/html" \
    --metadata-directive REPLACE \
    --region "$AWS_REGION"

echo "=========================================="
echo "Frontend deployment complete!"
echo "S3 Bucket: ${BUCKET_NAME}"
echo "Note: Set up CloudFront distribution for production"
echo "=========================================="

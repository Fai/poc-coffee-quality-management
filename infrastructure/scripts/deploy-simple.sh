#!/bin/bash
set -e

# Configuration
PROJECT_NAME="coffee-qm"
REGION="${AWS_REGION:-ap-southeast-1}"
ACCOUNT_ID=$(aws sts get-caller-identity --query Account --output text)
STACK_NAME="${PROJECT_NAME}-app"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

echo "=== Coffee QM Deployment (No Docker) ==="
echo "Region: $REGION"
echo "Account: $ACCOUNT_ID"

# Step 1: Create Lambda zip with mock response
echo ">>> Creating Lambda package..."
LAMBDA_DIR=$(mktemp -d)
cat > ${LAMBDA_DIR}/lambda_handler.py << 'PYEOF'
import json
import random

def handler(event, context):
    # Mock AI response for testing
    is_defective = random.random() > 0.5
    return {
        'statusCode': 200,
        'headers': {
            'Content-Type': 'application/json',
            'Access-Control-Allow-Origin': '*',
            'Access-Control-Allow-Methods': 'POST, OPTIONS',
            'Access-Control-Allow-Headers': '*'
        },
        'body': json.dumps({
            'request_id': f'det-mock-{random.randint(10000,99999)}',
            'detection': {
                'request_id': f'det-mock-{random.randint(10000,99999)}',
                'image_url': 's3://mock/image.jpg',
                'is_defective': is_defective,
                'defect_probability': round(random.uniform(0.6, 0.95) if is_defective else random.uniform(0.05, 0.4), 2),
                'confidence_score': round(random.uniform(0.75, 0.98), 2),
                'processing_time_ms': random.randint(100, 500),
                'model': 'mock-model-for-testing',
                'note': 'MOCK RESPONSE - Deploy with Docker for real AI detection'
            },
            'suggested_grade': 'needs_inspection' if is_defective else 'likely_specialty'
        })
    }
PYEOF

cd ${LAMBDA_DIR}
zip -r lambda.zip lambda_handler.py
LAMBDA_ZIP="${LAMBDA_DIR}/lambda.zip"

# Step 2: Build frontend
echo ">>> Building frontend..."
cd "${SCRIPT_DIR}/../../frontend"
npm ci --silent
npm run build

# Step 3: Deploy CloudFormation stack
echo ">>> Deploying CloudFormation stack..."

cat > /tmp/coffee-qm-simple-stack.yaml << 'EOF'
AWSTemplateFormatVersion: '2010-09-09'
Description: Coffee QM - Frontend + Mock AI Lambda (No Docker)

Parameters:
  ProjectName:
    Type: String
    Default: coffee-qm

Resources:
  # S3 Bucket for Frontend
  FrontendBucket:
    Type: AWS::S3::Bucket
    Properties:
      BucketName: !Sub '${ProjectName}-frontend-${AWS::AccountId}'
      PublicAccessBlockConfiguration:
        BlockPublicAcls: false
        BlockPublicPolicy: false
        IgnorePublicAcls: false
        RestrictPublicBuckets: false
      WebsiteConfiguration:
        IndexDocument: index.html
        ErrorDocument: index.html

  FrontendBucketPolicy:
    Type: AWS::S3::BucketPolicy
    Properties:
      Bucket: !Ref FrontendBucket
      PolicyDocument:
        Statement:
          - Effect: Allow
            Principal: '*'
            Action: s3:GetObject
            Resource: !Sub '${FrontendBucket.Arn}/*'

  # Lambda Function (Mock)
  AILambdaRole:
    Type: AWS::IAM::Role
    Properties:
      AssumeRolePolicyDocument:
        Version: '2012-10-17'
        Statement:
          - Effect: Allow
            Principal:
              Service: lambda.amazonaws.com
            Action: sts:AssumeRole
      ManagedPolicyArns:
        - arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole

  AILambda:
    Type: AWS::Lambda::Function
    Properties:
      FunctionName: !Sub '${ProjectName}-ai-detect'
      Runtime: python3.11
      Handler: lambda_handler.handler
      Role: !GetAtt AILambdaRole.Arn
      Timeout: 30
      MemorySize: 128
      Code:
        ZipFile: |
          import json
          import random
          def handler(event, context):
              is_defective = random.random() > 0.5
              return {
                  'statusCode': 200,
                  'headers': {'Content-Type': 'application/json', 'Access-Control-Allow-Origin': '*'},
                  'body': json.dumps({
                      'request_id': f'det-mock-{random.randint(10000,99999)}',
                      'detection': {
                          'is_defective': is_defective,
                          'defect_probability': round(random.uniform(0.6, 0.95) if is_defective else random.uniform(0.05, 0.4), 2),
                          'confidence_score': round(random.uniform(0.75, 0.98), 2),
                          'processing_time_ms': random.randint(100, 500),
                          'model': 'mock-model-for-testing',
                          'note': 'MOCK RESPONSE - Deploy with Docker for real AI'
                      },
                      'suggested_grade': 'needs_inspection' if is_defective else 'likely_specialty'
                  })
              }

  # API Gateway
  AIApi:
    Type: AWS::ApiGatewayV2::Api
    Properties:
      Name: !Sub '${ProjectName}-ai-api'
      ProtocolType: HTTP
      CorsConfiguration:
        AllowOrigins: ['*']
        AllowMethods: [POST, OPTIONS]
        AllowHeaders: ['*']

  AIApiIntegration:
    Type: AWS::ApiGatewayV2::Integration
    Properties:
      ApiId: !Ref AIApi
      IntegrationType: AWS_PROXY
      IntegrationUri: !GetAtt AILambda.Arn
      PayloadFormatVersion: '2.0'

  AIApiRoute:
    Type: AWS::ApiGatewayV2::Route
    Properties:
      ApiId: !Ref AIApi
      RouteKey: 'POST /detect'
      Target: !Sub 'integrations/${AIApiIntegration}'

  AIApiStage:
    Type: AWS::ApiGatewayV2::Stage
    Properties:
      ApiId: !Ref AIApi
      StageName: '$default'
      AutoDeploy: true

  AILambdaPermission:
    Type: AWS::Lambda::Permission
    Properties:
      FunctionName: !Ref AILambda
      Action: lambda:InvokeFunction
      Principal: apigateway.amazonaws.com
      SourceArn: !Sub 'arn:aws:execute-api:${AWS::Region}:${AWS::AccountId}:${AIApi}/*'

Outputs:
  FrontendUrl:
    Value: !GetAtt FrontendBucket.WebsiteURL
  AIApiUrl:
    Value: !Sub 'https://${AIApi}.execute-api.${AWS::Region}.amazonaws.com'
  FrontendBucket:
    Value: !Ref FrontendBucket
EOF

aws cloudformation deploy \
  --template-file /tmp/coffee-qm-simple-stack.yaml \
  --stack-name $STACK_NAME \
  --parameter-overrides ProjectName=$PROJECT_NAME \
  --capabilities CAPABILITY_IAM \
  --region $REGION

# Step 4: Get outputs
echo ">>> Getting stack outputs..."
FRONTEND_BUCKET=$(aws cloudformation describe-stacks --stack-name $STACK_NAME --region $REGION \
  --query 'Stacks[0].Outputs[?OutputKey==`FrontendBucket`].OutputValue' --output text)
FRONTEND_URL=$(aws cloudformation describe-stacks --stack-name $STACK_NAME --region $REGION \
  --query 'Stacks[0].Outputs[?OutputKey==`FrontendUrl`].OutputValue' --output text)
AI_API_URL=$(aws cloudformation describe-stacks --stack-name $STACK_NAME --region $REGION \
  --query 'Stacks[0].Outputs[?OutputKey==`AIApiUrl`].OutputValue' --output text)

# Step 5: Rebuild frontend with correct API URL and upload
echo ">>> Rebuilding frontend with API URL..."
cd "${SCRIPT_DIR}/../../frontend"
echo "VITE_AI_API_URL=${AI_API_URL}" > .env.production
npm run build

echo ">>> Uploading frontend to S3..."
aws s3 sync dist/ s3://${FRONTEND_BUCKET}/ --delete

# Cleanup
rm -rf ${LAMBDA_DIR}

echo ""
echo "=========================================="
echo "=== Deployment Complete ==="
echo "=========================================="
echo ""
echo "Frontend URL: $FRONTEND_URL"
echo "AI API URL: $AI_API_URL"
echo ""
echo "⚠️  NOTE: Using MOCK AI responses for testing"
echo "    To enable real AI, fix Docker and run deploy-app.sh"
echo ""
echo "📱 Test on mobile:"
echo "   1. Open $FRONTEND_URL on your phone"
echo "   2. Login with any email/password"
echo "   3. Go to 'Defect Detection' menu"
echo "   4. Take a photo or upload an image"
echo ""

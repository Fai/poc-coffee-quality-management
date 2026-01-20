#!/bin/bash
set -e

# Configuration
PROJECT_NAME="coffee-qm"
REGION="${AWS_REGION:-ap-southeast-1}"
ACCOUNT_ID=$(aws sts get-caller-identity --query Account --output text)
ECR_REPO="${ACCOUNT_ID}.dkr.ecr.${REGION}.amazonaws.com/${PROJECT_NAME}-ai"
STACK_NAME="${PROJECT_NAME}-app"

echo "=== Coffee QM Deployment ==="
echo "Region: $REGION"
echo "Account: $ACCOUNT_ID"

# Step 1: Create ECR repository if not exists
echo ">>> Creating ECR repository..."
aws ecr describe-repositories --repository-names ${PROJECT_NAME}-ai --region $REGION 2>/dev/null || \
  aws ecr create-repository --repository-name ${PROJECT_NAME}-ai --region $REGION

# Step 2: Build and push Lambda container
echo ">>> Building Lambda container..."
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "${SCRIPT_DIR}/../../ai-defect-detection/lambda"
aws ecr get-login-password --region $REGION | docker login --username AWS --password-stdin ${ACCOUNT_ID}.dkr.ecr.${REGION}.amazonaws.com

docker build --platform linux/amd64 -t ${PROJECT_NAME}-ai:latest .
docker tag ${PROJECT_NAME}-ai:latest ${ECR_REPO}:latest
docker push ${ECR_REPO}:latest

# Step 3: Build frontend
echo ">>> Building frontend..."
cd "${SCRIPT_DIR}/../../frontend"
npm ci
npm run build

# Step 4: Deploy CloudFormation stack
echo ">>> Deploying CloudFormation stack..."
cd "${SCRIPT_DIR}"

cat > /tmp/coffee-qm-stack.yaml << 'EOF'
AWSTemplateFormatVersion: '2010-09-09'
Description: Coffee QM - Frontend + AI Lambda

Parameters:
  ProjectName:
    Type: String
    Default: coffee-qm
  LambdaImageUri:
    Type: String

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

  # S3 Bucket for Images
  ImagesBucket:
    Type: AWS::S3::Bucket
    Properties:
      BucketName: !Sub '${ProjectName}-images-${AWS::AccountId}'
      CorsConfiguration:
        CorsRules:
          - AllowedHeaders: ['*']
            AllowedMethods: [GET, PUT, POST]
            AllowedOrigins: ['*']

  # Lambda Function
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
      Policies:
        - PolicyName: S3Access
          PolicyDocument:
            Version: '2012-10-17'
            Statement:
              - Effect: Allow
                Action: [s3:PutObject, s3:GetObject]
                Resource: !Sub '${ImagesBucket.Arn}/*'

  AILambda:
    Type: AWS::Lambda::Function
    Properties:
      FunctionName: !Sub '${ProjectName}-ai-detect'
      PackageType: Image
      Code:
        ImageUri: !Ref LambdaImageUri
      Role: !GetAtt AILambdaRole.Arn
      Timeout: 60
      MemorySize: 3008
      Environment:
        Variables:
          BUCKET_NAME: !Ref ImagesBucket

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
  --template-file /tmp/coffee-qm-stack.yaml \
  --stack-name $STACK_NAME \
  --parameter-overrides \
    ProjectName=$PROJECT_NAME \
    LambdaImageUri=${ECR_REPO}:latest \
  --capabilities CAPABILITY_IAM \
  --region $REGION

# Step 5: Get outputs
echo ">>> Getting stack outputs..."
FRONTEND_BUCKET=$(aws cloudformation describe-stacks --stack-name $STACK_NAME --region $REGION \
  --query 'Stacks[0].Outputs[?OutputKey==`FrontendBucket`].OutputValue' --output text)
FRONTEND_URL=$(aws cloudformation describe-stacks --stack-name $STACK_NAME --region $REGION \
  --query 'Stacks[0].Outputs[?OutputKey==`FrontendUrl`].OutputValue' --output text)
AI_API_URL=$(aws cloudformation describe-stacks --stack-name $STACK_NAME --region $REGION \
  --query 'Stacks[0].Outputs[?OutputKey==`AIApiUrl`].OutputValue' --output text)

# Step 6: Upload frontend with correct API URL
echo ">>> Uploading frontend..."
cd "${SCRIPT_DIR}/../../frontend"

# Create env file for production
echo "VITE_AI_API_URL=${AI_API_URL}" > .env.production
npm run build

aws s3 sync dist/ s3://${FRONTEND_BUCKET}/ --delete

echo ""
echo "=== Deployment Complete ==="
echo "Frontend URL: $FRONTEND_URL"
echo "AI API URL: $AI_API_URL"
echo ""
echo "Test on mobile: Open $FRONTEND_URL in your phone browser"
echo "Add to home screen for PWA experience"

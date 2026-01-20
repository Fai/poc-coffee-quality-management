#!/bin/bash
set -e

REGION="${AWS_REGION:-ap-southeast-1}"
PROJECT_NAME="coffee-qm-ai-build"
ECR_REPO="coffee-qm-ai"
LAMBDA_NAME="coffee-qm-ai-detect"
ACCOUNT_ID=$(aws sts get-caller-identity --query Account --output text)
ECR_URI="${ACCOUNT_ID}.dkr.ecr.${REGION}.amazonaws.com/${ECR_REPO}"

echo "=== Deploy Real AI Model with CodeBuild ==="

# Step 1: Ensure ECR repository exists
aws ecr describe-repositories --repository-names ${ECR_REPO} --region ${REGION} >/dev/null 2>&1 || \
  aws ecr create-repository --repository-name ${ECR_REPO} --region ${REGION} >/dev/null

# Step 2: Create IAM role
ROLE_NAME="codebuild-${PROJECT_NAME}-role"
aws iam get-role --role-name ${ROLE_NAME} >/dev/null 2>&1 || \
aws iam create-role --role-name ${ROLE_NAME} --assume-role-policy-document '{
  "Version": "2012-10-17",
  "Statement": [{"Effect": "Allow", "Principal": {"Service": "codebuild.amazonaws.com"}, "Action": "sts:AssumeRole"}]
}' >/dev/null

for policy in AmazonEC2ContainerRegistryPowerUser CloudWatchLogsFullAccess AWSLambda_FullAccess; do
  aws iam attach-role-policy --role-name ${ROLE_NAME} --policy-arn arn:aws:iam::aws:policy/${policy} 2>/dev/null || true
done
sleep 10

# Step 3: Create buildspec file
cat > /tmp/buildspec.yml << 'BUILDSPECEOF'
version: 0.2
phases:
  pre_build:
    commands:
      - echo Logging into ECR...
      - aws ecr get-login-password --region $AWS_DEFAULT_REGION | docker login --username AWS --password-stdin $ECR_URI
  build:
    commands:
      - echo Creating Dockerfile...
      - |
        cat > Dockerfile << 'DEOF'
        FROM python:3.11-slim as builder
        RUN pip install --no-cache-dir boto3 transformers torch --extra-index-url https://download.pytorch.org/whl/cpu
        RUN pip install --no-cache-dir Pillow
        RUN python -c "from transformers import pipeline; pipeline('image-classification', model='everycoffee/autotrain-coffee-bean-quality-97496146930')"
        
        FROM public.ecr.aws/lambda/python:3.11
        COPY --from=builder /usr/local/lib/python3.11/site-packages /var/lang/lib/python3.11/site-packages
        COPY --from=builder /root/.cache/huggingface /root/.cache/huggingface
        COPY handler.py ${LAMBDA_TASK_ROOT}/lambda_handler.py
        CMD ["lambda_handler.handler"]
        DEOF
      - echo Creating handler...
      - |
        cat > handler.py << 'HEOF'
        import json, boto3, base64, os, uuid
        from datetime import datetime
        from io import BytesIO
        s3 = boto3.client("s3")
        BUCKET = os.environ.get("BUCKET_NAME", "coffee-qm-images")
        MODEL = "everycoffee/autotrain-coffee-bean-quality-97496146930"
        _clf = None
        def get_clf():
            global _clf
            if _clf is None:
                from transformers import pipeline
                _clf = pipeline("image-classification", model=MODEL)
            return _clf
        def handler(event, context):
            try:
                body = json.loads(event.get("body", "{}"))
                img_b64 = body.get("image_base64")
                if not img_b64:
                    return {"statusCode": 400, "headers": {"Content-Type": "application/json", "Access-Control-Allow-Origin": "*"}, "body": json.dumps({"error": "Missing image_base64"})}
                rid = "det-" + datetime.utcnow().strftime("%Y%m%d%H%M%S") + "-" + str(uuid.uuid4())[:8]
                t0 = datetime.utcnow()
                img_bytes = base64.b64decode(img_b64)
                key = "uploads/" + rid + ".jpg"
                s3.put_object(Bucket=BUCKET, Key=key, Body=img_bytes, ContentType="image/jpeg")
                from PIL import Image
                img = Image.open(BytesIO(img_bytes)).convert("RGB")
                res = get_clf()(img)
                d_score = next((r["score"] for r in res if "defect" in r["label"].lower()), 0.0)
                is_def = d_score > 0.5
                ms = int((datetime.utcnow() - t0).total_seconds() * 1000)
                return {"statusCode": 200, "headers": {"Content-Type": "application/json", "Access-Control-Allow-Origin": "*"}, "body": json.dumps({"request_id": rid, "detection": {"is_defective": is_def, "defect_probability": round(d_score, 3), "confidence_score": round(max(d_score, 1-d_score), 3), "processing_time_ms": ms, "model": MODEL, "note": "Binary classification"}, "suggested_grade": "needs_inspection" if is_def else "likely_specialty"})}
            except Exception as e:
                return {"statusCode": 500, "headers": {"Content-Type": "application/json", "Access-Control-Allow-Origin": "*"}, "body": json.dumps({"error": str(e)})}
        HEOF
      - echo Building Docker image...
      - docker build -t $ECR_URI:latest .
  post_build:
    commands:
      - echo Pushing to ECR...
      - docker push $ECR_URI:latest
      - echo Build complete!
BUILDSPECEOF

# Upload buildspec to S3
BUILDSPEC_BUCKET="coffee-qm-frontend-${ACCOUNT_ID}"
aws s3 cp /tmp/buildspec.yml s3://${BUILDSPEC_BUCKET}/buildspec.yml

# Step 4: Create CodeBuild project using JSON
cat > /tmp/project.json << EOF
{
  "name": "${PROJECT_NAME}",
  "source": {
    "type": "S3",
    "location": "${BUILDSPEC_BUCKET}/buildspec.yml",
    "buildspec": "buildspec.yml"
  },
  "artifacts": {"type": "NO_ARTIFACTS"},
  "environment": {
    "type": "LINUX_CONTAINER",
    "computeType": "BUILD_GENERAL1_MEDIUM",
    "image": "aws/codebuild/standard:7.0",
    "privilegedMode": true,
    "environmentVariables": [
      {"name": "ECR_URI", "value": "${ECR_URI}"},
      {"name": "LAMBDA_NAME", "value": "${LAMBDA_NAME}"}
    ]
  },
  "serviceRole": "arn:aws:iam::${ACCOUNT_ID}:role/${ROLE_NAME}",
  "timeoutInMinutes": 30
}
EOF

aws codebuild delete-project --name ${PROJECT_NAME} --region ${REGION} 2>/dev/null || true
sleep 2
aws codebuild create-project --cli-input-json file:///tmp/project.json --region ${REGION} >/dev/null

echo ">>> Starting build (10-15 min)..."
BUILD_ID=$(aws codebuild start-build --project-name ${PROJECT_NAME} --region ${REGION} --query 'build.id' --output text)
echo "    Build: ${BUILD_ID}"
echo "    Monitor: https://${REGION}.console.aws.amazon.com/codesuite/codebuild/projects/${PROJECT_NAME}"

while true; do
  STATUS=$(aws codebuild batch-get-builds --ids ${BUILD_ID} --region ${REGION} --query 'builds[0].buildStatus' --output text)
  PHASE=$(aws codebuild batch-get-builds --ids ${BUILD_ID} --region ${REGION} --query 'builds[0].currentPhase' --output text)
  echo "    ${STATUS} - ${PHASE}"
  [ "$STATUS" = "SUCCEEDED" ] && break
  [ "$STATUS" = "FAILED" ] || [ "$STATUS" = "FAULT" ] && { echo "❌ Failed"; exit 1; }
  sleep 30
done

echo ">>> Updating Lambda..."
aws lambda update-function-code --function-name ${LAMBDA_NAME} --image-uri ${ECR_URI}:latest --region ${REGION} >/dev/null
aws lambda wait function-updated --function-name ${LAMBDA_NAME} --region ${REGION}

echo ""
echo "✅ Done! Test at: http://coffee-qm-frontend-${ACCOUNT_ID}.s3-website-${REGION}.amazonaws.com"

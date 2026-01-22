#!/bin/bash
set -e

REGION="${AWS_REGION:-ap-southeast-1}"
LAMBDA_NAME="coffee-qm-yolo-detect"
ACCOUNT_ID=$(aws sts get-caller-identity --query Account --output text)

echo "=== Deploy YOLOv8 Object Detection Lambda (Roboflow API) ==="
echo "    (Keeping coffee-qm-ai-detect as fallback)"

# Create Lambda code
mkdir -p /tmp/yolo-lambda
cat > /tmp/yolo-lambda/lambda_handler.py << 'PYEOF'
import json, boto3, base64, os, uuid, urllib.request
from datetime import datetime

ROBOFLOW_API_KEY = os.environ.get("ROBOFLOW_API_KEY", "")
MODEL_ENDPOINT = "coffee-bean-defect-smvw1/3"
CLASSES = ["black", "broken", "foreign", "fraghusk", "green", "husk", "immature", "infested", "sour"]
s3 = boto3.client("s3")
BUCKET = os.environ.get("BUCKET_NAME", "coffee-qm-images")

def handler(event, context):
    try:
        body = json.loads(event.get("body", "{}"))
        img_b64 = body.get("image_base64")
        if not img_b64:
            return resp(400, {"error": "Missing image_base64"})
        if not ROBOFLOW_API_KEY:
            return resp(500, {"error": "ROBOFLOW_API_KEY not configured"})
        
        rid = f"yolo-{datetime.utcnow().strftime('%Y%m%d%H%M%S')}-{uuid.uuid4().hex[:8]}"
        t0 = datetime.utcnow()
        
        # Save to S3
        img_bytes = base64.b64decode(img_b64)
        s3.put_object(Bucket=BUCKET, Key=f"uploads/{rid}.jpg", Body=img_bytes, ContentType="image/jpeg")
        
        # Call Roboflow API
        url = f"https://detect.roboflow.com/{MODEL_ENDPOINT}?api_key={ROBOFLOW_API_KEY}"
        req = urllib.request.Request(url, data=img_b64.encode(), headers={"Content-Type": "application/x-www-form-urlencoded"})
        with urllib.request.urlopen(req, timeout=25) as r:
            result = json.loads(r.read().decode())
        
        # Parse predictions
        defects = {c: 0 for c in CLASSES}
        detections = []
        for pred in result.get("predictions", []):
            cls = pred.get("class", "unknown")
            conf = pred.get("confidence", 0)
            defects[cls] = defects.get(cls, 0) + 1
            detections.append({
                "class": cls,
                "confidence": round(conf, 3),
                "bbox": [pred.get("x", 0), pred.get("y", 0), pred.get("width", 0), pred.get("height", 0)]
            })
        
        total = sum(defects.values())
        ms = int((datetime.utcnow() - t0).total_seconds() * 1000)
        
        # SCA-style grading based on defect counts
        grade = "specialty" if total == 0 else "premium" if total <= 5 else "commercial" if total <= 15 else "below_grade"
        
        return resp(200, {
            "request_id": rid,
            "detection": {
                "total_defects": total,
                "defect_counts": {k: v for k, v in defects.items() if v > 0},
                "detections": detections[:100],
                "processing_time_ms": ms,
                "model": "roboflow/coffee-bean-defect-smvw1/3"
            },
            "suggested_grade": grade
        })
    except Exception as e:
        import traceback
        return resp(500, {"error": str(e), "trace": traceback.format_exc()})

def resp(code, body):
    return {"statusCode": code, "headers": {"Content-Type": "application/json", "Access-Control-Allow-Origin": "*"}, "body": json.dumps(body)}
PYEOF

cd /tmp/yolo-lambda && zip -q lambda.zip lambda_handler.py

# Create/update Lambda
LAMBDA_ROLE="arn:aws:iam::${ACCOUNT_ID}:role/coffee-qm-lambda-role"

aws iam get-role --role-name coffee-qm-lambda-role >/dev/null 2>&1 || {
  echo "Creating Lambda role..."
  aws iam create-role --role-name coffee-qm-lambda-role --assume-role-policy-document '{
    "Version": "2012-10-17",
    "Statement": [{"Effect": "Allow", "Principal": {"Service": "lambda.amazonaws.com"}, "Action": "sts:AssumeRole"}]
  }' >/dev/null
  aws iam attach-role-policy --role-name coffee-qm-lambda-role --policy-arn arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole
  aws iam attach-role-policy --role-name coffee-qm-lambda-role --policy-arn arn:aws:iam::aws:policy/AmazonS3FullAccess
  sleep 10
}

echo ">>> Deploying Lambda: ${LAMBDA_NAME}..."
aws lambda get-function --function-name ${LAMBDA_NAME} --region ${REGION} >/dev/null 2>&1 && {
  aws lambda update-function-code --function-name ${LAMBDA_NAME} --zip-file fileb:///tmp/yolo-lambda/lambda.zip --region ${REGION} >/dev/null
  aws lambda wait function-updated --function-name ${LAMBDA_NAME} --region ${REGION} 2>/dev/null || sleep 5
  aws lambda update-function-configuration --function-name ${LAMBDA_NAME} --timeout 30 --memory-size 256 --region ${REGION} >/dev/null
} || {
  aws lambda create-function \
    --function-name ${LAMBDA_NAME} \
    --runtime python3.11 \
    --handler lambda_handler.handler \
    --zip-file fileb:///tmp/yolo-lambda/lambda.zip \
    --role ${LAMBDA_ROLE} \
    --timeout 30 \
    --memory-size 256 \
    --environment '{"Variables":{"BUCKET_NAME":"coffee-qm-images","ROBOFLOW_API_KEY":"PLACEHOLDER"}}' \
    --region ${REGION} >/dev/null
}

# Add API Gateway route
API_ID="f6m336nzv4"
LAMBDA_ARN=$(aws lambda get-function --function-name ${LAMBDA_NAME} --region ${REGION} --query 'Configuration.FunctionArn' --output text)

echo ">>> Adding /yolo route to API Gateway..."
aws lambda add-permission \
  --function-name ${LAMBDA_NAME} \
  --statement-id apigateway-yolo-$(date +%s) \
  --action lambda:InvokeFunction \
  --principal apigateway.amazonaws.com \
  --source-arn "arn:aws:execute-api:${REGION}:${ACCOUNT_ID}:${API_ID}/*/*/yolo" \
  --region ${REGION} 2>/dev/null || true

# Check if route exists
ROUTE_EXISTS=$(aws apigatewayv2 get-routes --api-id ${API_ID} --region ${REGION} --query "Items[?RouteKey=='POST /yolo'].RouteId" --output text)

if [ -z "$ROUTE_EXISTS" ]; then
  INTEGRATION_ID=$(aws apigatewayv2 create-integration \
    --api-id ${API_ID} \
    --integration-type AWS_PROXY \
    --integration-uri ${LAMBDA_ARN} \
    --payload-format-version 2.0 \
    --timeout-in-millis 29000 \
    --region ${REGION} \
    --query 'IntegrationId' --output text)
  
  aws apigatewayv2 create-route \
    --api-id ${API_ID} \
    --route-key "POST /yolo" \
    --target "integrations/${INTEGRATION_ID}" \
    --region ${REGION} >/dev/null
  echo "    Created POST /yolo route"
else
  echo "    Route POST /yolo already exists"
fi

echo ""
echo "=== Deployment Complete ==="
echo ""
echo "YOLO endpoint: https://${API_ID}.execute-api.${REGION}.amazonaws.com/yolo"
echo "Fallback:      https://${API_ID}.execute-api.${REGION}.amazonaws.com/detect"
echo ""
echo "⚠️  Set Roboflow API key:"
echo "   aws lambda update-function-configuration --function-name ${LAMBDA_NAME} \\"
echo "     --environment 'Variables={BUCKET_NAME=coffee-qm-images,ROBOFLOW_API_KEY=YOUR_KEY}' \\"
echo "     --region ${REGION}"
echo ""
echo "Get free API key at: https://app.roboflow.com (1,000 free inferences/month)"

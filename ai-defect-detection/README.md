# AI Defect Detection Microservice

AWS-hosted microservice for automated coffee bean defect detection using computer vision.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                  AI Defect Detection Architecture                │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Mobile App / Backend                                            │
│      │                                                           │
│      ▼                                                           │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────────────┐  │
│  │ API Gateway │───▶│   Lambda    │───▶│  SageMaker Endpoint │  │
│  │  (REST)     │    │  (Handler)  │    │  (Vision Model)     │  │
│  └─────────────┘    └─────────────┘    └─────────────────────┘  │
│                            │                     │               │
│                            ▼                     │               │
│                     ┌─────────────┐              │               │
│                     │     S3      │◀─────────────┘               │
│                     │  (Images)   │                              │
│                     └─────────────┘                              │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## Components

### CloudFormation Templates

- `cloudformation/ai-defect-detection.yaml` - Main infrastructure (API Gateway, Lambda, S3)
- `cloudformation/sagemaker-endpoint.yaml` - SageMaker model endpoint

### Lambda Function

- `lambda/lambda_handler.py` - Main request handler
- `lambda/annotate_image.py` - Image annotation with bounding boxes
- `lambda/requirements.txt` - Python dependencies

### Model

- `model/inference.py` - SageMaker inference script

## API

### POST /detect

Detect defects in a coffee bean sample image.

**Request:**
```json
{
  "image_base64": "base64_encoded_image_data",
  "sample_weight_grams": 350.0
}
```

**Response:**
```json
{
  "request_id": "det-20241223120000-abc12345",
  "detection": {
    "request_id": "det-20241223120000-abc12345",
    "image_url": "s3://coffee-qm-images/uploads/det-20241223120000-abc12345.jpg",
    "detected_beans": 350,
    "defect_breakdown": {
      "full_black": 1,
      "full_sour": 0,
      "partial_black": 2,
      "broken": 3,
      ...
    },
    "category1_count": 1,
    "category2_count": 5,
    "confidence_score": 0.95,
    "processing_time_ms": 1500,
    "annotated_image_url": "s3://coffee-qm-images/annotated/det-20241223120000-abc12345_annotated.jpg"
  },
  "suggested_grade": "premium_grade"
}
```

## Defect Types

### Category 1 (Primary Defects)
- full_black - ดำเต็มเมล็ด
- full_sour - เปรี้ยวเต็มเมล็ด
- pod_cherry - เชอร์รี่ติดฝัก
- large_stones - หินใหญ่
- medium_stones - หินกลาง
- large_sticks - กิ่งไม้ใหญ่
- medium_sticks - กิ่งไม้กลาง

### Category 2 (Secondary Defects)
- partial_black - ดำบางส่วน
- partial_sour - เปรี้ยวบางส่วน
- parchment - กะลา
- floater - ลอย
- immature - ไม่สุก
- withered - เหี่ยว
- shell - เปลือก
- broken - แตก
- chipped - บิ่น
- cut - ตัด
- insect_damage - แมลงกัด
- husk - เปลือกแห้ง

## Grade Classification (SCA Standards)

| Grade | Total Defects | Category 1 |
|-------|---------------|------------|
| Specialty Grade | 0-5 | 0 |
| Premium Grade | 0-8 | Any |
| Exchange Grade | 9-23 | Any |
| Below Standard | 24-86 | Any |
| Off Grade | 87+ | Any |

## Deployment

### Prerequisites

- AWS CLI configured
- Python 3.11+
- Trained model artifacts in S3

### Deploy Infrastructure

```bash
# Deploy main infrastructure
aws cloudformation deploy \
  --template-file cloudformation/ai-defect-detection.yaml \
  --stack-name coffee-qm-ai-defect-detection-dev \
  --parameter-overrides Environment=dev \
  --capabilities CAPABILITY_NAMED_IAM

# Deploy SageMaker endpoint (requires trained model)
aws cloudformation deploy \
  --template-file cloudformation/sagemaker-endpoint.yaml \
  --stack-name coffee-qm-ai-sagemaker-dev \
  --parameter-overrides \
    Environment=dev \
    ModelDataUrl=s3://coffee-qm-models-dev/model.tar.gz \
  --capabilities CAPABILITY_NAMED_IAM
```

### Update Lambda Code

```bash
# Package Lambda function
cd lambda
zip -r ../lambda.zip .

# Update function
aws lambda update-function-code \
  --function-name coffee-qm-defect-detection-dev \
  --zip-file fileb://../lambda.zip
```

## Environment Variables

| Variable | Description |
|----------|-------------|
| ENVIRONMENT | Deployment environment (dev/staging/prod) |
| BUCKET_NAME | S3 bucket for images |
| SAGEMAKER_ENDPOINT | SageMaker endpoint name |

## Backend Integration

Configure the Rust backend to use the AI service:

```bash
export CQM__AI_DETECTION__API_ENDPOINT=https://xxx.execute-api.region.amazonaws.com/dev/detect
export CQM__AI_DETECTION__API_KEY=your-api-key
```

## Model Training

The defect detection model is based on object detection (YOLO/Faster R-CNN) fine-tuned on coffee bean images.

### Training Data Requirements

- High-resolution images of green bean samples (350g spread)
- Bounding box annotations for each defect
- Minimum 1000 images per defect type recommended
- Balanced dataset across all 18 defect types

### Model Output

- Bounding boxes with defect classifications
- Confidence scores per detection
- Support for batch processing

"""
Coffee Bean Defect Detection Lambda Handler

This Lambda function handles image uploads, invokes the SageMaker endpoint
for defect detection, and returns structured results.
"""

import json
import boto3
import base64
import os
import uuid
from datetime import datetime
from typing import Any, Dict, Optional
from decimal import Decimal

# Initialize AWS clients
s3 = boto3.client('s3')
sagemaker_runtime = boto3.client('sagemaker-runtime')

# Environment variables
BUCKET_NAME = os.environ.get('BUCKET_NAME', 'coffee-qm-images')
SAGEMAKER_ENDPOINT = os.environ.get('SAGEMAKER_ENDPOINT', 'coffee-defect-detection')
ENVIRONMENT = os.environ.get('ENVIRONMENT', 'dev')

# Defect type mappings
CATEGORY1_DEFECTS = [
    'full_black', 'full_sour', 'pod_cherry', 
    'large_stones', 'medium_stones', 'large_sticks', 'medium_sticks'
]

CATEGORY2_DEFECTS = [
    'partial_black', 'partial_sour', 'parchment', 'floater',
    'immature', 'withered', 'shell', 'broken', 'chipped',
    'cut', 'insect_damage', 'husk'
]


def handler(event: Dict[str, Any], context: Any) -> Dict[str, Any]:
    """
    Main Lambda handler for defect detection requests.
    
    Expected request body:
    {
        "image_base64": "base64_encoded_image_data",
        "sample_weight_grams": 350.0  // optional
    }
    
    Returns:
    {
        "request_id": "det-20241223-abc123",
        "detection": {
            "request_id": "...",
            "image_url": "s3://...",
            "detected_beans": 350,
            "defect_breakdown": {...},
            "category1_count": 2,
            "category2_count": 5,
            "confidence_score": 0.95,
            "processing_time_ms": 1500,
            "annotated_image_url": "s3://..."
        },
        "suggested_grade": "premium_grade"
    }
    """
    try:
        # Parse request body
        body = json.loads(event.get('body', '{}'))
        image_base64 = body.get('image_base64')
        sample_weight_grams = body.get('sample_weight_grams', 350.0)
        
        if not image_base64:
            return error_response(400, 'Missing required field: image_base64')
        
        # Generate request ID
        request_id = generate_request_id(context)
        start_time = datetime.utcnow()
        
        # Decode and store image
        try:
            image_bytes = base64.b64decode(image_base64)
        except Exception as e:
            return error_response(400, f'Invalid base64 image data: {str(e)}')
        
        # Upload original image to S3
        image_key = f'uploads/{request_id}.jpg'
        s3.put_object(
            Bucket=BUCKET_NAME,
            Key=image_key,
            Body=image_bytes,
            ContentType='image/jpeg',
            Metadata={
                'request_id': request_id,
                'sample_weight_grams': str(sample_weight_grams)
            }
        )
        
        # Invoke SageMaker endpoint for detection
        detection_result = invoke_sagemaker(image_bytes, request_id)
        
        # Calculate processing time
        processing_time_ms = int((datetime.utcnow() - start_time).total_seconds() * 1000)
        
        # Format detection result
        detection = format_detection_result(
            detection_result,
            request_id,
            image_key,
            processing_time_ms
        )
        
        # Calculate suggested grade
        suggested_grade = calculate_grade(
            detection['category1_count'],
            detection['category2_count']
        )
        
        return success_response({
            'request_id': request_id,
            'detection': detection,
            'suggested_grade': suggested_grade
        })
        
    except Exception as e:
        print(f'Error processing request: {str(e)}')
        return error_response(500, f'Internal server error: {str(e)}')


def generate_request_id(context: Any) -> str:
    """Generate a unique request ID."""
    timestamp = datetime.utcnow().strftime('%Y%m%d%H%M%S')
    short_uuid = str(uuid.uuid4())[:8]
    return f'det-{timestamp}-{short_uuid}'


def invoke_sagemaker(image_bytes: bytes, request_id: str) -> Dict[str, Any]:
    """
    Invoke SageMaker endpoint for defect detection.
    
    In production, this calls the actual SageMaker endpoint.
    For development/testing, returns mock data.
    """
    if ENVIRONMENT == 'dev' and not endpoint_exists():
        # Return mock data for development
        return generate_mock_detection()
    
    try:
        response = sagemaker_runtime.invoke_endpoint(
            EndpointName=SAGEMAKER_ENDPOINT,
            ContentType='application/x-image',
            Body=image_bytes
        )
        
        result = json.loads(response['Body'].read().decode())
        return result
        
    except Exception as e:
        print(f'SageMaker invocation error: {str(e)}')
        # Fall back to mock data if endpoint fails
        if ENVIRONMENT == 'dev':
            return generate_mock_detection()
        raise


def endpoint_exists() -> bool:
    """Check if SageMaker endpoint exists."""
    try:
        sagemaker = boto3.client('sagemaker')
        sagemaker.describe_endpoint(EndpointName=SAGEMAKER_ENDPOINT)
        return True
    except:
        return False


def generate_mock_detection() -> Dict[str, Any]:
    """Generate mock detection results for development/testing."""
    import random
    
    # Simulate realistic defect distribution
    defects = {
        # Category 1 (Primary) - typically fewer
        'full_black': random.randint(0, 2),
        'full_sour': random.randint(0, 1),
        'pod_cherry': random.randint(0, 1),
        'large_stones': 0,
        'medium_stones': 0,
        'large_sticks': 0,
        'medium_sticks': 0,
        # Category 2 (Secondary) - more common
        'partial_black': random.randint(0, 3),
        'partial_sour': random.randint(0, 2),
        'parchment': random.randint(0, 2),
        'floater': random.randint(0, 1),
        'immature': random.randint(0, 3),
        'withered': random.randint(0, 2),
        'shell': random.randint(0, 2),
        'broken': random.randint(0, 4),
        'chipped': random.randint(0, 3),
        'cut': random.randint(0, 1),
        'insect_damage': random.randint(0, 2),
        'husk': random.randint(0, 1)
    }
    
    return {
        'total_beans': random.randint(300, 400),
        'defects': defects,
        'confidence': round(random.uniform(0.85, 0.98), 2),
        'processing_time_ms': random.randint(800, 2000),
        'annotated_image_url': None  # Would be set by actual model
    }


def format_detection_result(
    result: Dict[str, Any],
    request_id: str,
    image_key: str,
    processing_time_ms: int
) -> Dict[str, Any]:
    """Format model output into structured detection result."""
    defects = result.get('defects', {})
    
    # Calculate category counts
    category1_count = sum(defects.get(d, 0) for d in CATEGORY1_DEFECTS)
    category2_count = sum(defects.get(d, 0) for d in CATEGORY2_DEFECTS)
    
    # Build defect breakdown
    defect_breakdown = {
        # Category 1
        'full_black': defects.get('full_black', 0),
        'full_sour': defects.get('full_sour', 0),
        'pod_cherry': defects.get('pod_cherry', 0),
        'large_stones': defects.get('large_stones', 0),
        'medium_stones': defects.get('medium_stones', 0),
        'large_sticks': defects.get('large_sticks', 0),
        'medium_sticks': defects.get('medium_sticks', 0),
        # Category 2
        'partial_black': defects.get('partial_black', 0),
        'partial_sour': defects.get('partial_sour', 0),
        'parchment': defects.get('parchment', 0),
        'floater': defects.get('floater', 0),
        'immature': defects.get('immature', 0),
        'withered': defects.get('withered', 0),
        'shell': defects.get('shell', 0),
        'broken': defects.get('broken', 0),
        'chipped': defects.get('chipped', 0),
        'cut': defects.get('cut', 0),
        'insect_damage': defects.get('insect_damage', 0),
        'husk': defects.get('husk', 0)
    }
    
    return {
        'request_id': request_id,
        'image_url': f's3://{BUCKET_NAME}/{image_key}',
        'detected_beans': result.get('total_beans', 0),
        'defect_breakdown': defect_breakdown,
        'category1_count': category1_count,
        'category2_count': category2_count,
        'confidence_score': result.get('confidence', 0.0),
        'processing_time_ms': processing_time_ms,
        'annotated_image_url': result.get('annotated_image_url')
    }


def calculate_grade(category1_count: int, category2_count: int) -> str:
    """
    Calculate SCA grade classification from defect counts.
    
    Grade classifications:
    - Specialty Grade: 0-5 total defects, 0 category 1
    - Premium Grade: 0-8 total defects
    - Exchange Grade: 9-23 total defects
    - Below Standard: 24-86 total defects
    - Off Grade: 87+ total defects
    """
    total = category1_count + category2_count
    
    if category1_count == 0 and total <= 5:
        return 'specialty_grade'
    elif total <= 8:
        return 'premium_grade'
    elif total <= 23:
        return 'exchange_grade'
    elif total <= 86:
        return 'below_standard'
    else:
        return 'off_grade'


def success_response(data: Dict[str, Any]) -> Dict[str, Any]:
    """Create a successful API response."""
    return {
        'statusCode': 200,
        'headers': {
            'Content-Type': 'application/json',
            'Access-Control-Allow-Origin': '*'
        },
        'body': json.dumps(data)
    }


def error_response(status_code: int, message: str) -> Dict[str, Any]:
    """Create an error API response."""
    return {
        'statusCode': status_code,
        'headers': {
            'Content-Type': 'application/json',
            'Access-Control-Allow-Origin': '*'
        },
        'body': json.dumps({
            'error': message,
            'error_th': translate_error(message)
        })
    }


def translate_error(message: str) -> str:
    """Translate error message to Thai."""
    translations = {
        'Missing required field: image_base64': 'ต้องระบุข้อมูลรูปภาพ',
        'Invalid base64 image data': 'ข้อมูลรูปภาพไม่ถูกต้อง',
        'Internal server error': 'เกิดข้อผิดพลาดภายในระบบ'
    }
    
    for eng, thai in translations.items():
        if eng in message:
            return thai
    
    return 'เกิดข้อผิดพลาด'

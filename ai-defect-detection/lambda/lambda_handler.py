"""
Coffee Bean Defect Detection Lambda Handler

Uses HuggingFace everycoffee/autotrain-coffee-bean-quality model for binary
defect detection (defect/good), with SCA grade estimation.
"""

import json
import boto3
import base64
import os
import uuid
from datetime import datetime
from typing import Any, Dict
from io import BytesIO

s3 = boto3.client('s3')

BUCKET_NAME = os.environ.get('BUCKET_NAME', 'coffee-qm-images')
HF_MODEL = os.environ.get('HF_MODEL', 'everycoffee/autotrain-coffee-bean-quality-97496146930')

_classifier = None

def get_classifier():
    """Lazy load HuggingFace classifier to reduce cold start."""
    global _classifier
    if _classifier is None:
        from transformers import pipeline
        _classifier = pipeline("image-classification", model=HF_MODEL)
    return _classifier


def handler(event: Dict[str, Any], context: Any) -> Dict[str, Any]:
    """
    Main Lambda handler for defect detection.
    
    Request: {"image_base64": "..."}
    Response: {"request_id": "...", "detection": {...}, "suggested_grade": "..."}
    """
    try:
        body = json.loads(event.get('body', '{}'))
        image_base64 = body.get('image_base64')
        
        if not image_base64:
            return error_response(400, 'Missing required field: image_base64')
        
        request_id = f"det-{datetime.utcnow().strftime('%Y%m%d%H%M%S')}-{str(uuid.uuid4())[:8]}"
        start_time = datetime.utcnow()
        
        try:
            image_bytes = base64.b64decode(image_base64)
        except Exception as e:
            return error_response(400, f'Invalid base64 image data: {str(e)}')
        
        # Upload to S3
        image_key = f'uploads/{request_id}.jpg'
        s3.put_object(Bucket=BUCKET_NAME, Key=image_key, Body=image_bytes, ContentType='image/jpeg')
        
        # Run inference
        model_result = invoke_model(image_bytes)
        processing_time_ms = int((datetime.utcnow() - start_time).total_seconds() * 1000)
        
        detection = {
            'request_id': request_id,
            'image_url': f's3://{BUCKET_NAME}/{image_key}',
            'is_defective': model_result['is_defective'],
            'defect_probability': model_result['defect_probability'],
            'confidence_score': model_result['confidence'],
            'processing_time_ms': processing_time_ms,
            'model': HF_MODEL,
            'note': 'Binary classification only. Fine-tuned model needed for SCA defect breakdown.'
        }
        
        suggested_grade = 'needs_inspection' if model_result['is_defective'] else 'likely_specialty'
        
        return success_response({'request_id': request_id, 'detection': detection, 'suggested_grade': suggested_grade})
        
    except Exception as e:
        print(f'Error: {str(e)}')
        return error_response(500, f'Internal server error: {str(e)}')


def invoke_model(image_bytes: bytes) -> Dict[str, Any]:
    """Invoke HuggingFace model for binary defect detection."""
    from PIL import Image
    
    image = Image.open(BytesIO(image_bytes)).convert('RGB')
    results = get_classifier()(image)
    
    defect_score = next((r['score'] for r in results if r['label'].lower() == 'defect'), 0.0)
    good_score = next((r['score'] for r in results if r['label'].lower() != 'defect'), 0.0)
    
    return {
        'is_defective': defect_score > good_score,
        'defect_probability': defect_score,
        'confidence': max(defect_score, good_score)
    }


def success_response(data: Dict[str, Any]) -> Dict[str, Any]:
    return {
        'statusCode': 200,
        'headers': {'Content-Type': 'application/json', 'Access-Control-Allow-Origin': '*'},
        'body': json.dumps(data)
    }


def error_response(status_code: int, message: str) -> Dict[str, Any]:
    return {
        'statusCode': status_code,
        'headers': {'Content-Type': 'application/json', 'Access-Control-Allow-Origin': '*'},
        'body': json.dumps({'error': message})
    }

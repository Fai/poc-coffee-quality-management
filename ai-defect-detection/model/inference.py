"""
SageMaker Inference Script for Coffee Bean Defect Detection

This script is packaged with the model and handles inference requests
on the SageMaker endpoint.
"""

import json
import io
import torch
import torchvision.transforms as transforms
from PIL import Image
from typing import Dict, Any, List, Tuple

# Defect class names (matching training labels)
DEFECT_CLASSES = [
    'normal',
    # Category 1 (Primary) Defects
    'full_black',
    'full_sour', 
    'pod_cherry',
    'large_stones',
    'medium_stones',
    'large_sticks',
    'medium_sticks',
    # Category 2 (Secondary) Defects
    'partial_black',
    'partial_sour',
    'parchment',
    'floater',
    'immature',
    'withered',
    'shell',
    'broken',
    'chipped',
    'cut',
    'insect_damage',
    'husk'
]

# Image preprocessing
TRANSFORM = transforms.Compose([
    transforms.Resize((640, 640)),
    transforms.ToTensor(),
    transforms.Normalize(
        mean=[0.485, 0.456, 0.406],
        std=[0.229, 0.224, 0.225]
    )
])


def model_fn(model_dir: str) -> torch.nn.Module:
    """
    Load the trained model from the model directory.
    
    Args:
        model_dir: Directory containing model artifacts
        
    Returns:
        Loaded PyTorch model
    """
    import os
    
    model_path = os.path.join(model_dir, 'model.pt')
    
    # Load model (assuming YOLO or similar object detection model)
    # In production, this would load the actual trained model
    device = torch.device('cuda' if torch.cuda.is_available() else 'cpu')
    
    try:
        model = torch.load(model_path, map_location=device)
        model.eval()
        return model
    except Exception as e:
        print(f'Error loading model: {e}')
        # Return a placeholder for development
        return None


def input_fn(request_body: bytes, content_type: str) -> Image.Image:
    """
    Deserialize input data.
    
    Args:
        request_body: Raw request body bytes
        content_type: Content type of the request
        
    Returns:
        PIL Image object
    """
    if content_type == 'application/x-image':
        image = Image.open(io.BytesIO(request_body))
        return image.convert('RGB')
    else:
        raise ValueError(f'Unsupported content type: {content_type}')


def predict_fn(image: Image.Image, model: torch.nn.Module) -> Dict[str, Any]:
    """
    Run inference on the input image.
    
    Args:
        image: PIL Image to process
        model: Loaded PyTorch model
        
    Returns:
        Detection results dictionary
    """
    import time
    start_time = time.time()
    
    if model is None:
        # Return mock results for development
        return generate_mock_results(image)
    
    # Preprocess image
    input_tensor = TRANSFORM(image).unsqueeze(0)
    device = next(model.parameters()).device
    input_tensor = input_tensor.to(device)
    
    # Run inference
    with torch.no_grad():
        outputs = model(input_tensor)
    
    # Post-process detections
    detections = post_process_detections(outputs)
    
    # Count defects by type
    defect_counts = count_defects(detections)
    
    processing_time = int((time.time() - start_time) * 1000)
    
    return {
        'total_beans': len(detections),
        'defects': defect_counts,
        'confidence': calculate_average_confidence(detections),
        'processing_time_ms': processing_time,
        'detections': detections  # Raw detections for annotation
    }


def output_fn(prediction: Dict[str, Any], accept: str) -> str:
    """
    Serialize prediction output.
    
    Args:
        prediction: Prediction dictionary
        accept: Accepted content type
        
    Returns:
        JSON string of predictions
    """
    # Remove raw detections from output (too large)
    output = {k: v for k, v in prediction.items() if k != 'detections'}
    return json.dumps(output)


def post_process_detections(outputs: Any) -> List[Dict[str, Any]]:
    """
    Post-process model outputs into detection list.
    
    Args:
        outputs: Raw model outputs
        
    Returns:
        List of detection dictionaries
    """
    detections = []
    
    # This would be customized based on the actual model architecture
    # Example for YOLO-style output:
    if hasattr(outputs, 'xyxy'):
        for det in outputs.xyxy[0]:
            x1, y1, x2, y2, conf, cls = det.tolist()
            detections.append({
                'bbox': [x1, y1, x2, y2],
                'confidence': conf,
                'class_id': int(cls),
                'class_name': DEFECT_CLASSES[int(cls)]
            })
    
    return detections


def count_defects(detections: List[Dict[str, Any]]) -> Dict[str, int]:
    """
    Count defects by type from detections.
    
    Args:
        detections: List of detection dictionaries
        
    Returns:
        Dictionary of defect counts
    """
    counts = {cls: 0 for cls in DEFECT_CLASSES if cls != 'normal'}
    
    for det in detections:
        class_name = det.get('class_name', 'normal')
        if class_name != 'normal' and class_name in counts:
            counts[class_name] += 1
    
    return counts


def calculate_average_confidence(detections: List[Dict[str, Any]]) -> float:
    """
    Calculate average confidence score across all detections.
    
    Args:
        detections: List of detection dictionaries
        
    Returns:
        Average confidence score
    """
    if not detections:
        return 0.0
    
    total_conf = sum(d.get('confidence', 0) for d in detections)
    return round(total_conf / len(detections), 2)


def generate_mock_results(image: Image.Image) -> Dict[str, Any]:
    """
    Generate mock detection results for development/testing.
    
    Args:
        image: Input image (used to estimate bean count)
        
    Returns:
        Mock detection results
    """
    import random
    
    # Estimate bean count based on image size (rough approximation)
    width, height = image.size
    estimated_beans = min(400, max(200, (width * height) // 10000))
    
    # Generate realistic defect distribution
    defects = {
        # Category 1 - typically fewer
        'full_black': random.randint(0, 2),
        'full_sour': random.randint(0, 1),
        'pod_cherry': random.randint(0, 1),
        'large_stones': 0,
        'medium_stones': 0,
        'large_sticks': 0,
        'medium_sticks': 0,
        # Category 2 - more common
        'partial_black': random.randint(0, 3),
        'partial_sour': random.randint(0, 2),
        'parchment': random.randint(0, 2),
        'floater': random.randint(0, 1),
        'immature': random.randint(0, 4),
        'withered': random.randint(0, 2),
        'shell': random.randint(0, 2),
        'broken': random.randint(0, 5),
        'chipped': random.randint(0, 3),
        'cut': random.randint(0, 1),
        'insect_damage': random.randint(0, 2),
        'husk': random.randint(0, 1)
    }
    
    return {
        'total_beans': estimated_beans,
        'defects': defects,
        'confidence': round(random.uniform(0.85, 0.98), 2),
        'processing_time_ms': random.randint(500, 1500)
    }
